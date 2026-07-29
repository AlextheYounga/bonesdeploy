from bonesinfra.config.paths import SCRIPTS_DIR
from pyinfra.operations import server

NODE_ROOT = "/opt/bonesdeploy/node"


def install(ctx) -> str:
    """Install the exact Node version selected in bones.toml and return its binary."""
    version = str(ctx.runtime.data.get("node_version", "24.18.0"))
    if not _is_exact_version(version):
        raise ValueError(f"node_version must be an exact X.Y.Z version, got {version!r}")

    server.script(
        name=f"Install Node.js v{version}",
        src=str(SCRIPTS_DIR / "install-node.sh"),
        args=(version,),
        _sudo=True,
    )
    return f"{NODE_ROOT}/v{version}/bin/node"


def _is_exact_version(version: str) -> bool:
    parts = version.split(".")
    return len(parts) == 3 and all(part.isdigit() for part in parts)
