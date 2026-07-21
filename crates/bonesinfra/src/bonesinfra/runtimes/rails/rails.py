from pathlib import Path
from shlex import quote

from pyinfra.operations import server

from bonesinfra.domain.context import template_data
from bonesinfra.infra.deploy_helpers import render
from bonesinfra.runtimes.common import apparmor, logs, nginx, paths as common_paths, service, validation
from bonesinfra.runtimes.rails import ruby_packages


def deploy(ctx):
    paths = service.runtime_paths(ctx)
    socket_path = f"{paths['runtime_socket_dir']}/puma/puma.sock"
    runtime_write_paths = [
        f"{paths['shared']}/tmp",  # noqa: S108
        f"{paths['shared']}/log",
        f"{paths['shared']}/storage",
    ]
    rails_env = ctx.runtime.data.get("rails_env", "production")
    ruby_packages.install_packages()
    common_paths.ensure_runtime_dirs(ctx)
    logs.ensure(ctx)
    apparmor_profile_name = apparmor.render_app_profile(
        ctx,
        paths=paths,
        runtime="puma",
        apparmor_exec_paths=["/usr/bin/ruby*", "/usr/bin/bundle*"],
        apparmor_writable_paths=runtime_write_paths,
    )
    _seed_placeholder_server(ctx, paths)
    validation.run_as_runtime_user(
        ctx,
        "Validate Puma availability as runtime user",
        f"cd {quote(paths['current'])} && bundle exec puma --help >/dev/null",
    )
    service.render_app_service(
        ctx,
        paths=paths,
        name="puma",
        runtime_label="Puma",
        runtime_exec=f"/usr/bin/env RAILS_ENV={rails_env} bundle exec puma -e {rails_env} -b unix://{socket_path}",
        apparmor_profile_name=apparmor_profile_name,
        runtime_write_paths=runtime_write_paths,
    )
    nginx.render_proxy(ctx, paths=paths, socket_path=socket_path)
    service.enable_and_start(ctx, "puma", apparmor_profile_name=apparmor_profile_name)


def _seed_placeholder_server(ctx, paths):
    """Seed a minimal Gemfile + bundle install + Rack app into the placeholder
    release so the app service can start before any real release is deployed.

    ponytail: bonesremote service restart only restarts
    <project>-nginx.service, not <project>-puma.service.
    """
    placeholder = paths["placeholder_release"]
    render(
        "Seed placeholder Gemfile",
        Path(__file__).parent / "assets/placeholder-Gemfile.j2",
        f"{placeholder}/Gemfile",
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        **template_data(ctx, paths=paths),
    )
    server.shell(
        name="Install placeholder gems",
        commands=[f"cd {quote(placeholder)} && bundle install"],
        _sudo=True,
    )
    render(
        "Seed placeholder Rack config",
        Path(__file__).parent / "assets/placeholder-config.ru.j2",
        f"{placeholder}/config.ru",
        user="root",
        group=ctx.runtime.runtime_group,
        mode="0640",
        **template_data(ctx, paths=paths),
    )
