from pyinfra.operations import server, systemd

from bonesinfra.config.context import template_data
from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.pyinfra.operations import render


def setup(ctx, paths, nginx_apparmor_network="network unix stream,"):
    systemd.service(
        name="Ensure apparmor service is enabled and started",
        service="apparmor",
        enabled=True,
        running=True,
        _sudo=True,
    )
    server.shell(
        name="Verify apparmor kernel enabled",
        commands=[f"cat {paths['apparmor_enabled_param']}"],
        _sudo=True,
    )

    profile_name = f"bonesdeploy-{ctx.app.project_name}-nginx"
    profile_path = f"/etc/apparmor.d/{profile_name}"

    render(
        "Deploy per-project nginx AppArmor profile",
        ASSETS_DIR / "apparmor/project-nginx-profile.j2",
        profile_path,
        apparmor_profile_name=profile_name,
        nginx_apparmor_network=nginx_apparmor_network,
        **template_data(ctx, paths=paths),
    )
    server.shell(
        name="Load updated nginx AppArmor profile",
        commands=[f"apparmor_parser -r {profile_path}"],
        _sudo=True,
    )
    server.shell(
        name="Ensure nginx profile is in enforce mode",
        commands=[f"aa-enforce {profile_path}"],
        _sudo=True,
    )
