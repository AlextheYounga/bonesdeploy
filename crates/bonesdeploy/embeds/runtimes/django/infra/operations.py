from pyinfra.operations import apt

SETUP_LABEL = "Django"
PYTHON_PREREQUISITE_PACKAGES = [
    "python3",
    "python3-dev",
    "python3-pip",
    "python3-venv",
    "python3-gunicorn",
    "libpq-dev",
]

apt.packages(
    name="Install Python repo prerequisites",
    packages=PYTHON_PREREQUISITE_PACKAGES,
    present=True,
    update=True,
    _sudo=True,
)