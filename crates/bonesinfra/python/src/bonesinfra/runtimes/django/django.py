from pathlib import Path
from shlex import quote

from pyinfra.operations import server

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import mkdir, render
from bonesinfra.runtimes.common import apparmor, logs, nginx, paths as common_paths, service, validation
from bonesinfra.runtimes.django import python_packages


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    socket_path = f"{paths['runtime_socket_dir']}/gunicorn/gunicorn.sock"
    wsgi_module = ctx.runtime.data.get("wsgi_module", "config.wsgi:application")
    writable = [f"{paths['shared']}/staticfiles", f"{paths['shared']}/media"]
    gunicorn_bin = f"{paths['current']}/.venv/bin/gunicorn"
    python_packages.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="gunicorn",
        apparmor_exec_paths=[gunicorn_bin],
        apparmor_writable_paths=writable,
    )
    _seed_placeholder_server(ctx, paths)
    validation.run_as_runtime_user(
        ctx,
        "Validate Gunicorn configuration as runtime user",
        f"{gunicorn_bin} --check-config {wsgi_module}",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="gunicorn",
        runtime_label="Gunicorn",
        runtime_exec=f"{gunicorn_bin} {wsgi_module} --bind unix:{socket_path}",
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=writable,
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "gunicorn", apparmor_profile_name=apparmor_profile_name)


def _seed_placeholder_server(ctx, paths):
    """Create a venv with gunicorn and a minimal WSGI app in the placeholder
    release so the app service can start before any real release is deployed.

    ponytail: bonesremote service restart only restarts
    <project>-nginx.service, not <project>-gunicorn.service.
    """
    placeholder = paths["placeholder_release"]
    server.shell(
        name="Create placeholder venv with gunicorn",
        commands=[
            f"cd {quote(placeholder)} && python3 -m venv .venv && .venv/bin/pip install gunicorn",
        ],
        _sudo=True,
    )
    mkdir(
        name="Ensure placeholder config directory exists",
        path=f"{placeholder}/config",
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0750",
    )
    render(
        "Seed placeholder WSGI application",
        Path(__file__).parent / "assets/placeholder-wsgi.py.j2",
        f"{placeholder}/config/wsgi.py",
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        **template_data(ctx, paths=paths),
    )
