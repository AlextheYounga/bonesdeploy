from shlex import quote

from pyinfra.operations import server

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import TEMPLATES_DIR
from bonesinfra.pyinfra.operations import render
from bonesinfra.services.languages import RUBY
from bonesinfra.services.linux import application, runtime, shared, validation

TEMPLATES = TEMPLATES_DIR / "frameworks/rails"
SHARED_DIRECTORIES = ("tmp", "log", "storage")
BUNDLER_PATH = "vendor/bundle"
BUNDLER_SCRIPT = f"{BUNDLER_PATH}/bundler/bin/bundle"


def bundled_bundler(paths):
    return f"{paths['current']}/{BUNDLER_SCRIPT}"


def bundled_bundler_command(ruby_binary, bundle_script, command):
    return (
        f"BUNDLE_DISABLE_VERSION_CHECK=true BUNDLE_PATH={BUNDLER_PATH} "
        f"{quote(ruby_binary)} {quote(bundle_script)} {command}"
    )


def apparmor_exec_paths(paths, ruby_binary):
    bundle_root = f"{paths['releases']}/*/{BUNDLER_PATH}"
    return [
        "/usr/bin/env",
        ruby_binary,
        f"{bundle_root}/bundler/bin/bundle",
        f"{bundle_root}/ruby/*/bin/puma",
    ]


def deploy(ctx):
    def provision(current_ctx):
        shared.ensure_directories(current_ctx, current_ctx.paths_dict, SHARED_DIRECTORIES)

        def seed_placeholder(current_ctx, paths, ruby_binary):
            placeholder = paths["placeholder_release"]
            packaged_bundler = f"{placeholder}/{BUNDLER_SCRIPT}"
            bundler_version_command = f"{quote(ruby_binary)} -S bundle --version | awk '{{ print $3 }}'"
            render(
                "Seed placeholder Gemfile",
                TEMPLATES / "rails/placeholder-Gemfile.j2",
                f"{placeholder}/Gemfile",
                user="root",
                group=current_ctx.runtime.runtime_group,
                mode="0640",
                **template_data(current_ctx, paths=paths),
            )
            server.shell(
                name="Install placeholder gems",
                commands=[
                    f"cd {quote(placeholder)} && bundler_version=$({bundler_version_command}) "
                    f'&& {quote(ruby_binary)} -S gem install bundler --version "$bundler_version" '
                    f"--install-dir {BUNDLER_PATH}/bundler --bindir {BUNDLER_PATH}/bundler/bin --no-document "
                    f"&& BUNDLE_PATH={BUNDLER_PATH} {quote(ruby_binary)} {quote(packaged_bundler)} install"
                ],
                _sudo=True,
            )
            render(
                "Seed placeholder Rack config",
                TEMPLATES / "rails/placeholder-config.ru.j2",
                f"{placeholder}/config.ru",
                user="root",
                group=current_ctx.runtime.runtime_group,
                mode="0640",
                **template_data(current_ctx, paths=paths),
            )

        def install(current_ctx):
            return RUBY.install(current_ctx)

        def command(current_ctx, paths, ruby_binary):
            environment = current_ctx.runtime.data.get("rails_env", "production")
            socket = f"{paths['runtime_socket_dir']}/puma/puma.sock"
            return (
                f"/usr/bin/env RAILS_ENV={environment} "
                f"{bundled_bundler_command(ruby_binary, bundled_bundler(paths), f'exec puma -e {environment} -b unix://{socket}')}"
            )

        def validate(current_ctx, paths, ruby_binary):
            validation.run_as_runtime_user(
                current_ctx,
                "Validate Puma availability as runtime user",
                f"cd {quote(paths['current'])} && "
                f"{bundled_bundler_command(ruby_binary, bundled_bundler(paths), 'exec puma --help >/dev/null')}",
            )

        application.deploy_server(
            current_ctx,
            name="puma",
            runtime_label="Puma",
            nginx_template=TEMPLATES / "nginx/app-site-nginx.conf.j2",
            apparmor_template=TEMPLATES / "app-profile.j2",
            install=install,
            seed_placeholder=seed_placeholder,
            validate=validate,
            command=command,
            exec_paths=lambda _ctx, paths, ruby: apparmor_exec_paths(paths, ruby),
            apparmor_runtime_access=RUBY.apparmor_access,
            writable_paths=lambda _ctx, paths: [
                f"{paths['shared']}/tmp",  # noqa: S108 - Rails owns this shared application path.
                f"{paths['shared']}/log",
                f"{paths['shared']}/storage",
            ],
        )

    runtime.orchestrate(ctx, provision)
