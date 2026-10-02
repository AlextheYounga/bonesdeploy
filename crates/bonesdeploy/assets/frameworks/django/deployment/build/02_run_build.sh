#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

main() {
	if [ ! -f manage.py ]; then
		log "manage.py not found; skipping Django build."
		exit 0
	fi

	local python_version="${BONES_RUNTIME_PYTHON_VERSION:?BONES_RUNTIME_PYTHON_VERSION is required}"
	local python_binary="python${python_version}"
	local packages_dir=".python-packages"

	command -v "$python_binary" >/dev/null 2>&1 || die "$python_binary not found"
	rm -rf "$packages_dir" .venv
	mkdir -p "$packages_dir"

	if [ -f requirements.txt ]; then
		log "Installing Django Python dependencies..."
		"$python_binary" -m pip install --target "$packages_dir" -r requirements.txt --quiet
	else
		log "No requirements.txt found; skipping Python dependency install."
	fi

	# Dependencies are resolved locally but rebuilt against the target interpreter.
	rm -rf "$packages_dir" .venv deployment/build
	log "Django build complete."
}

main "$@"
