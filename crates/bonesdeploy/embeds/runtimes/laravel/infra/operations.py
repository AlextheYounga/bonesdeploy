import os

from pyinfra import host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, files, server, systemd
from src.utils import unflatten, load_runtime_config


SETUP_LABEL = "Laravel"


here = os.path.dirname(__file__)
data = unflatten(host.data.dict())
runtime = load_runtime_config(__file__)
php_version = runtime.get("php_version", "8.3")

SETUP_APT_EXTRAS = [
    f"php{php_version}",
    f"php{php_version}-cli",
    f"php{php_version}-fpm",
    f"php{php_version}-bcmath",
    f"php{php_version}-curl",
    f"php{php_version}-gd",
    f"php{php_version}-intl",
    f"php{php_version}-mbstring",
    f"php{php_version}-mysql",
    f"php{php_version}-sqlite3",
    f"php{php_version}-xml",
    f"php{php_version}-zip",
    "composer",
]
LARAVEL_PHP_SURY_PREREQUISITE_PACKAGES = [
    "apt-transport-https",
    "ca-certificates",
    "curl",
    "lsb-release",
]
LARAVEL_PHP_SURY_KEYRING_PACKAGE_URL = "https://packages.sury.org/debsuryorg-archive-keyring.deb"
LARAVEL_PHP_SURY_KEYRING_PACKAGE_PATH = "/tmp/debsuryorg-archive-keyring.deb"
LARAVEL_PHP_SURY_KEYRING_PATH = "/usr/share/keyrings/deb.sury.org-php.gpg"


here = os.path.dirname(__file__)
data = unflatten(host.data.dict())
pool_config_path = f"/srv/conf/{data['project_name']}/php-fpm.conf"
php_fpm_socket_path = data["paths"]["runtime_php_fpm_socket"]

apt.packages(
    name="Install PHP repo prerequisites",
    packages=LARAVEL_PHP_SURY_PREREQUISITE_PACKAGES,
    present=True,
    update=True,
    _sudo=True,
)

server.shell(
    name="Download PHP repo keyring package",
    commands=[
        f"curl -sSLo {LARAVEL_PHP_SURY_KEYRING_PACKAGE_PATH} {LARAVEL_PHP_SURY_KEYRING_PACKAGE_URL}"
    ],
    _sudo=True,
)

apt.deb(
    name="Install PHP repo keyring package",
    src=LARAVEL_PHP_SURY_KEYRING_PACKAGE_PATH,
    _sudo=True,
)

deb = host.get_fact(LinuxDistribution)
release_meta = deb.get("release_meta", {}) if deb else {}
codename = (
    release_meta.get("VERSION_CODENAME")
    or release_meta.get("CODENAME")
    or release_meta.get("DISTRIB_CODENAME")
    or "noble"
)

server.shell(
    name="Remove stale PHP apt source file",
    commands=["rm -f /etc/apt/sources.list.d/php.list"],
    _sudo=True,
)

apt.repo(
    name="Add Laravel PHP apt repository",
    src=f"deb [signed-by={LARAVEL_PHP_SURY_KEYRING_PATH}] https://packages.sury.org/php {codename} main",
    filename="php",
    _sudo=True,
)

apt.packages(
    name="Install Laravel PHP packages",
    packages=SETUP_APT_EXTRAS,
    present=True,
    update=True,
    _sudo=True,
)

files.directory(
    name="Ensure conf directory exists",
    path=data["paths"]["conf_root"],
    user="root",
    group=data["runtime_group"],
    mode="0750",
    _sudo=True,
)

files.directory(
    name="Ensure storage directories exist",
    path=f"{data['paths']['current']}/storage/logs",
    user=data["runtime_user"],
    group=data["runtime_group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework cache directory exists",
    path=f"{data['paths']['current']}/storage/framework/cache",
    user=data["runtime_user"],
    group=data["runtime_group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework sessions directory exists",
    path=f"{data['paths']['current']}/storage/framework/sessions",
    user=data["runtime_user"],
    group=data["runtime_group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework views directory exists",
    path=f"{data['paths']['current']}/storage/framework/views",
    user=data["runtime_user"],
    group=data["runtime_group"],
    mode="0775",
    _sudo=True,
)

files.template(
    name="Deploy PHP-FPM pool config",
    src=os.path.join(here, "assets/php/php-fpm-pool.conf.j2"),
    dest=pool_config_path,
    user="root",
    group="root",
    mode="0644",
    laravel_php_fpm_pool_name=data["project_name"],
    laravel_php_fpm_socket_path=php_fpm_socket_path,
    **data,
    _sudo=True,
)

files.template(
    name="Deploy PHP-FPM systemd service",
    src=os.path.join(here, "assets/php/site-php-fpm.service.j2"),
    dest=f"/etc/systemd/system/{data['project_name']}-php-fpm.service",
    user="root",
    group="root",
    mode="0644",
    laravel_php_fpm_pool_config_path=pool_config_path,
    laravel_php_version_resolved=php_version,
    apparmor_profile_name=f"bonesdeploy-{data['project_name']}-php-fpm",
    **data,
    _sudo=True,
)

files.template(
    name="Deploy PHP-FPM AppArmor profile",
    src=os.path.join(here, "assets/php/site-php-fpm-profile.j2"),
    dest=f"/etc/apparmor.d/bonesdeploy-{data['project_name']}-php-fpm",
    user="root",
    group="root",
    mode="0644",
    apparmor_profile_name=f"bonesdeploy-{data['project_name']}-php-fpm",
    **data,
    _sudo=True,
)

server.shell(
    name="Load PHP-FPM AppArmor profile",
    commands=[f"apparmor_parser -r -T -W /etc/apparmor.d/bonesdeploy-{data['project_name']}-php-fpm"],
    _sudo=True,
)

server.shell(
    name="Validate PHP-FPM configuration",
    commands=[f"/usr/sbin/php-fpm{php_version} --test --fpm-config {pool_config_path}"],
    _sudo=True,
)

systemd.service(
    name="Enable and start per-project PHP-FPM service",
    service=f"{data['project_name']}-php-fpm.service",
    enabled=True,
    running=True,
    daemon_reload=True,
    _sudo=True,
)

files.template(
    name="Deploy Laravel-specific per-site nginx config",
    src=os.path.join(here, "assets/nginx/laravel-site-nginx.conf.j2"),
    dest=data["paths"]["site_nginx_config"],
    user="root",
    group=data["runtime_group"],
    mode="0640",
    laravel_php_fpm_socket_path=php_fpm_socket_path,
    **data,
    _sudo=True,
)

files.directory(
    name="Ensure runtime socket directory exists before nginx validation",
    path=data["paths"]["runtime_socket_dir"],
    user=data["runtime_user"],
    group=data["runtime_group"],
    mode="0750",
    _sudo=True,
)

server.shell(
    name="Validate nginx configuration with Laravel config",
    commands=[f"nginx -t -c {data['paths']['site_nginx_config']} -g 'daemon off;'"],
    _sudo=True,
)

systemd.service(
    name="Restart per-site nginx with Laravel config",
    service=f"{data['project_name']}-nginx",
    restarted=True,
    _sudo=True,
)
