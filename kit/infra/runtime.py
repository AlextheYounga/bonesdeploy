import os
import importlib.util

from pyinfra import host
from pyinfra.operations import apt, files, server, systemd


def _unflatten(data_dict):
    result = {}
    for key, value in data_dict.items():
        parts = key.split(".")
        node = result
        for part in parts[:-1]:
            if part not in node:
                node[part] = {}
            node = node[part]
        node[parts[-1]] = value
    return result


DEPLOY_DATA = _unflatten(host.data.dict())
PATHS = DEPLOY_DATA.get("paths", {})

# Install runtime apt packages
pkgs = DEPLOY_DATA.get("runtime_apt_packages", [])
if pkgs:
    apt.packages(
        name="Install runtime apt packages",
        packages=pkgs,
        present=True,
        update=True,
        cache_time=3600,
        _sudo=True,
    )

# Template-specific runtime setup
if DEPLOY_DATA.get("template"):
    ops_path = os.path.join(os.path.dirname(__file__), "operations.py")
    if os.path.exists(ops_path):
        spec = importlib.util.spec_from_file_location("operations", ops_path)
        ops = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(ops)

# --- AppArmor ---
systemd.service(
    name="Ensure apparmor service is enabled and started",
    service="apparmor",
    enabled=True,
    running=True,
    _sudo=True,
)

server.shell(
    name="Verify apparmor kernel enabled",
    commands=[f"cat {PATHS['apparmor_enabled_param']}"],
    _sudo=True,
)

apparmor_profile_name = f"bonesdeploy-{DEPLOY_DATA["project_name"]}-nginx"
apparmor_profile_path = f"/etc/apparmor.d/{apparmor_profile_name}"

files.template(
    name="Deploy per-project apparmor profile",
    src=os.path.join(os.path.dirname(__file__), "assets/apparmor/project-nginx-profile.j2"),
    dest=apparmor_profile_path,
    user="root",
    group="root",
    mode="0644",
    apparmor_profile_name=apparmor_profile_name,
    **DEPLOY_DATA,
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

# --- Nginx ---

server.group(
    name="Create per-project runtime group",
    group=DEPLOY_DATA["project_name"],
    system=True,
    _sudo=True,
)

server.user(
    name="Add service user to project group",
    user=DEPLOY_DATA["service_user"],
    groups=[DEPLOY_DATA["project_name"]],
    append=True,
    _sudo=True,
)

files.directory(
    name="Ensure socket directory exists",
    path=PATHS["runtime_socket_dir"],
    user=DEPLOY_DATA["service_user"],
    group=DEPLOY_DATA["group"],
    mode="0750",
    _sudo=True,
)

files.directory(
    name="Ensure conf directory exists",
    path=PATHS["conf_root"],
    user="root",
    group=DEPLOY_DATA["group"],
    mode="0750",
    _sudo=True,
)

here = os.path.dirname(__file__)

files.template(
    name="Deploy per-site nginx config",
    src=os.path.join(here, "assets/nginx/site-nginx.conf.j2"),
    dest=PATHS["site_nginx_config"],
    user="root",
    group=DEPLOY_DATA["group"],
    mode="0640",
    **DEPLOY_DATA,
    _sudo=True,
)

files.template(
    name="Deploy per-site nginx systemd service",
    src=os.path.join(here, "assets/nginx/site-nginx.service.j2"),
    dest=PATHS["systemd_site_nginx_service"],
    user="root",
    group="root",
    mode="0644",
    **DEPLOY_DATA,
    _sudo=True,
)

systemd.daemon_reload(
    name="Reload systemd after site-nginx service change",
    _sudo=True,
)

nginx_server_name = DEPLOY_DATA.get("ssl_domain", "_")
nginx_ssl_enabled = bool(DEPLOY_DATA.get("ssl_cert_path") and DEPLOY_DATA.get("ssl_key_path"))

files.template(
    name="Deploy router nginx config",
    src=os.path.join(here, "assets/nginx/router.conf.j2"),
    dest=PATHS["nginx_site_available"],
    user="root",
    group="root",
    mode="0644",
    nginx_server_name=nginx_server_name,
    nginx_ssl_enabled=nginx_ssl_enabled,
    nginx_ssl_certificate_path=DEPLOY_DATA.get("ssl_cert_path", ""),
    nginx_ssl_certificate_key_path=DEPLOY_DATA.get("ssl_key_path", ""),
    **DEPLOY_DATA,
    _sudo=True,
)

files.link(
    name="Enable router nginx site",
    path=PATHS["nginx_site_enabled"],
    target=PATHS["nginx_site_available"],
    force=True,
    _sudo=True,
)

files.file(
    name="Disable default nginx site",
    path=PATHS["nginx_default_site_enabled"],
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

site_name = os.path.basename(PATHS["systemd_site_nginx_service"]).replace(".service", "")
systemd.service(
    name="Ensure per-site nginx service is enabled and started",
    service=site_name,
    enabled=True,
    running=True,
    daemon_reload=True,
    _sudo=True,
)

# --- Post-task: doctor ---

server.shell(
    name="Run bonesremote doctor as deploy user",
    commands=["/usr/local/bin/bonesremote doctor"],
    _sudo=True,
    _sudo_user=DEPLOY_DATA["deploy_user"],
)
