#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

on_error() {
	local status=$?
	echo "$LOG_PREFIX Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2
	exit "$status"
}

trap on_error ERR

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_command() {
	local name="$1"

	command -v "$name" >/dev/null 2>&1 || die "$name not found"
}

artisan_command_exists() {
	local command_name="$1"

	php artisan list --raw 2>/dev/null |
		awk '{ print $1 }' |
		grep -qx -- "$command_name"
}

ensure_app_key() {
	if [ ! -f .env ] || ! grep -Eq '^APP_KEY=base64:' .env; then
		log "Generating Laravel APP_KEY..."
		php artisan key:generate --force
	fi
}

ensure_storage_link() {
	log "Ensuring Laravel storage link exists..."
	php artisan storage:link --force || true
}

run_migrations() {
	if [ "${BONES_LARAVEL_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_LARAVEL_SKIP_MIGRATIONS=1."
		return
	fi

	log "Running migrations..."
	php artisan migrate --force
}

restart_queue_workers() {
	if artisan_command_exists "queue:restart"; then
		php artisan queue:restart || true
	fi
}

finish_laravel_build() {
	php artisan optimize:clear
	restart_queue_workers
	php artisan up
}

main() {
	ensure_app_key
	ensure_storage_link
	run_migrations
	finish_laravel_build

	trap - ERR

	log "Laravel build complete."
}

main "$@"
