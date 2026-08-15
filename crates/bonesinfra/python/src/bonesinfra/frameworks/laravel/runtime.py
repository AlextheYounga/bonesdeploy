from pathlib import Path

from bonesinfra.config.context import template_data
from bonesinfra.pyinfra.operations import render
from bonesinfra.services.languages import PHP
from bonesinfra.services.linux import runtime, shared, systemd
from bonesinfra.services.linux.nginx import site

from . import custom, docker

TEMPLATES = Path(__file__).parent / "templates"
SHARED_DIRECTORIES = ("storage", "storage/framework/views", "cache", "uploads")


def deploy(ctx):
    runtime.setup(ctx)
    paths = ctx.paths_dict
    shared.ensure_directories(ctx, paths, SHARED_DIRECTORIES)
    if ctx.runtime.backend == "docker":
        docker.deploy(ctx)
        custom.deploy(ctx)
        runtime.start_services(ctx)
        return
    php_executable = PHP.install(ctx)
    socket = PHP.configure_fpm_pool(ctx, paths=paths)
    site.render_php_fpm(
        ctx,
        paths=paths,
        template_src=TEMPLATES / "nginx/laravel-site-nginx.conf.j2",
        php_fpm_socket_path=socket,
    )
    if ctx.runtime.data.get("install_queue_worker", False):
        render(
            "Deploy Laravel queue worker service",
            TEMPLATES / "queue-worker.service.j2",
            ctx.paths.systemd_service("worker"),
            **template_data(ctx, paths=paths, php_executable=php_executable),
        )
        systemd.register_service(ctx, paths=paths, name="worker")
        systemd.enable_and_start(ctx, "worker")
    custom.deploy(ctx)
    runtime.start_services(ctx)
