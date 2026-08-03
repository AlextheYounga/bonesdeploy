#!/usr/bin/env bash

set -Eeuo pipefail

ensure_storage_dirs() {
	log "Ensuring Laravel storage directory structure..."
	# If storage is a symlink (e.g. linked to shared/storage by the release
	# lifecycle), mkdir -p fails with EEXIST on a dangling symlink because
	# the symlink inode exists but stat can't resolve it to a directory.
	# Resolve the real target path first so mkdir operates on the destination.
	local base
	if [ -L storage ]; then
		base="$(readlink -m storage)"
	else
		base="${PWD}/storage"
	fi
	mkdir -p \
		"${base}/framework/cache/data" \
		"${base}/framework/sessions" \
		"${base}/framework/views" \
		"${base}/logs" \
		"${base}/app/public"
}

ensure_app_key() {
	if [ ! -f .env ] || ! grep -Eq '^APP_KEY=base64:' .env; then
		log "Generating Laravel APP_KEY..."
		php artisan key:generate --force
	fi
}

ensure_storage_link() {
	log "Ensuring Laravel storage link exists..."
	php artisan storage:link --force
}

run_migrations() {
	if [ "${BONES_LARAVEL_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_LARAVEL_SKIP_MIGRATIONS=1."
		return
	fi

	log "Running migrations..."
	php artisan migrate --force
}

finish_laravel_prepare() {
	php artisan optimize
}

main() {
	ensure_storage_dirs
	ensure_app_key
	ensure_storage_link
	run_migrations
	finish_laravel_prepare

	trap - ERR

	log "Laravel prepare complete."
}

main "$@"
