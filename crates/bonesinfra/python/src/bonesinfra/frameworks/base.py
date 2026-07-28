from pyinfra.operations import files, server

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR, SCRIPTS_DIR
from bonesinfra.services.linux.apparmor import app as apparmor
from bonesinfra.frameworks.common import logs, php_fpm_pool
from bonesinfra.frameworks.common import paths as common_paths
from bonesinfra.frameworks.common import systemd as service
from bonesinfra.frameworks.common import validation
from bonesinfra.pyinfra.operations import mkdir, render
from bonesinfra.services.linux.nginx import site as nginx_site


class Framework:
    uses_tcp: bool = False

    def deploy(self, ctx):
        raise NotImplementedError


class StaticFramework(Framework):
    static_root: str

    def deploy(self, ctx):
        paths = service.runtime_paths(ctx)
        common_paths.ensure_runtime_dirs(ctx)
        self._seed_static_placeholder(ctx, paths)
        nginx_site.render_static(ctx, paths=paths, root=self.static_root)

    def _seed_static_placeholder(self, ctx, paths):
        static_web_root = f"{paths['placeholder_release']}/{self.static_root}"
        mkdir(
            name=f"Ensure {type(self).__name__} static placeholder directory exists",
            path=static_web_root,
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
        )
        render(
            f"Seed {type(self).__name__} static placeholder index page",
            ASSETS_DIR / "nginx/index.html.j2",
            f"{static_web_root}/index.html",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0640",
            **template_data(ctx, paths=paths),
        )


class ServerFramework(Framework):
    service_name: str
    runtime_label: str
    uses_tcp: bool = False
    default_port: int = 3000
    static_root: str | None = None

    def install_packages(self, ctx):
        pass

    def seed_placeholder(self, ctx, paths):
        raise NotImplementedError

    def validate(self, ctx, paths):
        pass

    def exec_command(self, ctx, paths) -> str:
        raise NotImplementedError

    def writable_paths(self, ctx, paths) -> list:
        return []

    def apparmor_exec_paths(self, ctx, paths) -> list:
        return []

    def apparmor_network(self) -> str | None:
        return None

    def socket_path(self, paths) -> str:
        return f"{paths['runtime_socket_dir']}/{self.service_name}/{self.service_name}.sock"

    def deploy(self, ctx):
        if self.static_root and ctx.runtime.data.get("is_static", True):
            self._deploy_as_static(ctx)
        else:
            self._deploy_as_server(ctx)

    def _deploy_as_static(self, ctx):
        paths = service.runtime_paths(ctx)
        common_paths.ensure_runtime_dirs(ctx)
        static_web_root = f"{paths['placeholder_release']}/{self.static_root}"
        mkdir(
            name=f"Ensure {type(self).__name__} static placeholder directory exists",
            path=static_web_root,
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
        )
        render(
            f"Seed {type(self).__name__} static placeholder index page",
            ASSETS_DIR / "nginx/index.html.j2",
            f"{static_web_root}/index.html",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0640",
            **template_data(ctx, paths=paths),
        )
        nginx_site.render_static(ctx, paths=paths, root=self.static_root)

    def _deploy_as_server(self, ctx):
        paths = service.runtime_paths(ctx)
        self.install_packages(ctx)
        common_paths.ensure_runtime_dirs(ctx)
        logs.ensure(ctx)

        apparmor_kwargs = {}
        if net := self.apparmor_network():
            apparmor_kwargs["apparmor_network"] = net

        profile_name = apparmor.render_profile(
            ctx,
            paths=paths,
            runtime=self.service_name,
            apparmor_exec_paths=self.apparmor_exec_paths(ctx, paths),
            apparmor_writable_paths=self.writable_paths(ctx, paths),
            **apparmor_kwargs,
        )
        self.seed_placeholder(ctx, paths)
        self.validate(ctx, paths)

        service.render_app_service(
            ctx,
            paths=paths,
            name=self.service_name,
            runtime_label=self.runtime_label,
            runtime_exec=self.exec_command(ctx, paths),
            apparmor_profile_name=profile_name,
            runtime_write_paths=self.writable_paths(ctx, paths),
            runtime_address_families="AF_UNIX AF_INET" if self.uses_tcp else "AF_UNIX",
        )

        if self.uses_tcp:
            port = ctx.runtime.data.get("internal_port", self.default_port)
            nginx_site.render_proxy(ctx, paths=paths, port=port)
        else:
            nginx_site.render_proxy(ctx, paths=paths, socket_path=self.socket_path(paths))

        service.enable_and_start(ctx, self.service_name, apparmor_profile_name=profile_name)


class PHPFramework(Framework):
    nginx_template: str

    def php_version(self, ctx) -> str:
        return ctx.runtime.data.get("php_version", "8.5")

    def install_packages(self, ctx):
        pass

    def deploy(self, ctx):
        php_version = self.php_version(ctx)
        paths = ctx.paths_dict
        project = ctx.app.project_name

        self.install_packages(ctx)
        php_fpm_pool.ensure_log_dir(ctx)
        server.script_template(
            name="Remove orphaned PHP-FPM pools from other PHP versions",
            src=str(SCRIPTS_DIR / "cleanup-orphaned-php-pools.sh.j2"),
            project=project,
            current_pool=php_fpm_pool.pool_config_path(project, php_version),
            _sudo=True,
        )
        php_fpm_pool.render_pool(ctx, paths=paths, php_version=php_version)
        php_fpm_pool.validate_php_fpm(php_version)
        php_fpm_pool.reload_php_fpm(php_version)

        nginx_site.render_php_fpm(
            ctx,
            paths=paths,
            template_src=ASSETS_DIR / self.nginx_template,
            php_fpm_socket_path=php_fpm_pool.socket_path(project, php_version),
        )
