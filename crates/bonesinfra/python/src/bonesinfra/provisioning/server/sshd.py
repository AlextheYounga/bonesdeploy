from shlex import quote

from pyinfra.operations import server

from bonesinfra.config.paths import ASSETS_DIR, ETC_SSH_SSHD_CONFIG_D
from bonesinfra.pyinfra.operations import render

ROOT_LOGIN_DROP_IN = f"{ETC_SSH_SSHD_CONFIG_D}/99-bonesdeploy-root-login.conf"


def disable_root_password_login():
    render(
        "Disable password login for root",
        ASSETS_DIR / "sshd/99-bonesdeploy-root-login.conf.j2",
        ROOT_LOGIN_DROP_IN,
    )

    drop_in = quote(ROOT_LOGIN_DROP_IN)
    server.shell(
        name="Validate sshd configuration",
        commands=[f"sshd -t || {{ rm -f {drop_in}; exit 1; }}"],
        _sudo=True,
    )
    server.shell(
        name="Reload ssh to apply root login hardening",
        commands=["systemctl reload ssh"],
        _sudo=True,
    )
