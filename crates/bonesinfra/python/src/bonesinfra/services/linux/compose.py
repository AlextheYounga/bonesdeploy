from shlex import quote

from pyinfra.context import ctx_host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, files, server, systemd

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.services.linux import runtime, systemd as site_systemd
from bonesinfra.services.linux.nginx import site as nginx_site

_STALE_SERVICES = (
    "cloudflared",
    "docker",
    "nginx",
    "worker",
    "django",
    "gunicorn",
    "next",
    "nuxt",
    "puma",
    "rails",
    "sveltekit",
    "vue",
)


def deploy(ctx) -> None:
    _remove_stale_managed_runtime(ctx)
    _install_docker()
    paths = ctx.paths_dict
    site_systemd.render_target(ctx, paths=paths)
    files.template(
        name="Deploy Compose runtime service",
        src=str(ASSETS_DIR / "systemd/compose.service.j2"),
        dest=ctx.paths.systemd_service("compose"),
        user="root",
        group="root",
        mode="0644",
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    site_systemd.register_service(ctx, paths=paths, name="compose")
    if ctx.runtime.compose_port is not None:
        runtime.setup(ctx, uses_tcp=True)
        nginx_site.render_proxy(
            ctx,
            paths=paths,
            template_src=ASSETS_DIR / "nginx/compose-site-nginx.conf.j2",
            port=ctx.runtime.compose_port,
        )
        runtime.reconcile_ingress(ctx)
    site_systemd.enable_and_start(ctx, "compose")
    if ctx.runtime.compose_port is not None:
        runtime.start_services(ctx)


def artifacts(ctx):
    artifacts = [
        ("Compose runtime service", ctx.paths.systemd_service("compose"), "file", "docker"),
        ("Compose runtime requirement", ctx.paths.systemd_service_requirement("compose"), "link", "docker"),
    ]
    if ctx.runtime.compose_port is not None:
        artifacts.extend(
            [
                ("Compose nginx configuration", ctx.paths.site_nginx_config, "file", "docker"),
                ("Compose nginx site", ctx.paths.nginx_site_available, "file", "docker"),
                ("Enabled Compose nginx site", ctx.paths.nginx_site_enabled, "link", "docker"),
            ]
        )
    return artifacts


def services(_ctx):
    return [("Compose stack", "{project}-compose.service", "docker")]


def mode(_ctx):
    return "compose"


def _install_docker() -> None:
    distribution, codename = _docker_repository(ctx_host.get().get_fact(LinuxDistribution))
    apt.packages(
        name="Install Docker repository prerequisites",
        packages=["ca-certificates"],
        present=True,
        update=True,
        _sudo=True,
    )
    files.directory(
        name="Ensure APT keyring directory exists",
        path="/etc/apt/keyrings",
        user="root",
        group="root",
        mode="0755",
        _sudo=True,
    )
    files.download(
        name="Install Docker package signing key",
        src=f"https://download.docker.com/linux/{distribution}/gpg",
        dest="/etc/apt/keyrings/docker.asc",
        user="root",
        group="root",
        mode="0644",
        _sudo=True,
    )
    files.template(
        name="Install Docker package source",
        src=str(ASSETS_DIR / "apt/docker.sources.j2"),
        dest="/etc/apt/sources.list.d/docker.sources",
        distribution=distribution,
        codename=codename,
        user="root",
        group="root",
        mode="0644",
        _sudo=True,
    )
    apt.packages(
        name="Remove packages that conflict with Docker Engine",
        packages=["docker.io", "docker-compose", "podman-docker", "containerd", "runc"],
        present=False,
        _sudo=True,
    )
    apt.packages(
        name="Install Docker Engine and Compose plugin",
        packages=["docker-ce", "docker-ce-cli", "containerd.io", "docker-buildx-plugin", "docker-compose-plugin"],
        present=True,
        update=True,
        _sudo=True,
    )
    systemd.service(name="Enable Docker Engine", service="docker", enabled=True, running=True, _sudo=True)
    server.shell(name="Verify Docker Compose plugin", commands=["docker compose version >/dev/null"], _sudo=True)


def _docker_repository(distribution: dict | None) -> tuple[str, str]:
    release = distribution.get("release_meta", {}) if distribution else {}
    distribution_id = str(release.get("ID", "")).lower()
    codename = release.get("VERSION_CODENAME") or release.get("CODENAME") or release.get("DISTRIB_CODENAME")
    if distribution_id not in {"debian", "ubuntu"} or not codename:
        raise ValueError("Docker Compose provisioning requires a supported Debian or Ubuntu release with a codename")
    return distribution_id, str(codename)


def _remove_stale_managed_runtime(ctx) -> None:
    project = quote(ctx.app.project_name)
    unit_names = " ".join(f"{ctx.app.project_name}-{name}.service" for name in _STALE_SERVICES)
    units = " ".join(quote(ctx.paths.systemd_service(name)) for name in _STALE_SERVICES)
    requirements = " ".join(
        quote(f"{ctx.paths.systemd_site_target_requires}/{ctx.app.project_name}-{name}.service")
        for name in _STALE_SERVICES
    )
    apparmor = " ".join(quote(ctx.paths.apparmor_profile(name)) for name in _STALE_SERVICES if name != "docker")
    server.shell(
        name="Stop stale managed native runtime services",
        commands=[
            f"systemctl disable --now {unit_names} 2>/dev/null || true",
            f"rm -f -- {requirements} {units} {apparmor} "
            f"{quote(ctx.paths.site_nginx_config)} {quote(ctx.paths.nginx_site_available)} "
            f"{quote(ctx.paths.nginx_site_enabled)}",
        ],
        _sudo=True,
    )
    server.shell(
        name="Remove stale Laravel Docker artifacts",
        commands=[
            f"rm -rf -- /var/lib/bonesdeploy/runtime-images/{project}",
            f"rm -f -- /etc/php/*/fpm/pool.d/{project}.conf /run/{project}/php-fpm.sock",
        ],
        _sudo=True,
    )
    systemd.daemon_reload(name="Reload systemd after stale runtime removal", _sudo=True)
    systemd.service(name="Reload nginx after stale ingress removal", service="nginx", reloaded=True, _sudo=True)
