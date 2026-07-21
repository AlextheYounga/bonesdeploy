from types import ModuleType

from bonesinfra.deploys.helpers import neovim, packages, rainfrog, starship
from bonesinfra.deploys.helpers.packages import HELPER_APT_PACKAGES


def deploy_helpers(ctx, custom: ModuleType | None = None):
    del ctx, custom
    packages.install_helper_apt_packages(HELPER_APT_PACKAGES)
    packages.install_debian_command_aliases()
    starship.install()
    neovim.install()
    rainfrog.install()
