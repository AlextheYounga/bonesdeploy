from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.common import logs, paths as common_paths
from bonesinfra.pyinfra.operations import mkdir, render
from bonesinfra.services.languages import PHP
from bonesinfra.services.linux import systemd as service
from bonesinfra.services.linux.apparmor import app as apparmor
from bonesinfra.services.linux.nginx import site as nginx_site


class Framework:
    uses_tcp: bool = False

    def deploy(self, ctx):
        raise NotImplementedError

    def manifest_artifacts(self, _ctx) -> list[tuple[str, str, str, str]]:
        """Return framework artifact specs as (name, path, kind, owner)."""
        return []

    def manifest_services(self, _ctx) -> list[tuple[str, str, str]]:
        return []

    def manifest_mode(self, _ctx) -> str:
        return "none"


class StaticFramework(Framework):
    static_root: str

    def manifest_artifacts(self, ctx) -> list[tuple[str, str, str, str]]:
        static_root = f"{ctx.paths.placeholder_release}/{self.static_root}"
        current_root = f"{ctx.paths.current}/{self.static_root}"
        return [
            ("static placeholder web root", static_root, "directory", "framework"),
            ("static placeholder index", f"{static_root}/index.html", "file", "framework"),
            ("current static web root", current_root, "directory", "framework"),
        ]

    def manifest_mode(self, _ctx) -> str:
        return "static"

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

    def writable_paths(self, _ctx, _paths) -> list:
        return []

    def apparmor_exec_paths(self, _ctx, _paths) -> list:
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

    def manifest_artifacts(self, ctx) -> list[tuple[str, str, str, str]]:
        paths = ctx.paths
        artifacts = []
        if self.static_root and ctx.runtime.data.get("is_static", True):
            static_root = f"{paths.placeholder_release}/{self.static_root}"
            current_root = f"{paths.current}/{self.static_root}"
            artifacts.append(("static placeholder web root", static_root, "directory", "framework"))
            artifacts.append(("static placeholder index", f"{static_root}/index.html", "file", "framework"))
            artifacts.append(("current static web root", current_root, "directory", "framework"))
        else:
            artifacts.extend(
                [
                    ("application AppArmor profile", paths.apparmor_profile(self.service_name), "file", "framework"),
                    ("application systemd service", paths.systemd_service(self.service_name), "file", "framework"),
                    (
                        "application systemd requirement",
                        paths.systemd_service_requirement(self.service_name),
                        "link",
                        "framework",
                    ),
                    (
                        "application runtime directory",
                        paths.runtime_service_dir(self.service_name),
                        "directory",
                        "framework",
                    ),
                    (
                        "application runtime socket",
                        paths.runtime_service_socket(self.service_name),
                        "file",
                        "framework",
                    ),
                    ("application log directory", paths.site_log_dir, "directory", "framework"),
                ]
            )
        return artifacts

    def manifest_services(self, ctx) -> list[tuple[str, str, str]]:
        if self.static_root and ctx.runtime.data.get("is_static", True):
            return []
        return [("application service", f"{ctx.app.project_name}-{self.service_name}.service", "framework")]

    def manifest_mode(self, ctx) -> str:
        if self.static_root and ctx.runtime.data.get("is_static", True):
            return "static"
        return "server"

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

    def manifest_artifacts(self, ctx) -> list[tuple[str, str, str, str]]:
        version = str(ctx.runtime.data.get(PHP.config_key, PHP.default_version))
        project = ctx.app.project_name
        return [
            ("PHP-FPM pool configuration", f"/etc/php/{version}/fpm/pool.d/{project}.conf", "file", "framework"),
            ("PHP-FPM socket", f"/run/php/php{version}-fpm-{project}.sock", "file", "framework"),
            ("PHP log directory", ctx.paths.site_log_dir, "directory", "framework"),
            ("current PHP web root", ctx.paths.current_web_root, "directory", "framework"),
        ]

    def manifest_mode(self, _ctx) -> str:
        return "php"

    def deploy(self, ctx):
        paths = ctx.paths_dict

        PHP.install(ctx)
        php_fpm_socket_path = PHP.configure_fpm_pool(ctx, paths=paths)

        nginx_site.render_php_fpm(
            ctx,
            paths=paths,
            template_src=ASSETS_DIR / self.nginx_template,
            php_fpm_socket_path=php_fpm_socket_path,
        )
