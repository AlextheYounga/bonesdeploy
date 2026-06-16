RUBY_PACKAGES = [
    "ruby-full",
    "ruby-bundler",
    "libffi-dev",
    "libpq-dev",
    "libyaml-dev",
    "shared-mime-info",
    "zlib1g-dev",
]


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


def deploy():
    from pyinfra.operations import apt

    apt.packages(
        name="Install Rails repo prerequisites",
        packages=RUBY_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )
