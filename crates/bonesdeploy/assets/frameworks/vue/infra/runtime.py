from pathlib import Path

from bonesinfra.services.linux import runtime
from bonesinfra.services.linux.application import deploy_static

from . import custom

TEMPLATES = Path(__file__).parent / "templates"


def deploy(ctx):
    runtime.setup(ctx)
    deploy_static(
        ctx,
        static_root="dist",
        nginx_template=TEMPLATES / "nginx/static-site-nginx.conf.j2",
        placeholder_template=TEMPLATES / "nginx/index.html.j2",
    )
    custom.deploy(ctx)
    runtime.start_services(ctx)
