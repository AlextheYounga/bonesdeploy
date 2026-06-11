from pyinfra import host
from pyinfra.operations import apt, server

LARAVEL_PHP_VERSION = "8.3"

LARAVEL_PHP_SURY_PREREQUISITE_PACKAGES = [
    "apt-transport-https",
    "ca-certificates",
    "curl",
    "lsb-release",
]

LARAVEL_PHP_SURY_KEYRING_PACKAGE_URL = "https://packages.sury.org/debsuryorg-archive-keyring.deb"
LARAVEL_PHP_SURY_KEYRING_PACKAGE_PATH = "/tmp/debsuryorg-archive-keyring.deb"
LARAVEL_PHP_SURY_KEYRING_PATH = "/usr/share/keyrings/deb.sury.org-php.gpg"

apt.packages(
    name="Install PHP repo prerequisites",
    packages=LARAVEL_PHP_SURY_PREREQUISITE_PACKAGES,
    present=True,
    update=True,
    _sudo=True,
)

server.shell(
    name="Download PHP repo keyring package",
    commands=[
        f"curl -sSLo {LARAVEL_PHP_SURY_KEYRING_PACKAGE_PATH} {LARAVEL_PHP_SURY_KEYRING_PACKAGE_URL}"
    ],
    _sudo=True,
)

apt.deb(
    name="Install PHP repo keyring package",
    src=LARAVEL_PHP_SURY_KEYRING_PACKAGE_PATH,
    _sudo=True,
)

deb = host.get_fact(server.LinuxDistribution)
codename = deb.get("release_meta", {}).get("DISTRIB_CODENAME", "noble") if deb else "noble"

apt.repo(
    name="Add Laravel PHP apt repository",
    src=f"deb [signed-by={LARAVEL_PHP_SURY_KEYRING_PATH}] https://packages.sury.org/php {codename} main",
    filename="php",
    _sudo=True,
)
