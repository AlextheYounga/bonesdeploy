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

main() {
	if [ ! -f Gemfile ]; then
		log "Gemfile not found; skipping Rails prepare."
		exit 0
	fi

	if [ "${BONES_RAILS_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_RAILS_SKIP_MIGRATIONS=1."
		exit 0
	fi

	log "Running Rails migrations..."
	RAILS_ENV=production bundle exec rails db:migrate

	trap - ERR

	log "Rails prepare complete."
}

main "$@"
