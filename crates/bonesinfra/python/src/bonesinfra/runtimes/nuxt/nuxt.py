from pathlib import Path

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import mkdir, render
from bonesinfra.runtimes.common import apparmor, logs, nginx, node, paths as common_paths, service

STATIC_ROOT = ".output/public"


def deploy(ctx):
    is_static = ctx.runtime.data.get("is_static", True)
    paths = service.runtime_paths(ctx)

    if is_static:
        common_paths.ensure_runtime_dirs(ctx)
        static_web_root = f"{paths['placeholder_release']}/{STATIC_ROOT}"
        mkdir(
            name="Ensure Nuxt static placeholder output directory exists",
            path=static_web_root,
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
        )
        render(
            "Seed Nuxt static placeholder index page",
            Path(__file__).parents[2] / "assets/nginx/index.html.j2",
            f"{static_web_root}/index.html",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0640",
            **template_data(ctx, paths=paths),
        )
        nginx.render_static(ctx, paths=paths, root=STATIC_ROOT)
        return

    socket_path = f"{paths['runtime_socket_dir']}/nuxt/nuxt.sock"
    node.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="nuxt",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[],
    )
    _seed_placeholder_server(ctx, paths)
    service.render_app_service(
        ctx,
        paths=paths,
        name="nuxt",
        runtime_label="Nuxt app server",
        runtime_exec=(
            f"/usr/bin/env NODE_ENV=production NITRO_UNIX_SOCKET={socket_path} node .output/server/index.mjs"
        ),
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=[],
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "nuxt", apparmor_profile_name=apparmor_profile_name)


def _seed_placeholder_server(ctx, paths):
    """Seed a minimal Node HTTP server into the placeholder release so the
    app service can start before any real release is deployed.

    ponytail: bonesremote service restart only restarts
    <project>-nginx.service, not <project>-nuxt.service.
    """
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
