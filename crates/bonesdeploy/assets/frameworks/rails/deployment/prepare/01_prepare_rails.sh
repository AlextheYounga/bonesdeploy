#!/usr/bin/env bash

set -Eeuo pipefail

main() {
	if [ ! -f Gemfile ]; then
		log "Gemfile not found; skipping Rails prepare."
		exit 0
	fi

	export HOME="$PROJECT_ROOT/shared"
	export BUNDLE_PATH="vendor/bundle"
	export BUNDLE_USER_CACHE="$PROJECT_ROOT/shared/.bundle-cache"
	export BUNDLE_WITHOUT="development:test"

	log "Installing production bundle with the host Ruby..."
	/usr/bin/bundle install --deployment --without development test

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
