from pyinfra.operations import apt

PYTHON_PACKAGES = [
    "python3",
    "python3-dev",
    "python3-pip",
    "python3-venv",
    "libpq-dev",
]


def install_packages():
    apt.packages(
        name="Install Python runtime packages",
        packages=PYTHON_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )
