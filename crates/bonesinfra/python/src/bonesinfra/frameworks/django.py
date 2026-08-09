from shlex import quote

from pyinfra.operations import server

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.frameworks.base import ServerFramework
from bonesinfra.frameworks.common import validation
from bonesinfra.pyinfra.operations import mkdir, render
from bonesinfra.services.languages import PYTHON


class DjangoFramework(ServerFramework):
    service_name = "gunicorn"
    runtime_label = "Gunicorn"

    def manifest_artifacts(self, ctx):
        placeholder = ctx.paths.placeholder_release
        return [
            *super().manifest_artifacts(ctx),
            ("Django placeholder virtual environment", f"{placeholder}/.venv", "directory", "framework"),
            ("Django placeholder configuration", f"{placeholder}/config", "directory", "framework"),
            ("Django placeholder WSGI application", f"{placeholder}/config/wsgi.py", "file", "framework"),
        ]

    def install_packages(self, _ctx):
        self.python_binary = PYTHON.install(_ctx)

    def apparmor_exec_paths(self, _ctx, paths):
        return [f"{paths['current']}/.venv/bin/gunicorn"]

    def writable_paths(self, _ctx, paths):
        return [f"{paths['shared']}/media"]

    def exec_command(self, ctx, paths):
        gunicorn_bin = f"{paths['current']}/.venv/bin/gunicorn"
        wsgi_module = ctx.runtime.data.get("wsgi_module", "config.wsgi:application")
        return f"{gunicorn_bin} {wsgi_module} --bind unix:{self.socket_path(paths)}"

    def validate(self, ctx, paths):
        gunicorn_bin = f"{paths['current']}/.venv/bin/gunicorn"
        wsgi_module = ctx.runtime.data.get("wsgi_module", "config.wsgi:application")
        validation.run_as_runtime_user(
            ctx,
            "Validate Gunicorn configuration as runtime user",
            f"{gunicorn_bin} --check-config {wsgi_module}",
        )

    def seed_placeholder(self, ctx, paths):
        placeholder = paths["placeholder_release"]
        server.shell(
            name="Create placeholder venv with gunicorn",
            commands=[
                f"cd {quote(placeholder)} && {self.python_binary} -m venv .venv && .venv/bin/pip install gunicorn"
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
            ASSETS_DIR / "django/placeholder-wsgi.py.j2",
            f"{placeholder}/config/wsgi.py",
            user="root",
            group=ctx.runtime.runtime_group,
            mode="0640",
            **template_data(ctx, paths=paths),
        )


DJANGO_FRAMEWORK = DjangoFramework()
