"""BonesDeploy's shared production mise installation policy."""

from dataclasses import dataclass
from shlex import quote

from pyinfra.operations import server

from bonesinfra.config.paths import SCRIPTS_DIR
from bonesinfra.services.languages.base import LanguageRuntime

MISE_VERSION = "2026.10.0"
MISE_BINARY_URL = "https://github.com/jdx/mise/releases/download/v2026.10.0/mise-v2026.10.0-linux-x64"
MISE_BINARY_SHA256 = "57ced973f968b8fbab07aa8e32bd7077d4a357e200a22356d98963c723c6de0a"
MISE_BINARY = "/usr/local/bin/mise"

MISE_DATA_DIR = "/var/lib/bonesdeploy/mise"
MISE_RUNTIME_STORE = f"{MISE_DATA_DIR}/installs"
MISE_CACHE_DIR = "/var/cache/bonesdeploy/mise"
MISE_CONFIG_DIR = "/etc/bonesdeploy/mise"
MISE_CONFIG_FILE = f"{MISE_CONFIG_DIR}/config.toml"


@dataclass(frozen=True)
class MisePolicy:
    """The operator-owned environment shared by all managed runtime installers."""

    version: str = MISE_VERSION
    binary: str = MISE_BINARY
    binary_url: str = MISE_BINARY_URL
    binary_sha256: str = MISE_BINARY_SHA256
    data_dir: str = MISE_DATA_DIR
    runtime_store: str = MISE_RUNTIME_STORE
    cache_dir: str = MISE_CACHE_DIR
    config_dir: str = MISE_CONFIG_DIR
    config_file: str = MISE_CONFIG_FILE

    def environment(self) -> dict[str, str]:
        return {
            "MISE_SYSTEM_DATA_DIR": self.data_dir,
            "MISE_SYSTEM_INSTALLS_DIR": self.runtime_store,
            "MISE_CACHE_DIR": self.cache_dir,
            "MISE_SYSTEM_CONFIG_FILE": self.config_file,
            "MISE_GLOBAL_CONFIG_ROOT": self.config_dir,
            "MISE_GLOBAL_CONFIG_FILE": self.config_file,
            "MISE_CONFIG_FILE": self.config_file,
            "MISE_ALL_COMPILE": "false",
            "MISE_NODE_COMPILE": "false",
            "MISE_PYTHON_COMPILE": "false",
            "MISE_RUBY_COMPILE": "false",
            "MISE_REGISTRY_FLOATING": "false",
            "MISE_AUTO_UPDATE": "false",
            "MISE_NO_HOOKS": "true",
            "MISE_SAFE": "true",
            "MISE_OVERRIDE_CONFIG_FILENAMES": ".bonesdeploy-mise-disabled",
            "MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES": "none",
            "MISE_SLSA": "true",
            "MISE_GITHUB_ATTESTATIONS": "true",
        }


@dataclass(frozen=True)
class MiseRuntimeAppArmorAccess:
    """The exact files AppArmor must mediate for a managed runtime."""

    executable: str
    root: str


class MiseRuntime(LanguageRuntime):
    """Install an exact runtime in the shared mise store and expose its site link."""

    tool: str
    executable_name: str
    version_output_prefix: str
    runtime_path_key: str

    def install_version(self, ctx) -> str:
        link_path = getattr(ctx.paths, self.runtime_path_key)
        server.script(
            name=f"Install {self.tool} {self.version} with mise",
            src=str(SCRIPTS_DIR / "install-mise-runtime.sh"),
            args=(self.tool, self.version, link_path),
            _env={
                **PRODUCTION_MISE.environment(),
                "MISE_BINARY": PRODUCTION_MISE.binary,
                "MISE_BINARY_URL": PRODUCTION_MISE.binary_url,
                "MISE_BINARY_SHA256": PRODUCTION_MISE.binary_sha256,
                "MISE_VERSION": PRODUCTION_MISE.version,
                "MISE_DATA_DIR": PRODUCTION_MISE.data_dir,
                "MISE_RUNTIME_STORE": PRODUCTION_MISE.runtime_store,
                "MISE_CACHE_DIR": PRODUCTION_MISE.cache_dir,
                "MISE_CONFIG_FILE": PRODUCTION_MISE.config_file,
            },
            _sudo=True,
        )
        executable = f"{link_path}/bin/{self.executable_name}"
        server.shell(
            name=f"Verify {self.tool} {self.version} as the runtime user",
            commands=[
                f"{quote(executable)} --version | grep -Eq "
                f"{quote(rf'^{self.version_output_prefix}{self.version}( |$)')}"
            ],
            _sudo=True,
            _sudo_user=ctx.runtime.runtime_user,
            _chdir="/",
        )
        return executable

    def apparmor_access(self) -> MiseRuntimeAppArmorAccess:
        if not self.version:
            raise RuntimeError("install the runtime before requesting AppArmor access")
        root = f"{PRODUCTION_MISE.runtime_store}/{self.tool}/{self.version}"
        return MiseRuntimeAppArmorAccess(
            executable=f"{root}/bin/{self.executable_name}",
            root=root,
        )


PRODUCTION_MISE = MisePolicy()
