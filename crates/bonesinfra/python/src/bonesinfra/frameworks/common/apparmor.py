from pathlib import Path

from pyinfra.operations import files, server

from bonesinfra.domain.context import template_data


def render_app_profile(  # noqa: PLR0913
    ctx,
    *,
    paths,
    runtime,
    apparmor_exec_paths,
    apparmor_writable_paths,
    apparmor_network="network unix stream,",
):
    here = Path(__file__).parent
    profile_name = f"bonesdeploy-{ctx.app.project_name}-{runtime}"
    profile_path = f"/etc/apparmor.d/{profile_name}"
    files.template(
        name=f"Deploy {runtime} AppArmor profile",
        src=str(here / "assets/app-profile.j2"),
        dest=profile_path,
        user="root",
        group="root",
        mode="0644",
        apparmor_profile_name=profile_name,
        apparmor_runtime=runtime,
        apparmor_exec_paths=apparmor_exec_paths,
        apparmor_writable_paths=apparmor_writable_paths,
        apparmor_network=apparmor_network,
        **template_data(ctx, paths=paths),
        _sudo=True,
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
