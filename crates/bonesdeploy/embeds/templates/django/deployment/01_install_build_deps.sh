#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
  exit 0
fi

if command -v curl >/dev/null 2>&1; then
  curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
elif command -v wget >/dev/null 2>&1; then
  wget -qO- "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
  echo "curl or wget is required to install nvm."
  exit 1
fi

# Verify the installer wrote the expected shell hook.
# shellcheck disable=SC1090
source "$NVM_DIR/nvm.sh"
command -v nvm >/dev/null 2>&1
