
from pyinfra import host
from pyinfra.operations import apt, files, server, systemd

SETUP_LABEL = "Rails"
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
    name="Install PHP repo prerequisites",
    packages=RUBY_PREREQUISITE_PACKAGES,
    present=True,
    update=True,
    _sudo=True,
)