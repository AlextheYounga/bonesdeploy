from pathlib import Path

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import mkdir, render
from bonesinfra.runtimes.common import nginx, service

VUE_STATIC_ROOT = "dist"


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    static_web_root = f"{paths['placeholder_release']}/{VUE_STATIC_ROOT}"
    mkdir(
        name="Ensure Vue static placeholder output directory exists",
        path=static_web_root,
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0750",
    )
    render(
        "Seed Vue static placeholder index page",
        Path(__file__).parents[2] / "assets/nginx/index.html.j2",
        f"{static_web_root}/index.html",
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        **template_data(ctx, paths=paths),
    )
    nginx.render_static(ctx, paths=paths, root=VUE_STATIC_ROOT)
