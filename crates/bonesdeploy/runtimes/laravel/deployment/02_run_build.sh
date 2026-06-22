#!/usr/bin/env bash

set -Eeuo pipefail

trap 'status=$?; echo "[bonesdeploy] Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2; exit "$status"' ERR

[ -f artisan ] || { echo "[bonesdeploy] artisan not found" >&2; exit 1; }

command -v php >/dev/null 2>&1 || { echo "[bonesdeploy] php not found" >&2; exit 1; }
command -v composer >/dev/null 2>&1 || { echo "[bonesdeploy] composer not found" >&2; exit 1; }

: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"

export COMPOSER_ALLOW_SUPERUSER="${COMPOSER_ALLOW_SUPERUSER:-1}"
export CI=1
export COREPACK_ENABLE_DOWNLOAD_PROMPT=0

artisan_command_exists() {
  local command_name="$1"
  php artisan list --raw 2>/dev/null | awk '{ print $1 }' | grep -qx "$command_name"
}

package_json_value() {
  local php_expr="$1"

  php -r '
    $p = json_decode(file_get_contents("package.json"), true) ?: [];
    '"$php_expr"'
  ' 2>/dev/null || true
}

detect_package_manager() {
  local package_manager

  package_manager="$(package_json_value '
    $pm = $p["packageManager"] ?? "";
    if ($pm) {
        echo explode("@", $pm)[0];
    }
  ')"

  if [ -n "$package_manager" ]; then
    printf '%s\n' "$package_manager"
    return
  fi

  if [ -f pnpm-lock.yaml ]; then
    echo "pnpm"
  elif [ -f yarn.lock ]; then
    echo "yarn"
  elif [ -f package-lock.json ] || [ -f npm-shrinkwrap.json ]; then
    echo "npm"
  else
    echo "npm"
  fi
}

has_build_script() {
  [ -f package.json ] || return 1

  [ "$(package_json_value '
    echo isset($p["scripts"]["build"]) ? "1" : "0";
  ')" = "1" ]
}

run_frontend_build() {
  [ -f package.json ] || {
    echo "[bonesdeploy] package.json not found; skipping frontend build."
    return
  }

  has_build_script || {
    echo "[bonesdeploy] package.json has no build script; skipping frontend build."
    return
  }

  export PATH="$PROJECT_ROOT/build/node/bin:$PATH"

  command -v node >/dev/null 2>&1 || {
    echo "[bonesdeploy] node not found. Did 01_install_build_deps.sh run?" >&2
    exit 1
  }

  if ! command -v corepack >/dev/null 2>&1; then
    echo "[bonesdeploy] corepack not found; installing corepack..."
    npm install -g corepack@latest
  fi

  corepack enable --install-directory "$(dirname "$(command -v node)")" 2>/dev/null || true

  local package_manager
  package_manager="$(detect_package_manager)"

  echo "[bonesdeploy] Node: $(node --version)"
  echo "[bonesdeploy] npm:  $(npm --version)"
  echo "[bonesdeploy] Frontend package manager: $package_manager"

  case "$package_manager" in
    npm)
      if [ -f package-lock.json ] || [ -f npm-shrinkwrap.json ]; then
        echo "[bonesdeploy] Installing frontend dependencies with npm ci..."
        npm ci --include=dev
      else
        echo "[bonesdeploy] package-lock.json not found; falling back to npm install..."
        npm install
      fi

      echo "[bonesdeploy] Building frontend assets with npm..."
      npm run build
      ;;

    pnpm)
      if [ -f pnpm-lock.yaml ]; then
        echo "[bonesdeploy] Installing frontend dependencies with pnpm frozen lockfile..."
        corepack pnpm install --frozen-lockfile --prod=false
      else
        echo "[bonesdeploy] pnpm-lock.yaml not found; falling back to non-frozen pnpm install..."
        corepack pnpm install --prod=false
      fi

      echo "[bonesdeploy] Building frontend assets with pnpm..."
      corepack pnpm run build
      ;;

    yarn)
      local yarn_version
      yarn_version="$(corepack yarn --version 2>/dev/null || true)"

      if [[ "$yarn_version" == 1.* ]]; then
        echo "[bonesdeploy] Installing frontend dependencies with Yarn classic..."
        corepack yarn install --frozen-lockfile
      else
        echo "[bonesdeploy] Installing frontend dependencies with Yarn modern..."
        corepack yarn install --immutable
      fi

      echo "[bonesdeploy] Building frontend assets with yarn..."
      corepack yarn run build
      ;;

    *)
      echo "[bonesdeploy] Unsupported package manager: $package_manager" >&2
      exit 1
      ;;
  esac
}

enter_maintenance_mode() {
  if [ "${BONES_LARAVEL_MAINTENANCE:-1}" = "0" ]; then
    echo "[bonesdeploy] Maintenance mode disabled by BONES_LARAVEL_MAINTENANCE=0."
    return
  fi

  echo "[bonesdeploy] Entering Laravel maintenance mode..."

  if php artisan down --render="errors::503"; then
    return
  fi

  php artisan down
}

exit_maintenance_mode() {
  php artisan up || true
}

echo "[bonesdeploy] Installing Composer dependencies..."
composer install --no-dev --prefer-dist --no-interaction --optimize-autoloader

# Generate frontend route/action files before Vite compiles the JS/TS bundle.
if artisan_command_exists "wayfinder:generate"; then
  echo "[bonesdeploy] Generating Wayfinder files..."
  php artisan wayfinder:generate
fi

run_frontend_build

enter_maintenance_mode
trap exit_maintenance_mode EXIT

if [ ! -f .env ] || ! grep -Eq '^APP_KEY=base64:' .env; then
  echo "[bonesdeploy] Generating Laravel APP_KEY..."
  php artisan key:generate --force
fi

echo "[bonesdeploy] Ensuring Laravel storage link exists..."
php artisan storage:link --force || true

if [ "${BONES_LARAVEL_SKIP_MIGRATIONS:-0}" = "1" ]; then
  echo "[bonesdeploy] Skipping migrations because BONES_LARAVEL_SKIP_MIGRATIONS=1."
else
  echo "[bonesdeploy] Running migrations..."
  php artisan migrate --force
fi

php artisan optimize:clear

if artisan_command_exists "queue:restart"; then
  php artisan queue:restart || true
fi

php artisan up
trap - EXIT

echo "[bonesdeploy] Laravel build complete."
