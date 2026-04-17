```bash
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
  npm install -g pnpm
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

# Restart the application via pm2
if command -v pm2 >/dev/null 2>&1; then
  if pm2 describe "$PROJECT_NAME" >/dev/null 2>&1; then
    pm2 restart "$PROJECT_NAME"
  else
    pm2 start npm --name "$PROJECT_NAME" -- start
  fi
  pm2 save
else
  echo "pm2 not found. Install it globally: npm install -g pm2"
  exit 1
fi
```
