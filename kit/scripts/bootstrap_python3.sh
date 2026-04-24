#!/usr/bin/env bash
set -euo pipefail

if command -v python3 >/dev/null 2>&1; then
  exit 0
fi

if ! command -v apt-get >/dev/null 2>&1; then
  echo "apt-get is required to bootstrap python3" >&2
  exit 1
fi

install_python() {
  export DEBIAN_FRONTEND=noninteractive
  apt-get update
  apt-get install -y python3 python3-apt
}

if [ "$(id -u)" -eq 0 ]; then
  install_python
  exit 0
fi

if ! command -v sudo >/dev/null 2>&1; then
  echo "sudo is required to bootstrap python3" >&2
  exit 1
fi

sudo bash -s <<'EOF'
set -euo pipefail
export DEBIAN_FRONTEND=noninteractive
apt-get update
apt-get install -y python3 python3-apt
EOF
