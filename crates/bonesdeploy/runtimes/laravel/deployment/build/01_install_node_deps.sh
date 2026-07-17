#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

if [ ! -f artisan ]; then
	log "artisan not found; skipping Laravel build dependency install."
	exit 0
fi

if [ ! -f package.json ]; then
	log "package.json not found; skipping Node install."
	exit 0
fi

install_node_dependencies
