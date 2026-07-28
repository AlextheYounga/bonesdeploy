from pathlib import Path

from bonesinfra.config.context import template_data
from bonesinfra.frameworks.base import ServerFramework
from bonesinfra.frameworks.common import node
from bonesinfra.infra.operations import mkdir, render


class NuxtFramework(ServerFramework):
    service_name = "nuxt"
    runtime_label = "Nuxt app server"
    static_root = ".output/public"

    def install_packages(self, ctx):
        node.install_packages()

    def apparmor_exec_paths(self, ctx, paths):
        return ["/usr/bin/node"]

    def exec_command(self, ctx, paths):
        socket = self.socket_path(paths)
        return f"/usr/bin/env NODE_ENV=production NITRO_UNIX_SOCKET={socket} node .output/server/index.mjs"

    def seed_placeholder(self, ctx, paths):
        server_dir = f"{paths['placeholder_release']}/.output/server"
        mkdir(
            name="Ensure placeholder .output/server directory exists",
            path=server_dir,
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
        )
        render(
            "Seed placeholder Nuxt nitro server",
            Path(__file__).parent / "assets/placeholder-server.mjs.j2",
            f"{server_dir}/index.mjs",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
            **template_data(ctx, paths=paths),
        )


FRAMEWORK = NuxtFramework()
