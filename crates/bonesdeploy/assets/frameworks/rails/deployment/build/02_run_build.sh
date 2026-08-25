#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

skip_unless_rails_project() {
	if [ ! -f Gemfile ]; then
		log "Gemfile not found; skipping Rails build."
		exit 0
	fi
}

install_application_packages() {
	apt-get update
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		default-libmysqlclient-dev \
		git \
		libsqlite3-dev \
		pkg-config
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
	ruby_enable_toolchain
	install_application_packages
	install_bundle_dependencies
	precompile_assets

	trap - ERR

	log "Rails build complete."
}

main "$@"
