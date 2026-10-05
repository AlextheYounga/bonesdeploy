#!/usr/bin/env bash

set -Eeuo pipefail

readonly PYTHON_RUNTIME_MARKER=".bonesdeploy-python-runtime"
readonly SITE_ROOT="$(cd -- "$PWD/../.." && pwd)"
readonly PYTHON_BIN="$SITE_ROOT/.bonesdeploy/runtimes/python/bin/python"

validate_artifact() {
	[ -f "$PYTHON_RUNTIME_MARKER" ] || die "$PYTHON_RUNTIME_MARKER not found; build the Django artifact first"
	[ -x "$PYTHON_BIN" ] || die "$PYTHON_BIN not found; provision the configured Python runtime first"

	local artifact_identity provisioned_identity
	artifact_identity="$(<"$PYTHON_RUNTIME_MARKER")"
	provisioned_identity="$("$PYTHON_BIN" --version)"
	[ "$artifact_identity" = "$provisioned_identity" ] || die "Django artifact Python runtime $artifact_identity does not match provisioned runtime $provisioned_identity"
}

validate_application() {
	[ -x .venv/bin/gunicorn ] || die "gunicorn launcher not found; add gunicorn to requirements.txt"

	log "Checking Django production configuration..."
	"$PYTHON_BIN" manage.py check --deploy
}

run_migrations() {
	if [ "${BONES_DJANGO_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_DJANGO_SKIP_MIGRATIONS=1."
		return
	fi

	log "Running Django migrations..."
	"$PYTHON_BIN" manage.py migrate --noinput
}

collect_static() {
	log "Collecting Django static files..."
	"$PYTHON_BIN" manage.py collectstatic --noinput
}

main() {
	if [ ! -f manage.py ]; then
		log "manage.py not found; skipping Django prepare."
		exit 0
	fi

	validate_artifact
	export PYTHONPATH="$PWD/.python-packages${PYTHONPATH:+:$PYTHONPATH}"
	validate_application
	run_migrations
	collect_static

	trap - ERR

	log "Django prepare complete."
}

main "$@"
