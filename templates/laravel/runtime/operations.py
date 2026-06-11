import os

from pyinfra.operations import files, server, systemd

here = os.path.dirname(__file__)

pool_config_path = f"/srv/conf/{data['project_name']}/php-fpm.conf"

files.directory(
    name="Ensure conf directory exists",
    path=data["paths"]["conf_root"],
    user="root",
    group=data["group"],
    mode="0750",
    _sudo=True,
)

files.directory(
    name="Ensure storage directories exist",
    path=f"{data['paths']['current']}/storage/logs",
    user=data["service_user"],
    group=data["group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework cache directory exists",
    path=f"{data['paths']['current']}/storage/framework/cache",
    user=data["service_user"],
    group=data["group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework sessions directory exists",
    path=f"{data['paths']['current']}/storage/framework/sessions",
    user=data["service_user"],
    group=data["group"],
    mode="0775",
    _sudo=True,
)

files.directory(
    name="Ensure storage framework views directory exists",
    path=f"{data['paths']['current']}/storage/framework/views",
    user=data["service_user"],
    group=data["group"],
    mode="0775",
    _sudo=True,
)

files.template(
    name="Deploy PHP-FPM pool config",
    src=os.path.join(here, "templates/php-fpm-pool.conf.j2"),
    dest=pool_config_path,
    user="root",
    group="root",
    mode="0644",
    laravel_php_fpm_pool_name=data["project_name"],
    laravel_php_fpm_socket_path=f"/run/{data['project_name']}/php-fpm.sock",
    project_root=data["project_root"],
    **data,
    _sudo=True,
)

files.template(
    name="Deploy PHP-FPM systemd service",
    src=os.path.join(here, "templates/site-php-fpm.service.j2"),
    dest=f"/etc/systemd/system/{data['project_name']}-php-fpm.service",
    user="root",
    group="root",
    mode="0644",
    laravel_php_fpm_pool_config_path=pool_config_path,
    laravel_php_version_resolved=data.get("laravel_php_version", "8.3"),
    apparmor_profile_name=f"bonesdeploy-{data['project_name']}-php-fpm",
    **data,
    _sudo=True,
)

files.template(
    name="Deploy PHP-FPM AppArmor profile",
    src=os.path.join(here, "templates/site-php-fpm-profile.j2"),
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
