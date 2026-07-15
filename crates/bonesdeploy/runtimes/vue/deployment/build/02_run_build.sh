#!/usr/bin/env bash

set -Eeuo pipefail

export PATH="$PROJECT_ROOT/build/node/bin:$PATH"
export NODE_OPTIONS="--max-old-space-size-percentage=70 --max-semi-space-size=64"
export UV_THREADPOOL_SIZE=4

if ! command -v corepack >/dev/null 2>&1; then
	npm install -g corepack@0.31.0
fi

corepack enable --install-directory "$(dirname "$(command -v node)")" 2>/dev/null || true

rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
	corepack pnpm install --frozen-lockfile
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
