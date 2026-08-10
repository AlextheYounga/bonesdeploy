from pathlib import Path

from bonesinfra.services.languages import PHP
from bonesinfra.services.linux.nginx import site

from . import custom, docker

TEMPLATES = Path(__file__).parent / "templates"


def deploy(ctx):
    if ctx.runtime.backend == "docker":
        docker.deploy(ctx)
        custom.deploy(ctx)
        return
    paths = ctx.paths_dict
    PHP.install(ctx)
    socket = PHP.configure_fpm_pool(ctx, paths=paths)
    site.render_php_fpm(
        ctx,
        paths=paths,
        template_src=TEMPLATES / "nginx/laravel-site-nginx.conf.j2",
        php_fpm_socket_path=socket,
    )
    custom.deploy(ctx)
