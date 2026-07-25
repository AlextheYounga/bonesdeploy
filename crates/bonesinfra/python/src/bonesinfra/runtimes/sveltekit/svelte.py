from pathlib import Path
from shlex import quote

from pyinfra.operations import server

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import render
from bonesinfra.runtimes.common import apparmor, logs, nginx, node, paths as common_paths, service, validation


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    socket_path = f"{paths['runtime_socket_dir']}/sveltekit/sveltekit.sock"
    origin = f"https://{ctx.app.dns.domain}" if ctx.app.dns.domain else "https://localhost"
    node.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="sveltekit",
        apparmor_exec_paths=["/usr/bin/node"],
        apparmor_writable_paths=[],
    )
    _seed_placeholder_server(ctx, paths)
    server.shell(
        name="Seed blank .env for SvelteKit placeholder",
        commands=[f"touch {quote(paths['placeholder_release'])}/.env"],
        _sudo=True,
    )
    validation.run_as_runtime_user(
        ctx,
        "Validate SvelteKit build entrypoint exists as runtime user",
        f"test -e {paths['current']}/build",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="sveltekit",
        runtime_label="SvelteKit app server",
        runtime_exec=(
            f"/usr/bin/env --chdir={paths['current']} NODE_ENV=production SOCKET_PATH={socket_path} "
            f"ORIGIN={origin} node --env-file=.env build"
        ),
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=[],
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "sveltekit", apparmor_profile_name=apparmor_profile_name)


def _seed_placeholder_server(ctx, paths):
    """Seed a minimal Node HTTP server into the placeholder release.

    ponytail: bonesremote service restart only restarts
    <project>-nginx.service, not <project>-sveltekit.service.
    """
    render(
        "Seed placeholder SvelteKit build entrypoint",
        Path(__file__).parent / "assets/placeholder-index.js.j2",
        f"{paths['placeholder_release']}/build",
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0750",
        **template_data(ctx, paths=paths),
    )
