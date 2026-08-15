from pathlib import Path

from bonesinfra.services.linux import runtime, shared
from bonesinfra.services.linux.application import deploy_static

from . import custom

TEMPLATES = Path(__file__).parent / "templates"
SHARED_DIRECTORIES = ()


def deploy(ctx):
    runtime.setup(ctx)
    shared.ensure_directories(ctx, ctx.paths_dict, SHARED_DIRECTORIES)
    deploy_static(
        ctx,
        static_root="dist",
        nginx_template=TEMPLATES / "nginx/static-site-nginx.conf.j2",
        placeholder_template=TEMPLATES / "nginx/index.html.j2",
    )
    custom.deploy(ctx)
    runtime.start_services(ctx)
