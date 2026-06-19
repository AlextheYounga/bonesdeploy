#!/usr/bin/env bash

set -Eeuo pipefail

# Node is a build dependency: install it once per project from a pinned .nvmrc.
# No nvm, no alias resolution — .nvmrc must name an exact version like "20.11.0" or "v20.11.0".
# Aliases ("lts/iron", "node", "latest") would force us back to fetching nodejs.org/dist/index.json
# on every deploy, which is the fragile network leg nvm trips over.

if [ ! -f "./.nvmrc" ]; then
  exit 0
fi

: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
NODE_DIR="$PROJECT_ROOT/build/node"
NODE_BIN="$NODE_DIR/bin/node"

version=$(grep -E '^[[:space:]]*v?[0-9]+\.[0-9]+\.[0-9]+' .nvmrc | head -1 | tr -d 'v' | xargs)
if [ -z "$version" ]; then
  echo "[bonesdeploy] .nvmrc must pin an exact version (e.g. 20.11.0), not an alias like 'lts/iron' or 'node'." >&2
  exit 1
fi

if [ -x "$NODE_BIN" ] && "$NODE_BIN" --version | grep -qx "v$version"; then
  exit 0
fi

arch=$(uname -m)
case "$arch" in
  x86_64)  arch="x64" ;;
  aarch64) arch="arm64" ;;
  *) echo "[bonesdeploy] Unsupported arch: $arch" >&2; exit 1 ;;
esac

url="https://nodejs.org/dist/v${version}/node-v${version}-linux-${arch}.tar.xz"
tmp=$(mktemp -d)
trap 'rm -rf "$tmp"' EXIT
echo "[bonesdeploy] Installing Node v${version} from nodejs.org..."
curl -fsSL "$url" | tar -xJ -C "$tmp"
rm -rf "$NODE_DIR"
mv "$tmp/node-v${version}-linux-${arch}" "$NODE_DIR"

export PATH="$NODE_DIR/bin:$PATH"
corepack enable 2>/dev/null || true