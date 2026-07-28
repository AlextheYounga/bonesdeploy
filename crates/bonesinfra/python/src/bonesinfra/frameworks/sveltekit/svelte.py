from pathlib import Path
from shlex import quote

from pyinfra.operations import server

from bonesinfra.config.context import template_data
from bonesinfra.frameworks.base import ServerFramework
from bonesinfra.frameworks.common import node, validation
from bonesinfra.pyinfra.operations import render


class SvelteKitFramework(ServerFramework):
    service_name = "sveltekit"
    runtime_label = "SvelteKit app server"

    def install_packages(self, ctx):
        node.install_packages()

    def apparmor_exec_paths(self, ctx, paths):
        return ["/usr/bin/node"]

    def exec_command(self, ctx, paths):
        socket = self.socket_path(paths)
        origin = f"https://{ctx.app.dns.domain}" if ctx.app.dns.domain else "https://localhost"
        return (
            f"/usr/bin/env --chdir={paths['current']} NODE_ENV=production SOCKET_PATH={socket} "
            f"ORIGIN={origin} node --env-file=.env build"
        )

    def validate(self, ctx, paths):
        validation.run_as_runtime_user(
            ctx,
            "Validate SvelteKit build entrypoint exists as runtime user",
            f"test -e {paths['current']}/build",
        )

    def seed_placeholder(self, ctx, paths):
        render(
            "Seed placeholder SvelteKit build entrypoint",
            Path(__file__).parent / "assets/placeholder-index.js.j2",
            f"{paths['placeholder_release']}/build",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
            **template_data(ctx, paths=paths),
        )
        server.shell(
            name="Seed blank .env for SvelteKit placeholder",
            commands=[f"touch {quote(paths['placeholder_release'])}/.env"],
            _sudo=True,
        )


FRAMEWORK = SvelteKitFramework()
