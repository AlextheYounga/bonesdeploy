PYTHON_PACKAGES = [
    "python3",
    "python3-dev",
    "python3-pip",
    "python3-venv",
    "python3-gunicorn",
    "libpq-dev",
]


def questions():
    return [
        {
            "key": "python_version",
            "type": "choice",
            "label": "Python version",
            "choices": ["3.11", "3.12", "3.13"],
            "default": "3.12",
        },
        {
            "key": "install_postgres",
            "type": "bool",
            "label": "Install PostgreSQL client libraries?",
            "default": False,
        },
    ]


def defaults():
    return {
        "python_version": "3.12",
        "install_postgres": False,
        "shared_paths": [],
    }


def shared_paths(ctx):
    return defaults()["shared_paths"]


def apply(ctx):
    raise NotImplementedError("django apply is not migrated yet")


def deploy():
    from pyinfra.operations import apt

    apt.packages(
        name="Install Python repo prerequisites",
        packages=PYTHON_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )
