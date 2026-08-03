import re

from pyinfra.operations import apt

from bonesinfra.services.languages.base import LanguageRuntime


class PythonRuntime(LanguageRuntime):
    config_key = "python_version"
    default_version = "3.14"
    version_pattern = re.compile(r"^[0-9]+\.[0-9]+$")

    def install_version(self, _ctx) -> str:
        apt.packages(
            name=f"Install Python {self.version} runtime packages",
            packages=[f"python{self.version}", f"python{self.version}-dev", f"python{self.version}-venv", "libpq-dev"],
            present=True,
            update=True,
            _sudo=True,
        )
        return f"/usr/bin/python{self.version}"


PYTHON = PythonRuntime()
