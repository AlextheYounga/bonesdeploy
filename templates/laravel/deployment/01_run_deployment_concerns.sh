#!/usr/bin/env bash

set -Eeuo pipefail

[ -f artisan ] || { echo "artisan not found"; exit 1; }
command -v php >/dev/null 2>&1 || { echo "php not found"; exit 1; }
command -v composer >/dev/null 2>&1 || { echo "composer not found"; exit 1; }

# Maintenance mode first — before anything destructive
php artisan down
trap 'php artisan up || true' EXIT

# Install PHP dependencies
composer install --no-dev --prefer-dist --no-interaction --optimize-autoloader

# Frontend build
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

command -v pnpm >/dev/null 2>&1 || {
  echo "pnpm not found. Install it globally or enable it via corepack before deploy."
  exit 1
}

pnpm install --frozen-lockfile
pnpm run build

if php artisan list | grep -q 'wayfinder:generate'; then
  php artisan wayfinder:generate
fi

if [ ! -f .env ] || ! grep -Eq '^APP_KEY=base64:' .env; then
  php artisan key:generate --force
fi

php artisan migrate --force

# Clear old caches and rebuild them back-to-back
php artisan optimize:clear
php artisan config:cache
php artisan route:cache
php artisan view:cache
php artisan event:cache || true
php artisan queue:restart || true

php artisan up
trap - EXIT
