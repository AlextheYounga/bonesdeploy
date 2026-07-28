from pathlib import Path

from bonesinfra.config.context import template_data
from bonesinfra.frameworks.base import ServerFramework
from bonesinfra.frameworks.common import node, validation
from bonesinfra.infra.operations import mkdir, render


class NextFramework(ServerFramework):
    service_name = "next"
    runtime_label = "Next.js app server"
    uses_tcp = True
    default_port = 3100
    static_root = "out"

    def install_packages(self, ctx):
        node.install_packages()

    def apparmor_exec_paths(self, ctx, paths):
        return ["/usr/bin/node"]

    def apparmor_network(self):
        return "network inet stream,"

    def exec_command(self, ctx, paths):
        port = ctx.runtime.data.get("internal_port", self.default_port)
        return f"/usr/bin/env NODE_ENV=production PORT={port} HOSTNAME=127.0.0.1 node .next/standalone/server.js"

    def validate(self, ctx, paths):
        validation.run_as_runtime_user(
            ctx,
            "Validate Next.js standalone server exists as runtime user",
            f"test -f {paths['current']}/.next/standalone/server.js",
        )

    def seed_placeholder(self, ctx, paths):
        server_dir = f"{paths['placeholder_release']}/.next/standalone"
        mkdir(
            name="Ensure placeholder .next/standalone directory exists",
            path=server_dir,
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
        )
        render(
            "Seed placeholder Next.js standalone server",
            Path(__file__).parent / "assets/placeholder-server.js.j2",
            f"{server_dir}/server.js",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
            **template_data(ctx, paths=paths),
        )


FRAMEWORK = NextFramework()
