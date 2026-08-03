import re

from pyinfra.operations import apt

from bonesinfra.services.languages.base import LanguageRuntime


class RubyRuntime(LanguageRuntime):
    config_key = "ruby_version"
    default_version = "3.3"
    version_pattern = re.compile(r"^[0-9]+\.[0-9]+$")

    def install_version(self, _ctx) -> str:
        apt.packages(
            name=f"Install Ruby {self.version} runtime packages",
            packages=[
                f"ruby{self.version}",
                f"ruby{self.version}-dev",
                "ruby-bundler",
                "libffi-dev",
                "libpq-dev",
                "libyaml-dev",
                "shared-mime-info",
                "zlib1g-dev",
            ],
            present=True,
            update=True,
            _sudo=True,
        )
        return f"/usr/bin/ruby{self.version}"


RUBY = RubyRuntime()
