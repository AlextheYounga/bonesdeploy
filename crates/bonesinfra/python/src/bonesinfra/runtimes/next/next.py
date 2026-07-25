from pathlib import Path

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import mkdir, render
from bonesinfra.runtimes.common import apparmor, logs, nginx, node, paths as common_paths, service, validation

STATIC_ROOT = "out"
USES_TCP = True


def deploy(ctx):
    is_static = ctx.runtime.data.get("is_static", True)
    paths = service.runtime_paths(ctx)

    if is_static:
        common_paths.ensure_runtime_dirs(ctx)
        static_web_root = f"{paths['placeholder_release']}/{STATIC_ROOT}"
        mkdir(
            name="Ensure Next static placeholder output directory exists",
            path=static_web_root,
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0750",
        )
        render(
            "Seed Next static placeholder index page",
            Path(__file__).parents[2] / "assets/nginx/index.html.j2",
            f"{static_web_root}/index.html",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0640",
            **template_data(ctx, paths=paths),
        )
        nginx.render_static(ctx, paths=paths, root=STATIC_ROOT)
        return

    port = ctx.runtime.data.get("internal_port", 3100)
    node.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="next",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[],
        apparmor_network="network inet stream,",
    )
    _seed_placeholder_server(ctx, paths)
    validation.run_as_runtime_user(
        ctx,
        "Validate Next.js standalone server exists as runtime user",
        f"test -f {paths['current']}/.next/standalone/server.js",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="next",
        runtime_label="Next.js app server",
        runtime_exec=(
            f"/usr/bin/env NODE_ENV=production PORT={port} HOSTNAME=127.0.0.1 node .next/standalone/server.js"
        ),
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=[],
        runtime_address_families="AF_UNIX AF_INET",
    )
    nginx.render_proxy(ctx, paths=paths, port=port)
    service.enable_and_start(ctx, "next", apparmor_profile_name=apparmor_profile_name)


def _seed_placeholder_server(ctx, paths):
    """Seed a minimal Node HTTP server into the placeholder release so the
    app service can start before any real release is deployed.

    ponytail: the placeholder server only serves the placeholder index page;
    the real Next.js standalone server replaces it when current flips to a
    real release. bonesremote service restart only restarts
    <project>-nginx.service, not <project>-next.service, so the app service
    keeps running the placeholder after deploy until manually restarted or
    bonesdeploy remote runtime is re-run.
    """
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
