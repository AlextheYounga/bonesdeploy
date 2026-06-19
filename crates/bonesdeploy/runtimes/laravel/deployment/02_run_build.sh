#!/usr/bin/env bash

set -Eeuo pipefail

trap 'status=$?; echo "[bonesdeploy] Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2; exit "$status"' ERR

[ -f artisan ] || { echo "artisan not found"; exit 1; }
command -v php >/dev/null 2>&1 || { echo "php not found"; exit 1; }
command -v composer >/dev/null 2>&1 || { echo "composer not found"; exit 1; }

# Install PHP dependencies first — artisan requires vendor/autoload.php
echo "[bonesdeploy] Installing Composer dependencies..."
composer install --no-dev --prefer-dist --no-interaction --optimize-autoloader

# Maintenance mode once the app can boot
echo "[bonesdeploy] Entering Laravel maintenance mode..."
php artisan down --render="errors::503"
trap 'php artisan up || true' EXIT

# Frontend build. Node was installed by 01_install_build_deps.sh at $PROJECT_ROOT/build/node.
export PATH="${PROJECT_ROOT:-.}/build/node/bin:$PATH"

command -v pnpm >/dev/null 2>&1 || {
  echo "pnpm not found. Install it globally or enable it via corepack before deploy."
  exit 1
}

echo "[bonesdeploy] Installing frontend dependencies..."
pnpm install --frozen-lockfile
echo "[bonesdeploy] Building frontend assets..."
pnpm run build

if php artisan list | grep -q 'wayfinder:generate'; then
  php artisan wayfinder:generate
fi

if [ ! -f .env ] || ! grep -Eq '^APP_KEY=base64:' .env; then
  php artisan key:generate --force
fi

echo "[bonesdeploy] Running migrations..."
php artisan migrate --force

# Clear old caches and rebuild them back-to-back
echo "[bonesdeploy] Rebuilding Laravel caches..."
php artisan optimize:clear
php artisan config:cache
php artisan route:cache
php artisan view:cache
php artisan event:cache || true
php artisan queue:restart || true

php artisan up
trap - EXIT
