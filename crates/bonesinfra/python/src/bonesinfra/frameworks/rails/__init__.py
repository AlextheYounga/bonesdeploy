from shlex import quote

from pyinfra.operations import server

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.base import ServerFramework
from bonesinfra.frameworks.common import validation
from bonesinfra.frameworks.rails import ruby_packages
from bonesinfra.pyinfra.operations import render


class RailsFramework(ServerFramework):
    service_name = "puma"
    runtime_label = "Puma"

    def install_packages(self, ctx):
        ruby_packages.install_packages()

    def apparmor_exec_paths(self, ctx, paths):
        return ["/usr/bin/ruby*", "/usr/bin/bundle*"]

    def writable_paths(self, ctx, paths):
        return [
            f"{paths['shared']}/tmp",  # noqa: S108
            f"{paths['shared']}/log",
            f"{paths['shared']}/storage",
        ]

    def exec_command(self, ctx, paths):
        rails_env = ctx.runtime.data.get("rails_env", "production")
        socket = self.socket_path(paths)
        return f"/usr/bin/env RAILS_ENV={rails_env} bundle exec puma -e {rails_env} -b unix://{socket}"

    def validate(self, ctx, paths):
        validation.run_as_runtime_user(
            ctx,
            "Validate Puma availability as runtime user",
            f"cd {quote(paths['current'])} && bundle exec puma --help >/dev/null",
        )

    def seed_placeholder(self, ctx, paths):
        placeholder = paths["placeholder_release"]
        render(
            "Seed placeholder Gemfile",
            ASSETS_DIR / "rails/placeholder-Gemfile.j2",
            f"{placeholder}/Gemfile",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0640",
            **template_data(ctx, paths=paths),
        )
        server.shell(
            name="Install placeholder gems",
            commands=[f"cd {quote(placeholder)} && bundle install"],
            _sudo=True,
        )
        render(
            "Seed placeholder Rack config",
            ASSETS_DIR / "rails/placeholder-config.ru.j2",
            f"{placeholder}/config.ru",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0640",
            **template_data(ctx, paths=paths),
        )


FRAMEWORK = RailsFramework()
