from types import ModuleType

from bonesinfra.deploys.setup import (
    bonesremote,
    directories,
    fail2ban,
    firewall,
    image_store,
    kernel_hardening,
    packages,
    placeholder,
    sudoers,
    unattended_upgrades,
    users,
)
from bonesinfra.deploys.setup.packages import BASE_SYSTEM_PACKAGES, SUPPLEMENTARY_PACKAGES
from bonesinfra.domain.custom import call_hook


def deploy_setup(ctx, custom: ModuleType | None = None):
    paths = ctx.paths_dict
    all_pkgs = BASE_SYSTEM_PACKAGES + SUPPLEMENTARY_PACKAGES

    packages.install_system(all_pkgs)
    kernel_hardening.configure()
    users.install_rust()
    image_store.ensure_shared_store()
    image_store.seed_base_image()
    users.ensure_users_and_groups(ctx)
    directories.setup_repo_and_project(ctx, paths)
    placeholder.seed(ctx, paths)
    firewall.configure(ctx)
    fail2ban.configure(ctx)
    unattended_upgrades.configure()
    users.install_authorized_key(ctx)
    bonesremote.install()
    sudoers.install(paths)

    call_hook(custom, "after_setup", ctx)
