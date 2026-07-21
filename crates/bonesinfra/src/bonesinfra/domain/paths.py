from dataclasses import dataclass
from pathlib import Path

ASSETS_DIR = Path(__file__).parent.parent / "assets"
SCRIPTS_DIR = ASSETS_DIR / "scripts"

DEFAULT_REPO_PARENT = "/srv/git"
IMAGE_STORE_GRAPH_ROOT = "/var/lib/bonesdeploy/image-store"
IMAGE_STORE_RUN_ROOT = "/run/bonesdeploy/image-store"
BUILD_CACHE_NAME = "cache"
DEFAULT_PROJECT_ROOT_PARENT = "/srv/sites"
DEFAULT_CONF_ROOT_PARENT = "/srv/conf"
DEFAULT_WEB_ROOT = "public"

ETC_NGINX_SITES_AVAILABLE = "/etc/nginx/sites-available"
ETC_NGINX_SITES_ENABLED = "/etc/nginx/sites-enabled"
ETC_SYSTEMD_SYSTEM = "/etc/systemd/system"
ETC_APPARMOR_D = "/etc/apparmor.d"
ETC_SSL_CERTS = "/etc/ssl/certs"
ETC_SSL_PRIVATE = "/etc/ssl/private"
ETC_SUDOERS_D = "/etc/sudoers.d"

RUNTIME_SOCKET_PARENT = "/run"
NGINX_CONF = "nginx.conf"
INDEX_HTML = "index.html"
GIT_HEAD = "HEAD"
RELEASES_DIR = "releases"
SHARED_DIR = "shared"
CURRENT_LINK = "current"
PLACEHOLDER_RELEASE_NAME = "19700101_000000"

NGINX_SOCKET = "nginx.sock"
NGINX_PID = "nginx.pid"
PHP_FPM_SOCKET = "php-fpm.sock"
DEFAULT_NGINX_SITE = "default"
BONESDEPLOY_NGINX_DEFAULT_DENY_SITE = "00-bonesdeploy-default-deny.conf"
BONESDEPLOY_NGINX_DEFAULT_DENY_CERT = "bonesdeploy-default-deny.crt"
BONESDEPLOY_NGINX_DEFAULT_DENY_KEY = "bonesdeploy-default-deny.key"

BONESREMOTE_BINARY = "bonesremote"

USR_LOCAL_BIN = "/usr/local/bin"
APPARMOR_ENABLED_PARAM = "/sys/module/apparmor/parameters/enabled"
APPARMOR_PROFILES = "/sys/kernel/security/apparmor/profiles"

BONESDEPLOY_REPO = "https://github.com/AlextheYounga/bonesdeploy.git"


def _parent_or_default(path: str, fallback: str) -> str:
    parent = str(Path(path).parent)
    return fallback if parent == "." else parent


@dataclass
class DeploymentPaths:
    repo: str
    repo_parent: str
    repo_head: str
    site_nginx_config: str
    conf_root: str
    project_root: str
    project_root_parent: str
    releases: str
    shared: str
    current: str
    current_web_root: str
    placeholder_release: str
    placeholder_web_root: str
    placeholder_index: str
    nginx_site_available: str
    nginx_site_enabled: str
    nginx_default_deny_site_available: str
    nginx_default_deny_site_enabled: str
    nginx_default_deny_ssl_certificate: str
    nginx_default_deny_ssl_certificate_key: str
    nginx_default_site_enabled: str
    systemd_site_nginx_service: str
    systemd_site_target: str
    systemd_site_target_requires: str
    runtime_socket_dir: str
    runtime_nginx_dir: str
    runtime_nginx_socket: str
    runtime_nginx_pid: str
    runtime_php_fpm_socket: str
    acme_webroot: str
    sudoers_path: str
    usr_local_bin: str
    bonesremote_global_link: str
    apparmor_enabled_param: str
    apparmor_profiles: str

    @classmethod
    def new(
        cls,
        project_name: str,
        repo_path: str,
        project_root: str,
        web_root: str | None = None,
    ) -> "DeploymentPaths":
        if web_root is None:
            web_root = DEFAULT_WEB_ROOT

        placeholder_release = Path(project_root) / RELEASES_DIR / PLACEHOLDER_RELEASE_NAME
        current = Path(project_root) / CURRENT_LINK
        runtime_socket_dir = Path(RUNTIME_SOCKET_PARENT) / project_name
        runtime_nginx_dir = runtime_socket_dir / "nginx"
        conf_root = Path(DEFAULT_CONF_ROOT_PARENT) / project_name

        return cls(
            repo=repo_path,
            repo_parent=_parent_or_default(repo_path, DEFAULT_REPO_PARENT),
            repo_head=str(Path(repo_path) / GIT_HEAD),
            site_nginx_config=str(conf_root / NGINX_CONF),
            conf_root=str(conf_root),
            project_root=project_root,
            project_root_parent=_parent_or_default(project_root, DEFAULT_PROJECT_ROOT_PARENT),
            releases=str(Path(project_root) / RELEASES_DIR),
            shared=str(Path(project_root) / SHARED_DIR),
            current=str(current),
            current_web_root=str(current / web_root),
            placeholder_release=str(placeholder_release),
            placeholder_web_root=str(placeholder_release / web_root),
            placeholder_index=str(placeholder_release / web_root / INDEX_HTML),
            nginx_site_available=str(Path(ETC_NGINX_SITES_AVAILABLE) / f"{project_name}.conf"),
            nginx_site_enabled=str(Path(ETC_NGINX_SITES_ENABLED) / f"{project_name}.conf"),
            nginx_default_deny_site_available=str(
                Path(ETC_NGINX_SITES_AVAILABLE) / BONESDEPLOY_NGINX_DEFAULT_DENY_SITE
            ),
            nginx_default_deny_site_enabled=str(Path(ETC_NGINX_SITES_ENABLED) / BONESDEPLOY_NGINX_DEFAULT_DENY_SITE),
            nginx_default_deny_ssl_certificate=str(Path(ETC_SSL_CERTS) / BONESDEPLOY_NGINX_DEFAULT_DENY_CERT),
            nginx_default_deny_ssl_certificate_key=str(Path(ETC_SSL_PRIVATE) / BONESDEPLOY_NGINX_DEFAULT_DENY_KEY),
            nginx_default_site_enabled=str(Path(ETC_NGINX_SITES_ENABLED) / DEFAULT_NGINX_SITE),
            systemd_site_nginx_service=str(Path(ETC_SYSTEMD_SYSTEM) / f"{project_name}-nginx.service"),
            systemd_site_target=str(Path(ETC_SYSTEMD_SYSTEM) / f"{project_name}.target"),
            systemd_site_target_requires=str(Path(ETC_SYSTEMD_SYSTEM) / f"{project_name}.target.requires"),
            runtime_socket_dir=str(runtime_socket_dir),
            runtime_nginx_dir=str(runtime_nginx_dir),
            runtime_nginx_socket=str(runtime_nginx_dir / NGINX_SOCKET),
            runtime_nginx_pid=str(runtime_nginx_dir / NGINX_PID),
            runtime_php_fpm_socket=str(runtime_socket_dir / PHP_FPM_SOCKET),
            acme_webroot=f"/var/www/{project_name}",
            sudoers_path=str(Path(ETC_SUDOERS_D) / "bonesdeploy"),
            usr_local_bin=USR_LOCAL_BIN,
            bonesremote_global_link=str(Path(USR_LOCAL_BIN) / BONESREMOTE_BINARY),
            apparmor_enabled_param=APPARMOR_ENABLED_PARAM,
            apparmor_profiles=APPARMOR_PROFILES,
        )
