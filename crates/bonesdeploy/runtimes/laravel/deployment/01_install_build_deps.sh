#!/usr/bin/env bash

set -Eeuo pipefail

trap 'status=$?; echo "[bonesdeploy] Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2; exit "$status"' ERR

[ -f artisan ] || { echo "[bonesdeploy] artisan not found; skipping Laravel build dependency install."; exit 0; }
[ -f package.json ] || { echo "[bonesdeploy] package.json not found; skipping Node install."; exit 0; }

: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"

NODE_DIR="$PROJECT_ROOT/build/node"
NODE_BIN="$NODE_DIR/bin/node"

command -v curl >/dev/null 2>&1 || { echo "[bonesdeploy] curl not found" >&2; exit 1; }
command -v tar >/dev/null 2>&1 || { echo "[bonesdeploy] tar not found" >&2; exit 1; }

read_node_version() {
  if [ -n "${BONES_NODE_VERSION:-}" ]; then
    printf '%s\n' "$BONES_NODE_VERSION"
    return
  fi

  if [ -f .node-version ]; then
    head -n 1 .node-version
    return
  fi

  if [ -f .nvmrc ]; then
    head -n 1 .nvmrc
    return
  fi

  if [ -f .tool-versions ]; then
    awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
    return
  fi

  if command -v php >/dev/null 2>&1; then
    php -r '
      $p = json_decode(file_get_contents("package.json"), true) ?: [];

      if (isset($p["volta"]["node"])) {
          echo $p["volta"]["node"];
          exit;
      }

      if (isset($p["engines"]["node"])) {
          echo $p["engines"]["node"];
          exit;
      }
    '
  fi
}

raw_version="$(read_node_version | head -n 1 | sed 's/#.*$//' | tr -d '\r' | xargs || true)"
version="${raw_version#v}"

if ! [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
  cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF
  exit 1
fi

ensure_corepack() {
  export PATH="$NODE_DIR/bin:$PATH"

  if ! command -v corepack >/dev/null 2>&1; then
    echo "[bonesdeploy] corepack not found in Node install; installing corepack..."
    npm install -g corepack@latest
  fi

  corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

if [ -x "$NODE_BIN" ] && "$NODE_BIN" --version | grep -qx "v$version"; then
  echo "[bonesdeploy] Node v${version} already installed."
  ensure_corepack
  exit 0
fi

arch="$(uname -m)"
case "$arch" in
  x86_64)  node_arch="x64" ;;
  aarch64) node_arch="arm64" ;;
  *) echo "[bonesdeploy] Unsupported architecture for Node binary install: $arch" >&2; exit 1 ;;
esac

url="https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
tmp="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp"
}
trap cleanup EXIT

echo "[bonesdeploy] Installing Node v${version}..."
echo "[bonesdeploy] Downloading $url"

mkdir -p "$(dirname "$NODE_DIR")"

curl -fsSL --retry 3 --retry-delay 2 "$url" | tar -xJ -C "$tmp"

rm -rf "$NODE_DIR"
mv "$tmp/node-v${version}-linux-${node_arch}" "$NODE_DIR"

ensure_corepack

echo "[bonesdeploy] Node installed: $(node --version)"
echo "[bonesdeploy] npm installed:  $(npm --version)"