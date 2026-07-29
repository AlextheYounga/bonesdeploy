from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.base import ServerFramework
from bonesinfra.pyinfra.operations import mkdir, render
from bonesinfra.services.runtime import node


class NuxtFramework(ServerFramework):
    service_name = "nuxt"
    runtime_label = "Nuxt app server"
    static_root = ".output/public"

    def install_packages(self, ctx):
        self.node_binary = node.install(ctx)

    def apparmor_exec_paths(self, _ctx, _paths):
        return [self.node_binary]

    def exec_command(self, _ctx, paths):
        socket = self.socket_path(paths)
        return (
            f"/usr/bin/env NODE_ENV=production NITRO_UNIX_SOCKET={socket} {self.node_binary} .output/server/index.mjs"
        )

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
            ASSETS_DIR / "nuxt/placeholder-server.mjs.j2",
            f"{server_dir}/index.mjs",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
            **template_data(ctx, paths=paths),
        )


NUXT_FRAMEWORK = NuxtFramework()
