import os

from pyinfra import host
from pyinfra.operations import apt, files, server, systemd
from src.utils import unflatten


def deploy():
    data = unflatten(host.data.dict())
    paths = data.get("paths", {})
    here = os.path.dirname(__file__)

    install_runtime_apt_packages(data)
    setup_apparmor(data, paths, here)
    setup_nginx(data, paths, here)
    setup_template_runtime(data)
    run_doctor(data)


def install_runtime_apt_packages(data):
    pkgs = data.get("runtime_apt_packages", [])
    if not pkgs:
        return
    apt.packages(
        name="Install runtime apt packages",
        packages=pkgs,
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )


def setup_apparmor(data, paths, here):
    systemd.service(
        name="Ensure apparmor service is enabled and started",
        service="apparmor",
        enabled=True,
        running=True,
        _sudo=True,
    )

    server.shell(
        name="Verify apparmor kernel enabled",
        commands=[f"cat {paths['apparmor_enabled_param']}"],
        _sudo=True,
    )

    apparmor_profile_name = f"bonesdeploy-{data['project_name']}-nginx"
    apparmor_profile_path = f"/etc/apparmor.d/{apparmor_profile_name}"

    files.template(
        name="Deploy per-project apparmor profile",
        src=os.path.join(here, "assets/apparmor/project-nginx-profile.j2"),
        dest=apparmor_profile_path,
        user="root",
        group="root",
        mode="0644",
        apparmor_profile_name=apparmor_profile_name,
        **data,
        _sudo=True,
    )

    server.shell(
        name="Load updated apparmor profile",
        commands=[f"apparmor_parser -r {apparmor_profile_path}"],
        _sudo=True,
    )

    server.shell(
        name="Ensure project profile is in enforce mode",
        commands=[f"aa-enforce {apparmor_profile_path}"],
        _sudo=True,
    )


def setup_nginx(data, paths, here):
    files.directory(
        name="Ensure socket directory exists",
        path=paths["runtime_socket_dir"],
        user=data["runtime_user"],
        group=data["runtime_group"],
        mode="0750",
        _sudo=True,
    )

    files.directory(
        name="Ensure conf directory exists",
        path=paths["conf_root"],
        user="root",
        group=data["runtime_group"],
        mode="0750",
        _sudo=True,
    )

    files.template(
        name="Deploy per-site nginx config",
        src=os.path.join(here, "assets/nginx/site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=data["runtime_group"],
        mode="0640",
        **data,
        _sudo=True,
    )

    files.template(
        name="Deploy per-site nginx systemd service",
        src=os.path.join(here, "assets/nginx/site-nginx.service.j2"),
        dest=paths["systemd_site_nginx_service"],
        user="root",
        group="root",
        mode="0644",
        **data,
        _sudo=True,
    )

    systemd.daemon_reload(
        name="Reload systemd after site-nginx service change",
        _sudo=True,
    )

    nginx_server_name = data.get("ssl_domain", "_")
    nginx_ssl_enabled = bool(data.get("ssl_cert_path") and data.get("ssl_key_path"))

    files.template(
        name="Deploy router nginx config",
        src=os.path.join(here, "assets/nginx/router.conf.j2"),
        dest=paths["nginx_site_available"],
        user="root",
        group="root",
        mode="0644",
        nginx_server_name=nginx_server_name,
        nginx_ssl_enabled=nginx_ssl_enabled,
        nginx_ssl_certificate_path=data.get("ssl_cert_path", ""),
        nginx_ssl_certificate_key_path=data.get("ssl_key_path", ""),
        **data,
        _sudo=True,
    )

    files.link(
        name="Enable router nginx site",
        path=paths["nginx_site_enabled"],
        target=paths["nginx_site_available"],
        force=True,
        _sudo=True,
    )

    files.link(
        name="Disable default nginx site",
        path=paths["nginx_default_site_enabled"],
        present=False,
        _sudo=True,
    )

    server.shell(
        name="Validate nginx configuration",
        commands=["nginx -t"],
        _sudo=True,
    )

    systemd.service(
        name="Ensure nginx service is enabled and started",
        service="nginx",
        enabled=True,
        running=True,
        _sudo=True,
    )

    site_name = os.path.basename(paths["systemd_site_nginx_service"]).replace(".service", "")
    systemd.service(
        name="Ensure per-site nginx service is enabled and started",
        service=site_name,
        enabled=True,
        running=True,
        daemon_reload=True,
        _sudo=True,
    )


def setup_template_runtime(data):
    template = data.get("template")
    if not template:
        return
    try:
        from src.runtimes import get_runtime
        mod = get_runtime(template)
        if hasattr(mod, "deploy"):
            mod.deploy()
    except (ImportError, KeyError):
        pass


def run_doctor(data):
    server.shell(
        name="Run bonesremote doctor as deploy user",
        commands=["/usr/local/bin/bonesremote doctor"],
        _sudo=True,
        _sudo_user=data["deploy_user"],
    )

