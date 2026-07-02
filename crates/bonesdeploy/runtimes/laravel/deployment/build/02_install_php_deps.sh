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

require_file() {
	local file="$1"
	local message="$2"

	[ -f "$file" ] || die "$message"
}

skip_unless_laravel_project() {
	if [ ! -f artisan ]; then
		log "artisan not found; skipping Laravel PHP dependency install."
		exit 0
	fi
}

require_environment() {
	require_file artisan "artisan not found"

	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

install_system_packages() {
	log "Installing PHP and Composer build packages..."
	export DEBIAN_FRONTEND=noninteractive
	apt-get update
	apt-get install -y --no-install-recommends \
		composer \
		git \
		php-cli \
		php-curl \
		php-mbstring \
		php-sqlite3 \
		php-xml \
		php-zip \
		unzip
}

configure_environment() {
	export COMPOSER_ALLOW_SUPERUSER="${COMPOSER_ALLOW_SUPERUSER:-1}"
	export CI=1
	export COREPACK_ENABLE_DOWNLOAD_PROMPT=0
}

install_composer_dependencies() {
	log "Installing Composer dependencies..."

	composer install \
		--no-dev \
		--prefer-dist \
		--no-interaction \
		--optimize-autoloader
}

main() {
	skip_unless_laravel_project
	require_environment
	install_system_packages
	configure_environment

	install_composer_dependencies

	trap - ERR

	log "Successfully installed php dependencies."
}

main "$@"
