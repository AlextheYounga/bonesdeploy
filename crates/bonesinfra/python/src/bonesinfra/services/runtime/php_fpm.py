from pyinfra.operations import files, server, systemd

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.common import logs
from bonesinfra.services.runtime.base import RuntimeService

PHP_FPM_SOCKET_PARENT = "/run/php"


class PHPFpmService(RuntimeService):
    def provision(self, _ctx):
        raise NotImplementedError("PHP-FPM is provisioned as part of a PHP framework deployment")

    @staticmethod
    def socket_path(project, php_version):
        return f"{PHP_FPM_SOCKET_PARENT}/php{php_version}-fpm-{project}.sock"

    @staticmethod
    def pool_config_path(project, php_version):
        return f"/etc/php/{php_version}/fpm/pool.d/{project}.conf"

    @staticmethod
    def ensure_log_dir(ctx):
        logs.ensure(ctx)

    def render_pool(self, ctx, *, paths, php_version):
        project = ctx.app.project_name
        files.template(
            name="Deploy PHP-FPM pool config",
            src=str(ASSETS_DIR / "php/php-fpm-pool.conf.j2"),
            dest=self.pool_config_path(project, php_version),
            user="root",
            group="root",
            mode="0644",
            php_fpm_pool_name=project,
            php_fpm_socket_path=self.socket_path(project, php_version),
            **template_data(ctx, paths=paths),
            _sudo=True,
        )

    @staticmethod
    def validate_php_fpm(php_version):
        server.shell(
            name="Validate PHP-FPM configuration",
            commands=[f"php-fpm{php_version} --test"],
            _sudo=True,
        )

    @staticmethod
    def reload_php_fpm(php_version):
        systemd.service(
            name="Enable and restart PHP-FPM service",
            service=f"php{php_version}-fpm",
            enabled=True,
            running=True,
            restarted=True,
            _sudo=True,
        )


PHP_FPM_SERVICE = PHPFpmService()
