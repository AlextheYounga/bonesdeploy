from pyinfra.operations import server

from bonesinfra.config.paths import BONESDEPLOY_REPO, SCRIPTS_DIR


def install():
    server.script_template(
        name="Install bonesremote binary",
        src=str(SCRIPTS_DIR / "install-bonesremote.sh.j2"),
        repo=BONESDEPLOY_REPO,
        _sudo=True,
    )
