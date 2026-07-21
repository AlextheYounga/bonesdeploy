from pyinfra.operations import server

from bonesinfra.domain.paths import ASSETS_DIR
from bonesinfra.infra.deploy_helpers import render

DISABLE_ALGIF_PATH = "/etc/modprobe.d/disable-algif.conf"


def configure():
    render(
        "Disable the vulnerable algif_aead module",
        ASSETS_DIR / "modprobe/disable-algif.conf.j2",
        DISABLE_ALGIF_PATH,
        user="root",
        group="root",
        mode="0644",
    )

    server.shell(
        name="Unload the vulnerable algif_aead module",
        commands=["if grep -q '^algif_aead ' /proc/modules; then rmmod algif_aead; fi"],
        _sudo=True,
    )
