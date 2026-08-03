from pyinfra.operations import server

from bonesinfra.config.paths import ASSETS_DIR
from bonesinfra.pyinfra.operations import render


def render_profile(
    ctx,
    *,
    paths,
    runtime,
    apparmor_exec_paths,
    apparmor_writable_paths,
    apparmor_network="network unix stream,",
):
    profile_name = f"bonesdeploy-{ctx.app.project_name}-{runtime}"
    profile_path = f"/etc/apparmor.d/{profile_name}"

    render(
        f"Deploy {runtime} AppArmor profile",
        ASSETS_DIR / "apparmor/app-profile.j2",
        profile_path,
        apparmor_profile_name=profile_name,
        apparmor_runtime=runtime,
        apparmor_exec_paths=apparmor_exec_paths,
        apparmor_writable_paths=apparmor_writable_paths,
        apparmor_network=apparmor_network,
    )
    server.shell(
        name=f"Load {runtime} AppArmor profile",
        commands=[f"apparmor_parser -r -T -W {profile_path}"],
        _sudo=True,
    )
    server.shell(
        name=f"Enforce {runtime} AppArmor profile",
        commands=[f"aa-enforce {profile_name}"],
        _sudo=True,
    )
    return profile_name
