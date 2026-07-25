#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

main() {
	if [ ! -f manage.py ]; then
		log "manage.py not found; skipping Django build."
		exit 0
	fi

	log "Django runtime setup now happens in prepare scripts. Build has no runtime-state work to do."
}

main "$@"
