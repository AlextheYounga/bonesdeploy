from bonesinfra.cli.commands.setup import (
    bonesremote,
    directories,
    disable_algif_aead,
    image_store,
    packages,
    placeholder,
    sudoers,
    users,
)
from bonesinfra.cli.commands.setup.packages import BASE_SYSTEM_PACKAGES, SUPPLEMENTARY_PACKAGES
from bonesinfra.services.linux import fail2ban, firewall, unattended_upgrades


def deploy_setup(ctx, bonesremote_version: str = ""):
    paths = ctx.paths_dict
    all_pkgs = BASE_SYSTEM_PACKAGES + SUPPLEMENTARY_PACKAGES

    packages.install_system(all_pkgs)
    disable_algif_aead.configure()
    image_store.ensure_shared_store()
    image_store.seed_base_image()
    users.ensure_users_and_groups(ctx)
    directories.setup_repo_and_project(ctx, paths)
    placeholder.seed(ctx, paths)
    firewall.configure(ctx)
    fail2ban.configure(ctx)
    unattended_upgrades.configure()
    users.install_authorized_key(ctx)
    bonesremote.install(bonesremote_version)
    sudoers.install(paths)
