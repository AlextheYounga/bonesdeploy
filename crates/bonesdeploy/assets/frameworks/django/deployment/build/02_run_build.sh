#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

main() {
	# Production prepare creates the release virtualenv with the host Python.
	# Do not package a local dependency tree or require a builder Python version.
	rm -rf .python-packages .venv deployment/build
	log "Django build complete."
}

main "$@"
