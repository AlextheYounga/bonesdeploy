#!/usr/bin/env bash

set -Eeuo pipefail

readonly VENV_DIR="${VENV_DIR:-.venv}"
readonly PYTHON_BIN="$VENV_DIR/bin/python"

validate_application() {
	[ -x "$VENV_DIR/bin/gunicorn" ] || die "gunicorn not found in $VENV_DIR; add it to requirements.txt"

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

	[ -x "$PYTHON_BIN" ] || die "$PYTHON_BIN not found; build the Django artifact first"
	validate_application
	run_migrations
	collect_static

	trap - ERR

	log "Django prepare complete."
}

main "$@"
