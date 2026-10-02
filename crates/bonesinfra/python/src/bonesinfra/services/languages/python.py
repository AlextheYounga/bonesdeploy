import re

from pyinfra.operations import apt

from bonesinfra.services.languages.base import LanguageRuntime

PYTHON_PACKAGES = [
    "build-essential",
    "ca-certificates",
    "libffi-dev",
    "libpq-dev",
    "libssl-dev",
    "pkg-config",
    "python3",
    "python3-dev",
    "python3-pip",
    "python3-venv",
]


class PythonRuntime(LanguageRuntime):
    config_key = "python_version"
    default_version = "3.14"
    version_pattern = re.compile(r"^[0-9]+\.[0-9]+$")

    def install_version(self, _ctx) -> str:
        apt.packages(
            name="Install distribution Python and native package build dependencies",
            packages=PYTHON_PACKAGES,
            present=True,
            update=True,
            _sudo=True,
        )
        return "/usr/bin/python3"


PYTHON = PythonRuntime()
