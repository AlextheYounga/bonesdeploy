import re

from pyinfra.operations import server

from bonesinfra.config.context import DEFAULT_NODE_VERSION, DeployContext
from bonesinfra.config.paths import SCRIPTS_DIR
from bonesinfra.services.languages.base import LanguageRuntime

NODE_ROOT = "/opt/bonesdeploy/node"


class NodeRuntime(LanguageRuntime):
    config_key = "node_version"
    default_version = DEFAULT_NODE_VERSION
    version_pattern = re.compile(r"^[0-9]+\.[0-9]+\.[0-9]+$")

    def configured_version(self, ctx: DeployContext) -> str:
        return ctx.runtime.node_version

    def install_version(self, _ctx) -> str:
        server.script(
            name=f"Install Node.js v{self.version}",
            src=str(SCRIPTS_DIR / "install-node.sh"),
            args=(self.version,),
            _sudo=True,
        )
        return f"{NODE_ROOT}/v{self.version}/bin/node"


NODE = NodeRuntime()
