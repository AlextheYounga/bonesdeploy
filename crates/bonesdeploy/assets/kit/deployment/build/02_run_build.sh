#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

node_enable_toolchain

rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
	corepack pnpm install --store-dir "$PNPM_STORE_DIR" --frozen-lockfile
	corepack pnpm build
elif [ -f "./yarn.lock" ]; then
	corepack yarn install --frozen-lockfile
	corepack yarn build
elif [ -f "./package-lock.json" ]; then
	npm install --include=optional
	npm run build
else
	echo "No lockfile found. Run your package manager locally first."
	exit 1
fi
