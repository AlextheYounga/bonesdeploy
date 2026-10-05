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
	export BUNDLE_DEPLOYMENT="true"
	export BUNDLE_PATH="vendor/bundle"
	export BUNDLE_WITHOUT="development:test"
	local bundler_version packaged_bundler
	bundler_version="$(awk '/^BUNDLED WITH$/ { getline; print; exit }' Gemfile.lock)"
	[[ "$bundler_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Gemfile.lock must declare an exact Bundler version"
	packaged_bundler="vendor/bundle/bundler/bin/bundle"

	log "Packaging Bundler ${bundler_version}..."
	"$RUBY_BIN" -S gem install bundler --version "$bundler_version" --install-dir vendor/bundle/bundler --bindir vendor/bundle/bundler/bin --no-document
	[ -f "$packaged_bundler" ] || die "Bundler ${bundler_version} was not packaged"
	log "Installing bundle dependencies..."
	"$RUBY_BIN" "$packaged_bundler" install
}

precompile_assets() {
	log "Precompiling Rails assets..."
	SECRET_KEY_BASE_DUMMY=1 RAILS_ENV=production "$RUBY_BIN" vendor/bundle/bundler/bin/bundle exec rails assets:precompile
}

main() {
	skip_unless_rails_project
	ruby_enable_toolchain
	"$RUBY_BIN" --version | awk '{ print $2 }' >.bonesdeploy-ruby-version
	install_application_packages
	install_bundle_dependencies
	precompile_assets
	rm -rf node_modules tmp/cache deployment/build

	trap - ERR

	log "Rails build complete."
}

main "$@"
