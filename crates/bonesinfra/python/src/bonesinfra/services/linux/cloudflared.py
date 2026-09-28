import zlib
from shlex import quote

from pyinfra.operations import apt, files, server, systemd

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR, SCRIPTS_DIR
from bonesinfra.pyinfra.operations import render
from bonesinfra.services.linux.nginx.router import validate_config

_PORT_MIN = 20_000
_PORT_RANGE = 40_000


def port_for(project_name: str) -> int:
    """Return a stable unprivileged loopback port for a project."""
    return _PORT_MIN + zlib.crc32(project_name.encode("utf-8")) % _PORT_RANGE


def validate_supported(ctx) -> None:
    if ctx.runtime.backend == "docker" and ctx.runtime.compose_port is None:
        raise ValueError("Quick Tunnel requires Docker Compose managed nginx ingress (site.compose_port)")


def install():
    """Install cloudflared from Cloudflare's supported package repository."""
    files.directory(
        name="Ensure APT keyring directory exists",
        path="/etc/apt/keyrings",
        user="root",
        group="root",
        mode="0755",
        _sudo=True,
    )
    files.download(
        name="Install Cloudflare package signing key",
        src="https://pkg.cloudflare.com/cloudflare-main.gpg",
        dest="/etc/apt/keyrings/cloudflare-main.gpg",
        user="root",
        group="root",
        mode="0644",
        _sudo=True,
    )
    files.template(
        name="Install Cloudflare package source",
        src=str(ASSETS_DIR / "apt/cloudflared.list.j2"),
        dest="/etc/apt/sources.list.d/cloudflared.list",
        user="root",
        group="root",
        mode="0644",
        _sudo=True,
    )
    apt.packages(name="Install cloudflared", packages=["cloudflared"], present=True, update=True, _sudo=True)


def start(ctx, paths):
    validate_supported(ctx)
    port = port_for(ctx.app.project_name)
    target_requirement = f"{paths['systemd_site_target_requires']}/{ctx.app.project_name}-cloudflared.service"
    install()
    render(
        "Deploy Cloudflared nginx loopback route",
        ASSETS_DIR / "nginx/cloudflared.conf.j2",
        paths["nginx_cloudflared_site_available"],
        cloudflared_port=port,
        paths=paths,
    )
    server.script(
        name="Enable and validate Cloudflared nginx loopback route",
        src=str(SCRIPTS_DIR / "enable-nginx-site.sh"),
        args=(paths["nginx_cloudflared_site_available"], paths["nginx_cloudflared_site_enabled"]),
        _sudo=True,
    )
    systemd.service(
        name="Reload nginx for Cloudflared loopback route",
        service="nginx",
        reloaded=True,
        _sudo=True,
    )
    files.template(
        name="Deploy project quick tunnel service",
        src=str(ASSETS_DIR / "systemd/cloudflared.service.j2"),
        dest=paths["systemd_cloudflared_service"],
        user="root",
        group="root",
        mode="0644",
        cloudflared_port=port,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    server.shell(
        name="Keep quick tunnel outside the site target",
        commands=[f"rm -f -- {quote(target_requirement)}"],
        _sudo=True,
    )
    systemd.daemon_reload(name="Reload systemd after quick tunnel change", _sudo=True)
    systemd.service(
        name="Enable and start project quick tunnel",
        service=f"{ctx.app.project_name}-cloudflared.service",
        enabled=True,
        restarted=True,
        _sudo=True,
    )


def stop(ctx, paths):
    service_name = f"{ctx.app.project_name}-cloudflared.service"
    quoted_service = quote(service_name)
    server.shell(
        name="Stop and remove project quick tunnel",
        commands=[
            f"if systemctl cat -- {quoted_service} >/dev/null 2>&1; "
            f"then systemctl disable --now -- {quoted_service}; fi; "
            f"rm -f -- {quote(paths['systemd_cloudflared_service'])} "
            f"{quote(paths['nginx_cloudflared_site_enabled'])} "
            f"{quote(paths['nginx_cloudflared_site_available'])} "
            f"{quote(paths['systemd_site_target_requires'] + '/' + service_name)}",
        ],
        _sudo=True,
    )
    systemd.daemon_reload(name="Reload systemd after quick tunnel removal", _sudo=True)
    validate_config("Validate nginx configuration after Cloudflared removal")
    systemd.service(
        name="Reload nginx after Cloudflared removal",
        service="nginx",
        reloaded=True,
        _sudo=True,
    )
