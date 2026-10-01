import re

from pyinfra.operations import apt

from bonesinfra.services.languages.base import LanguageRuntime

RUBY_RELEASES = {"3.2": "3.2.8", "3.3": "3.3.8", "3.4": "3.4.8"}
RUBY_PACKAGES = [
    "build-essential",
    "default-libmysqlclient-dev",
    "git",
    "libpq-dev",
    "libsqlite3-dev",
    "pkg-config",
    "ruby",
    "ruby-bundler",
    "ruby-dev",
]


class RubyRuntime(LanguageRuntime):
    config_key = "ruby_version"
    default_version = "3.3.8"
    version_pattern = re.compile(r"^3\.[234](?:\.8)?$")

    def install(self, ctx) -> str:
        version = str(ctx.runtime.data.get(self.config_key, self.default_version))
        if not self.version_pattern.fullmatch(version):
            raise ValueError(f"{self.config_key} has an invalid version: {version!r}")
        self.version = RUBY_RELEASES.get(version, version)
        self.executable = self.install_version(ctx)
        return self.executable

    def install_version(self, _ctx) -> str:
        apt.packages(
            name="Install distribution Ruby and native gem build dependencies",
            packages=RUBY_PACKAGES,
            present=True,
            update=True,
            _sudo=True,
        )
        return "/usr/bin/ruby"


RUBY = RubyRuntime()
