from bonesinfra.cli.commands.helpers import packages
from bonesinfra.cli.commands.helpers.packages import HELPER_APT_PACKAGES, neovim, rainfrog, starship


def deploy_helpers(ctx):
    del ctx
    packages.install_helper_apt_packages(HELPER_APT_PACKAGES)
    packages.install_debian_command_aliases()
    starship.install()
    neovim.install()
    rainfrog.install()
