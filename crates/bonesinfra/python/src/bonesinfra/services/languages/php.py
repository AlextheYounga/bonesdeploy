import re
from tempfile import NamedTemporaryFile

from pyinfra import host
from pyinfra.facts.server import LinuxDistribution
from pyinfra.operations import apt, server

from bonesinfra.services.languages.base import LanguageRuntime

PHP_SURY_KEYRING_URL = "https://packages.sury.org/debsuryorg-archive-keyring.deb"
PHP_SURY_KEYRING_DEST = "/usr/share/keyrings/deb.sury.org-php.gpg"
PHP_SURY_PREREQUISITES = ["apt-transport-https", "ca-certificates", "curl", "lsb-release"]


class PHPRuntime(LanguageRuntime):
    config_key = "php_version"
    default_version = "8.5"
    version_pattern = re.compile(r"^[0-9]+\.[0-9]+$")

    def install_version(self, _ctx) -> str:
        self._add_apt_source()
        packages = [
            f"php{self.version}",
            f"php{self.version}-cli",
            f"php{self.version}-fpm",
            f"php{self.version}-bcmath",
            f"php{self.version}-curl",
            f"php{self.version}-gd",
            f"php{self.version}-intl",
            f"php{self.version}-mbstring",
            f"php{self.version}-mysql",
            f"php{self.version}-sqlite3",
            f"php{self.version}-xml",
            f"php{self.version}-zip",
            "composer",
        ]
        apt.packages(
            name=f"Install PHP {self.version} runtime packages",
            packages=packages,
            present=True,
            update=True,
            _sudo=True,
        )
        return f"/usr/bin/php{self.version}"

    def _add_apt_source(self) -> None:
        apt.packages(
            name="Install PHP repo prerequisites",
            packages=PHP_SURY_PREREQUISITES,
            present=True,
            update=True,
            _sudo=True,
        )

        with NamedTemporaryFile(delete=False, suffix=".deb") as file:
            keyring_path = file.name

        server.shell(
            name="Download PHP repo keyring package",
            commands=[f"curl -sSLo {keyring_path} {PHP_SURY_KEYRING_URL}"],
            _sudo=True,
        )
        apt.deb(name="Install PHP repo keyring package", src=keyring_path, _sudo=True)
        server.shell(
            name="Remove stale PHP apt source file",
            commands=["rm -f /etc/apt/sources.list.d/php.list"],
            _sudo=True,
        )

        deb = host.get_fact(LinuxDistribution)
        release_meta = deb.get("release_meta", {}) if deb else {}
        codename = (
            release_meta.get("VERSION_CODENAME")
            or release_meta.get("CODENAME")
            or release_meta.get("DISTRIB_CODENAME")
            or "noble"
        )
        apt.repo(
            name="Add PHP apt repository",
            src=f"deb [signed-by={PHP_SURY_KEYRING_DEST}] https://packages.sury.org/php {codename} main",
            filename="php",
            _sudo=True,
        )


PHP = PHPRuntime()
