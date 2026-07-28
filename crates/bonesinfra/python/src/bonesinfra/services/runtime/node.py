from shlex import quote

from pyinfra.operations import server

NODE_ROOT = "/opt/bonesdeploy/node"


def install(ctx) -> str:
    """Install the exact Node version selected in bones.toml and return its binary."""
    version = str(ctx.runtime.data.get("node_version", "24.18.0"))
    if not _is_exact_version(version):
        raise ValueError(f"node_version must be an exact X.Y.Z version, got {version!r}")

    install_dir = f"{NODE_ROOT}/v{version}"
    command = (
        f"if [ ! -x {quote(install_dir + '/bin/node')} ]; then "
        'case "$(uname -m)" in x86_64) arch=x64;; aarch64) arch=arm64;; *) exit 1;; esac; '
        f"archive=node-v{version}-linux-$arch.tar.xz; tmp=$(mktemp -d); trap 'rm -rf -- $tmp' EXIT; "
        f"base=https://nodejs.org/dist/v{version}; curl -fsSL --retry 3 -o $tmp/$archive $base/$archive; "
        f"curl -fsSL --retry 3 -o $tmp/SHASUMS256.txt $base/SHASUMS256.txt; "
        f"cd $tmp; grep '  '$archive'$' SHASUMS256.txt | sha256sum --check --status -; "
        f"install -d -m 0755 {quote(NODE_ROOT)}; tar --no-same-owner -xJf $tmp/$archive -C $tmp; "
        f"mv $tmp/node-v{version}-linux-$arch {quote(install_dir)}; "
        f"fi"
    )
    server.shell(name=f"Install Node.js v{version}", commands=[command], _sudo=True)
    return f"{install_dir}/bin/node"


def _is_exact_version(version: str) -> bool:
    parts = version.split(".")
    _semver_parts = 3
    return len(parts) == _semver_parts and all(part.isdigit() for part in parts)
