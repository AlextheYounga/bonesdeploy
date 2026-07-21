from pyinfra.operations import files, server, systemd

from bonesinfra.domain.context import template_data
from bonesinfra.runtimes.common import logs

PHP_FPM_SOCKET_PARENT = "/run/php"


def socket_path(project, php_version):
    return f"{PHP_FPM_SOCKET_PARENT}/php{php_version}-fpm-{project}.sock"


def pool_config_path(project, php_version):
    return f"/etc/php/{php_version}/fpm/pool.d/{project}.conf"


def ensure_log_dir(ctx):
    logs.ensure(ctx)


def render_pool(ctx, *, here, paths, php_version):
    project = ctx.app.project_name
    files.template(
        name="Deploy Laravel PHP-FPM pool config",
        src=str(here / "assets/php/php-fpm-pool.conf.j2"),
        dest=pool_config_path(project, php_version),
        user="root",
        group="root",
        mode="0644",
        laravel_php_fpm_pool_name=project,
        laravel_php_fpm_socket_path=socket_path(project, php_version),
        **template_data(ctx, paths=paths),
        _sudo=True,
    )


def validate_php_fpm(php_version):
    server.shell(
        name="Validate PHP-FPM configuration",
        commands=[f"php-fpm{php_version} --test"],
        _sudo=True,
    )


def reload_php_fpm(php_version):
    # ponytail: reload may fail on a fresh install where the unit is inactive,
    # so we restart (always valid) rather than `systemctl reload`. Upgrade path
    # is a dedicated "ensure active then reload" sequence once first-boot is
    # handled out of band.
    systemd.service(
        name="Enable and restart PHP-FPM service",
        service=f"php{php_version}-fpm",
        enabled=True,
        running=True,
        restarted=True,
        _sudo=True,
    )
