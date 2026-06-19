  #!/usr/bin/env bash
  set -Eeuo pipefail

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

  if [ -f "./pnpm-lock.yaml" ]; then
    npm install -g pnpm
    pnpm install --frozen-lockfile
    pnpm generate
  elif [ -f "./yarn.lock" ]; then
    command -v corepack >/dev/null 2>&1 && corepack enable || true
    yarn install --frozen-lockfile
    yarn generate
  elif [ -f "./package-lock.json" ]; then
    npm ci --include=optional
    npm run generate
  else
    echo "No lockfile found. Run your package manager locally first."
    exit 1
  fi
