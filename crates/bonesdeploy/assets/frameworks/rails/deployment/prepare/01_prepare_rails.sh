#!/usr/bin/env bash

set -Eeuo pipefail

readonly RUBY_MARKER=".bonesdeploy-ruby-version"
readonly PACKAGED_BUNDLER="vendor/bundle/bundler/bin/bundle"

validate_runtime() {
	local site_root site_ruby expected_version actual_version
	site_root="$(cd ../.. && pwd -P)"
	site_ruby="$site_root/.bonesdeploy/runtimes/ruby/bin/ruby"

	[ -x "$site_ruby" ] || die "Provisioned site Ruby not found: $site_ruby"
	[ -f "$RUBY_MARKER" ] || die "$RUBY_MARKER not found; build the Rails artifact first"
	expected_version="$(<"$RUBY_MARKER")"
	[[ "$expected_version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "$RUBY_MARKER must contain an exact Ruby version"
	actual_version="$("$site_ruby" --version | awk '{ print $2 }')"
	[ "$actual_version" = "$expected_version" ] || die "Provisioned Ruby $actual_version does not match artifact Ruby $expected_version"
	[ -f "$PACKAGED_BUNDLER" ] || die "$PACKAGED_BUNDLER not found; build the Rails artifact first"

	SITE_RUBY="$site_ruby"
}

main() {
	if [ ! -f Gemfile ]; then
		log "Gemfile not found; skipping Rails prepare."
		exit 0
	fi

	validate_runtime

	export BUNDLE_DEPLOYMENT="true"
	export BUNDLE_DISABLE_VERSION_CHECK="true"
	export BUNDLE_PATH="vendor/bundle"
	export BUNDLE_WITHOUT="development:test"

	[ -d vendor/bundle ] || die "vendor/bundle not found; build the Rails artifact first"
	log "Checking packaged production bundle..."
	"$SITE_RUBY" "$PACKAGED_BUNDLER" check

	if [ "${BONES_RAILS_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_RAILS_SKIP_MIGRATIONS=1."
		exit 0
	fi

	log "Running Rails migrations..."
	RAILS_ENV=production "$SITE_RUBY" "$PACKAGED_BUNDLER" exec rails db:migrate

	trap - ERR

	log "Rails prepare complete."
}

main "$@"
