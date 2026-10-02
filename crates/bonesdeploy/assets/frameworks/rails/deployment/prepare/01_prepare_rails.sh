#!/usr/bin/env bash

set -Eeuo pipefail

main() {
	if [ ! -f Gemfile ]; then
		log "Gemfile not found; skipping Rails prepare."
		exit 0
	fi

	export BUNDLE_DEPLOYMENT="true"
	export BUNDLE_DISABLE_VERSION_CHECK="true"
	export BUNDLE_PATH="vendor/bundle"
	export BUNDLE_VERSION="system"
	export BUNDLE_WITHOUT="development:test"

	[ -d vendor/bundle ] || die "vendor/bundle not found; build the Rails artifact first"
	log "Checking packaged production bundle..."
	/usr/bin/bundle check

	if [ "${BONES_RAILS_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_RAILS_SKIP_MIGRATIONS=1."
		exit 0
	fi

	log "Running Rails migrations..."
	RAILS_ENV=production /usr/bin/bundle exec rails db:migrate

	trap - ERR

	log "Rails prepare complete."
}

main "$@"
