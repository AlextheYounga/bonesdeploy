import os

from pyinfra import host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, files, server, systemd
from src.utils import unflatten


SETUP_LABEL = "Laravel"
LARAVEL_PHP_VERSION = "8.3"
SETUP_APT_EXTRAS = [
    f"php{LARAVEL_PHP_VERSION}",
    f"php{LARAVEL_PHP_VERSION}-cli",
    f"php{LARAVEL_PHP_VERSION}-fpm",
    f"php{LARAVEL_PHP_VERSION}-bcmath",
    f"php{LARAVEL_PHP_VERSION}-curl",
    f"php{LARAVEL_PHP_VERSION}-gd",
    f"php{LARAVEL_PHP_VERSION}-intl",
    f"php{LARAVEL_PHP_VERSION}-mbstring",
    f"php{LARAVEL_PHP_VERSION}-mysql",
    f"php{LARAVEL_PHP_VERSION}-sqlite3",
    f"php{LARAVEL_PHP_VERSION}-xml",
    f"php{LARAVEL_PHP_VERSION}-zip",
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
codename = deb.get("release_meta", {}).get("DISTRIB_CODENAME", "noble") if deb else "noble"

apt.repo(
    name="Add Laravel PHP apt repository",
    src=f"deb [signed-by={LARAVEL_PHP_SURY_KEYRING_PATH}] https://packages.sury.org/php {codename} main",
    filename="php",
    _sudo=True,
)

files.directory(
    name="Ensure conf directory exists",
    path=data["paths"]["conf_root"],
    user="root",
    group=data["service_group"],
    mode="0750",
    _sudo=True,
)

files.directory(
    name="Ensure storage directories exist",
    path=f"{data['paths']['current']}/storage/logs",
    user=data["service_user"],
    group=data["service_group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework cache directory exists",
    path=f"{data['paths']['current']}/storage/framework/cache",
    user=data["service_user"],
    group=data["service_group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework sessions directory exists",
    path=f"{data['paths']['current']}/storage/framework/sessions",
    user=data["service_user"],
    group=data["service_group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework views directory exists",
    path=f"{data['paths']['current']}/storage/framework/views",
    user=data["service_user"],
    group=data["service_group"],
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
    laravel_php_version_resolved=data.get("laravel_php_version", LARAVEL_PHP_VERSION),
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
    group=data["service_group"],
    mode="0640",
    laravel_php_fpm_socket_path=php_fpm_socket_path,
    **data,
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
