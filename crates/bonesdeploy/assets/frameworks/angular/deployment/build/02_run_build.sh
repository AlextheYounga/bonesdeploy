#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

node_enable_toolchain

rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
	corepack pnpm install --store-dir "$PNPM_STORE_DIR" --frozen-lockfile
elif [ -f "./yarn.lock" ]; then
	corepack yarn install --frozen-lockfile
elif [ -f "./package-lock.json" ]; then
	npm ci --include=optional
else
	echo "No lockfile found. Run your package manager locally first."
	exit 1
fi

./node_modules/.bin/ng build --configuration production --output-path dist

if [ ! -f "dist/browser/index.html" ]; then
	echo "Angular build did not produce dist/browser/index.html. This template requires @angular/build:application."
	exit 1
fi

rm -rf .angular/cache node_modules deployment/build
