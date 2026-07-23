#!/usr/bin/env bash
set -Eeuo pipefail

source /workspace/deployment/functions.sh

node_enable_toolchain

if [ -f "./pnpm-lock.yaml" ]; then
	corepack pnpm install --store-dir "$PNPM_STORE_DIR" --frozen-lockfile
	corepack pnpm generate
elif [ -f "./yarn.lock" ]; then
	corepack yarn install --frozen-lockfile
	corepack yarn generate
elif [ -f "./package-lock.json" ]; then
	npm ci --include=optional
	npm run generate
else
	echo "No lockfile found. Run your package manager locally first."
	exit 1
fi

if [ -L dist ]; then
	rm dist
fi
