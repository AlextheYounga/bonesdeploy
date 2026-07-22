from pathlib import Path

from pyinfra.operations import files

from bonesinfra.domain.context import template_data
from bonesinfra.runtimes.common import validation


def _ensure_runtime_socket_dir(ctx, paths):
    # 0711: system nginx (www-data) must traverse to reach per-site sockets.
    files.directory(
        name="Ensure runtime socket directory exists before nginx validation",
        path=paths["runtime_socket_dir"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
        _sudo=True,
    )
    files.directory(
        name="Ensure nginx runtime directory exists before nginx validation",
        path=paths["runtime_nginx_dir"],
        user=ctx.runtime.runtime_user,
        group=ctx.runtime.runtime_group,
        mode="0711",
        _sudo=True,
    )


def render_proxy(ctx, *, paths, socket_path=None, port=None):
    here = Path(__file__).parent
    app_proxy_target = f"http://unix:{socket_path}:" if socket_path else f"http://127.0.0.1:{port}"
    files.template(
        name="Deploy per-site app nginx config",
        src=str(here / "assets/app-site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        app_proxy_target=app_proxy_target,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    _ensure_runtime_socket_dir(ctx, paths)
    validation.run_as_runtime_user(
        ctx,
        "Validate nginx configuration with app config",
        f"nginx -t -c {paths['site_nginx_config']}",
    )


def render_static(ctx, *, paths, root="dist"):
    here = Path(__file__).parent
    files.template(
        name="Deploy per-site static nginx config",
        src=str(here / "assets/static-site-nginx.conf.j2"),
        dest=paths["site_nginx_config"],
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        static_root=root,
        **template_data(ctx, paths=paths),
        _sudo=True,
    )
    _ensure_runtime_socket_dir(ctx, paths)
    validation.run_as_runtime_user(
        ctx,
        "Validate nginx configuration with static config",
        f"nginx -t -c {paths['site_nginx_config']}",
    )
