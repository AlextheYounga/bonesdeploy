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

require_file() {
	local file="$1"
	local message="$2"

	[ -f "$file" ] || die "$message"
}

require_environment() {
	require_file artisan "artisan not found"
	require_command php
	require_command composer

	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_environment() {
	export COMPOSER_ALLOW_SUPERUSER="${COMPOSER_ALLOW_SUPERUSER:-1}"
	export CI=1
	export COREPACK_ENABLE_DOWNLOAD_PROMPT=0
}

artisan_command_exists() {
	local command_name="$1"

	php artisan list --raw 2>/dev/null |
		awk '{ print $1 }' |
		grep -qx -- "$command_name"
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
	require_environment
	configure_environment

	install_composer_dependencies

	trap - ERR

	log "Successfully installed php dependencies."
}

main "$@"
