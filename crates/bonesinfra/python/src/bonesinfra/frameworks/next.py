from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.base import ServerFramework
from bonesinfra.frameworks.common import validation
from bonesinfra.pyinfra.operations import mkdir, render
from bonesinfra.services.languages import NODE


class NextFramework(ServerFramework):
    service_name = "next"
    runtime_label = "Next.js app server"
    uses_tcp = True
    default_port = 3100
    static_root = "out"

    def manifest_artifacts(self, ctx):
        artifacts = super().manifest_artifacts(ctx)
        if ctx.runtime.data.get("is_static", True):
            return artifacts
        placeholder = ctx.paths.placeholder_release
        return [
            *artifacts,
            ("Next.js placeholder standalone directory", f"{placeholder}/.next/standalone", "directory", "framework"),
            ("Next.js placeholder standalone server", f"{placeholder}/.next/standalone/server.js", "file", "framework"),
        ]

    def install_packages(self, ctx):
        self.node_binary = NODE.install(ctx)

    def apparmor_exec_paths(self, _ctx, _paths):
        return [self.node_binary]

    def apparmor_network(self):
        return "network inet stream,"

    def exec_command(self, ctx, _paths):
        port = ctx.runtime.data.get("internal_port", self.default_port)
        return (
            f"/usr/bin/env NODE_ENV=production PORT={port} HOSTNAME=127.0.0.1 "
            f"{self.node_binary} .next/standalone/server.js"
        )

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
            ASSETS_DIR / "next/placeholder-server.js.j2",
            f"{server_dir}/server.js",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
            **template_data(ctx, paths=paths),
        )


NEXT_FRAMEWORK = NextFramework()
