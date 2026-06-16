SETUP_LABEL = "Rails"


def questions():
    return [
        {
            "key": "ruby_version",
            "type": "choice",
            "label": "Ruby version",
            "choices": ["3.2", "3.3", "3.4"],
            "default": "3.3",
        },
        {
            "key": "install_postgres",
            "type": "bool",
            "label": "Install PostgreSQL client libraries?",
            "default": False,
        },
        {
            "key": "install_redis",
            "type": "bool",
            "label": "Install Redis?",
            "default": False,
        },
    ]


def defaults():
    return {
        "ruby_version": "3.3",
        "install_postgres": False,
        "install_redis": False,
        "shared_paths": [],
    }


def shared_paths(ctx):
    return defaults()["shared_paths"]


def apply(ctx):
    raise NotImplementedError("rails apply is not migrated yet")


def deploy():
    from pyinfra import host
    from pyinfra.operations import apt, files, server, systemd

    RUBY_PREREQUISITE_PACKAGES = [
        "ruby-full",
        "ruby-bundler",
        "libffi-dev",
        "libpq-dev",
        "libyaml-dev",
        "shared-mime-info",
        "zlib1g-dev",
    ]

    apt.packages(
        name="Install Rails repo prerequisites",
        packages=RUBY_PREREQUISITE_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )
