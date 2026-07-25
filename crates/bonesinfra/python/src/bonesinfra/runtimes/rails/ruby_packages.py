from pyinfra.operations import apt

RUBY_PACKAGES = [
    "ruby-full",
    "ruby-bundler",
    "libffi-dev",
    "libpq-dev",
    "libyaml-dev",
    "shared-mime-info",
    "zlib1g-dev",
]


def install_packages():
    apt.packages(
        name="Install Ruby runtime packages",
        packages=RUBY_PACKAGES,
        present=True,
        update=True,
        _sudo=True,
    )
