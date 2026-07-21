from pyinfra.operations import files, server

from bonesinfra.domain.context import template_data
from bonesinfra.runtimes.common import php_fpm_pool


def setup(here, ctx, paths, php_version):
    runtime_group = ctx.runtime.runtime_group
    php_fpm_socket_path = php_fpm_pool.socket_path(ctx.app.project_name, php_version)

    files.template(
        name="Deploy Laravel-specific per-site nginx config",
        src=str(here / "assets/nginx/laravel-site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=runtime_group,
        mode="0640",
        laravel_php_fpm_socket_path=php_fpm_socket_path,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )

    server.shell(
        name="Validate nginx configuration with Laravel config",
        commands=[f"nginx -t -c {paths['site_nginx_config']}"],
        _sudo=True,
    )
