#!/usr/bin/env bash

set -Eeuo pipefail

readonly VENV_DIR="${VENV_DIR:-.venv}"

ensure_virtualenv() {
	if [ -d "$VENV_DIR" ]; then
		return
	fi

	log "Creating Django virtual environment at $VENV_DIR..."
	python3 -m venv "$VENV_DIR"
}

activate_virtualenv() {
	# shellcheck disable=SC1091
	source "$VENV_DIR/bin/activate"
}

install_python_dependencies() {
	log "Installing Django Python dependencies..."

	if [ -f requirements.txt ]; then
		python -m pip install -r requirements.txt --quiet
		return
	fi

	log "No requirements.txt found; skipping Python dependency install."
}

validate_application() {
	[ -x "$VENV_DIR/bin/gunicorn" ] || die "gunicorn not found in $VENV_DIR; add it to requirements.txt"

	log "Checking Django production configuration..."
	python manage.py check --deploy
}

run_migrations() {
	if [ "${BONES_DJANGO_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_DJANGO_SKIP_MIGRATIONS=1."
		return
	fi

	log "Running Django migrations..."
	python manage.py migrate --noinput
}

collect_static() {
	log "Collecting Django static files..."
	python manage.py collectstatic --noinput
}

main() {
	if [ ! -f manage.py ]; then
		log "manage.py not found; skipping Django prepare."
		exit 0
	fi

	command -v python3 >/dev/null 2>&1 || {
		echo "$LOG_PREFIX python3 not found" >&2
		exit 1
	}

	ensure_virtualenv
	activate_virtualenv
	install_python_dependencies
	validate_application
	run_migrations
	collect_static

	trap - ERR

	log "Django prepare complete."
}

main "$@"
