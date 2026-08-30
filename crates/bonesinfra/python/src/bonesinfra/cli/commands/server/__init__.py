from bonesinfra.cli.commands.server import (
    bonesremote,
    disable_algif_aead,
    packages,
    sudoers,
    users,
)
from bonesinfra.cli.commands.server.packages import BASE_SYSTEM_PACKAGES, SUPPLEMENTARY_PACKAGES
from bonesinfra.config.context import ServerContext
from bonesinfra.config.paths import (
    BONESREMOTE_CONFIG_DIR,
    BONESREMOTE_LOCK_ROOT,
    BONESREMOTE_SITE_ROOT,
    BONESREMOTE_STATE_ROOT,
    DEPLOYMENT_SNAPSHOT_ROOT,
)
from bonesinfra.pyinfra.operations import mkdir
from bonesinfra.services.linux import apparmor, etckeeper, fail2ban, firewall, image_store, unattended_upgrades


def deploy_server_setup(ctx: ServerContext, bonesremote_version: str):
    packages.install_system(BASE_SYSTEM_PACKAGES + SUPPLEMENTARY_PACKAGES)
    etckeeper.initialize()
    apparmor.ensure_service()
    disable_algif_aead.configure()
    image_store.ensure_shared_store()
    image_store.seed_base_image()
    firewall.configure(ctx)
    fail2ban.configure(ctx)
    unattended_upgrades.configure()
    users.ensure_deploy_user(ctx)
    _ensure_bonesremote_roots()
    bonesremote.install(bonesremote_version)
    sudoers.install()
    etckeeper.commit_changes("BonesInfra server setup")


def _ensure_bonesremote_roots():
    mkdir(
        name="Ensure BonesRemote configuration root exists",
        path=BONESREMOTE_CONFIG_DIR,
        user="root",
        group="root",
        mode="0700",
    )
    mkdir(
        name="Ensure BonesRemote sites root exists",
        path=BONESREMOTE_SITE_ROOT,
        user="root",
        group="root",
        mode="0700",
    )
    mkdir(
        name="Ensure BonesRemote deployment state root exists",
        path=BONESREMOTE_STATE_ROOT,
        user="git",
        group="git",
        mode="0700",
    )
    mkdir(
        name="Ensure deployment snapshot root exists",
        path=DEPLOYMENT_SNAPSHOT_ROOT,
        user="root",
        group="git",
        mode="0750",
    )
    mkdir(
        name="Ensure BonesRemote deployment lock root exists",
        path=BONESREMOTE_LOCK_ROOT,
        user="root",
        group="git",
        mode="0770",
    )
