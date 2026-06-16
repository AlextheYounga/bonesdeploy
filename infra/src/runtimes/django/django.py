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


def deploy():
    from pyinfra.operations import apt

    apt.packages(
        name="Install Python repo prerequisites",
        packages=PYTHON_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )
