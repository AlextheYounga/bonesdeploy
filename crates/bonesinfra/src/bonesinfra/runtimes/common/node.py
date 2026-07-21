from pyinfra.operations import apt

NODE_PACKAGES = ["nodejs", "npm"]


def install_packages():
    apt.packages(
        name="Install Node.js runtime packages",
        packages=NODE_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )
