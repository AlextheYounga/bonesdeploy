#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"
export NODE_OPTIONS="--max-old-space-size-percentage=70"

on_error() {
	local status=$?
	echo "$LOG_PREFIX Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2
	exit "$status"
}

trap on_error ERR

log() {
	echo "$LOG_PREFIX $*"
}

skip_unless_rails_project() {
	if [ ! -f Gemfile ]; then
		log "Gemfile not found; skipping Rails build."
		exit 0
	fi
}

install_system_packages() {
	log "Installing Ruby and native build packages..."
	export DEBIAN_FRONTEND=noninteractive
	apt-get update
	apt-get install -y --no-install-recommends \
		build-essential \
		default-libmysqlclient-dev \
		git \
		libffi-dev \
		libpq-dev \
		libsqlite3-dev \
		libssl-dev \
		libyaml-dev \
		pkg-config \
		ruby-bundler \
		ruby-full \
		zlib1g-dev
}

install_bundle_dependencies() {
	export BUNDLE_WITHOUT="development:test"
	log "Installing bundle dependencies..."
	bundle install --deployment --without development test
}

precompile_assets() {
	log "Precompiling Rails assets..."
	SECRET_KEY_BASE_DUMMY=1 RAILS_ENV=production bundle exec rails assets:precompile
}

main() {
	skip_unless_rails_project
	install_system_packages
	install_bundle_dependencies
	precompile_assets

	trap - ERR

	log "Rails build complete."
}

main "$@"
