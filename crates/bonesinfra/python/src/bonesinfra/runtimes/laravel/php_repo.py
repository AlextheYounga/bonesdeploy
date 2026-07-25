from tempfile import NamedTemporaryFile

PHP_SURY_KEYRING_URL = "https://packages.sury.org/debsuryorg-archive-keyring.deb"
PHP_SURY_KEYRING_DEST = "/usr/share/keyrings/deb.sury.org-php.gpg"
PHP_SURY_PREREQUISITES = [
    "apt-transport-https",
    "ca-certificates",
    "curl",
    "lsb-release",
]


def _resolve_codename():
    from pyinfra import host
    from pyinfra.facts.server import LinuxDistribution

    deb = host.get_fact(LinuxDistribution)
    release_meta = deb.get("release_meta", {}) if deb else {}
    return (
        release_meta.get("VERSION_CODENAME")
        or release_meta.get("CODENAME")
        or release_meta.get("DISTRIB_CODENAME")
        or "noble"
    )


def add_php_apt_source():
    from pyinfra.operations import apt, server

    apt.packages(
        name="Install PHP repo prerequisites",
        packages=PHP_SURY_PREREQUISITES,
        present=True,
        update=True,
        _sudo=True,
    )

    with NamedTemporaryFile(delete=False, suffix=".deb") as f:
        keyring_path = f.name

    server.shell(
        name="Download PHP repo keyring package",
        commands=[f"curl -sSLo {keyring_path} {PHP_SURY_KEYRING_URL}"],
        _sudo=True,
    )

    apt.deb(
        name="Install PHP repo keyring package",
        src=keyring_path,
        _sudo=True,
    )

    server.shell(
        name="Remove stale PHP apt source file",
        commands=["rm -f /etc/apt/sources.list.d/php.list"],
        _sudo=True,
    )

    codename = _resolve_codename()
    apt.repo(
        name="Add Laravel PHP apt repository",
        src=f"deb [signed-by={PHP_SURY_KEYRING_DEST}] https://packages.sury.org/php {codename} main",
        filename="php",
        _sudo=True,
    )
