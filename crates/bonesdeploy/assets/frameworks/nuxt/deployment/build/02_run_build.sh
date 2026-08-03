#!/usr/bin/env bash
set -Eeuo pipefail

source /workspace/deployment/functions.sh

node_enable_toolchain

build_command() {
	if [ "${BONES_FRAMEWORK_IS_STATIC:-false}" = "true" ]; then
		echo generate
	else
		echo build
	fi
}

run_build() {
	local command
	command="$(build_command)"

	if [ -f "./pnpm-lock.yaml" ]; then
		corepack pnpm install --store-dir "$PNPM_STORE_DIR" --frozen-lockfile
		corepack pnpm "$command"
	elif [ -f "./yarn.lock" ]; then
		corepack yarn install --frozen-lockfile
		corepack yarn "$command"
	elif [ -f "./package-lock.json" ]; then
		npm ci --include=optional
		npm run "$command"
	else
		echo "No lockfile found. Run your package manager locally first."
		exit 1
	fi
}

run_build

if [ -L dist ]; then
	rm dist
fi
