from shlex import quote

from pyinfra.operations import server

from bonesinfra.domain.context import DEPLOY_USER
from bonesinfra.domain.paths import ASSETS_DIR
from bonesinfra.infra.deploy_helpers import render


def install(paths):
    render(
        "Install BonesDeploy sudoers drop-in",
        ASSETS_DIR / "sudoers/bonesdeploy.j2",
        paths["sudoers_path"],
        user="root",
        group="root",
        mode="0440",
        deploy_user=DEPLOY_USER,
        bonesremote_path=paths["bonesremote_global_link"],
    )

    sudoers_path = quote(paths["sudoers_path"])
    server.shell(
        name="Validate BonesDeploy sudoers drop-in",
        commands=[f"visudo -c -f {sudoers_path} >/dev/null || {{ rm -f {sudoers_path}; exit 1; }}"],
        _sudo=True,
    )
