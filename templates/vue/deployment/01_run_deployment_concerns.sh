#!/usr/bin/env bash

set -Eeuo pipefail

# Load nvm if .nvmrc is present
if [ -f "./.nvmrc" ]; then
  export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
  if [ -s "$NVM_DIR/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$NVM_DIR/nvm.sh"
  elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
    # shellcheck disable=SC1090
    source "$HOME/.config/nvm/nvm.sh"
  fi
  nvm install
fi

# Clean install and build
rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
  command -v pnpm >/dev/null 2>&1 || {
    echo "pnpm not found. Install it globally or enable it via corepack before deploy."
    exit 1
  }
  pnpm install --frozen-lockfile
  pnpm build
elif [ -f "./yarn.lock" ]; then
  command -v corepack >/dev/null 2>&1 && corepack enable || true
  yarn install --frozen-lockfile
  yarn build
elif [ -f "./package-lock.json" ]; then
  npm ci
  npm run build
else
  echo "No lockfile found. Run your package manager locally first."
  exit 1
fi
