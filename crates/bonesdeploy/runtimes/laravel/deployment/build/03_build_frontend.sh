#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"
export NODE_OPTIONS="--max-old-space-size-percentage=65 --max-semi-space-size=32"
export UV_THREADPOOL_SIZE=2

on_error() {
	local status=$?
	echo "$LOG_PREFIX Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2
	exit "$status"
}

trap on_error ERR

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_command() {
	local name="$1"

	command -v "$name" >/dev/null 2>&1 || die "$name not found"
}

artisan_command_exists() {
	local command_name="$1"

	php artisan list --raw 2>/dev/null |
		awk '{ print $1 }' |
		grep -qx -- "$command_name"
}

package_json_package_manager() {
	[ -f package.json ] || return 0

	awk '
		$0 ~ /"packageManager"[[:space:]]*:[[:space:]]*"/ {
			line = $0
			if (sub(/.*"packageManager"[[:space:]]*:[[:space:]]*"/, "", line)) {
				sub(/".*/, "", line)
				split(line, parts, "@")
				print parts[1]
				exit
			}
		}
	' package.json 2>/dev/null || true
}

package_json_has_build_script() {
	[ -f package.json ] || return 1

	[ "$(
		awk '
			$0 ~ /"scripts"[[:space:]]*:[[:space:]]*{/ {
				in_scripts = 1
			}

			in_scripts {
				line = $0
				if (sub(/.*"build"[[:space:]]*:[[:space:]]*"/, "", line)) {
					print "1"
					exit
				}
			}

			in_scripts && $0 ~ /}/ {
				in_scripts = 0
			}
		' package.json 2>/dev/null || true
	)" = "1" ]
}

detect_package_manager() {
	local package_manager

	package_manager="$(package_json_package_manager)"

	if [ -n "$package_manager" ]; then
		echo "$package_manager"
		return
	fi

	if [ -f pnpm-lock.yaml ]; then
		echo "pnpm"
	elif [ -f yarn.lock ]; then
		echo "yarn"
	else
		echo "npm"
	fi
}

ensure_node_toolchain() {
	export PATH="$PROJECT_ROOT/build/node/bin:$PATH"

	require_command php
	require_command node
	require_command npm

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found; installing corepack..."
		npm install -g corepack@0.31.0
	fi

	corepack enable --install-directory "$(dirname "$(command -v node)")" 2>/dev/null || true
}

install_and_build_with_npm() {
	if [ -f package-lock.json ] || [ -f npm-shrinkwrap.json ]; then
		log "Installing frontend dependencies with npm ci..."
		npm ci --include=dev
	else
		die "package-lock.json or npm-shrinkwrap.json is required for production builds"
	fi

	log "Building frontend assets with npm..."
	npm run build
}

install_and_build_with_pnpm() {
	if [ -f pnpm-lock.yaml ]; then
		log "Installing frontend dependencies with pnpm frozen lockfile..."
		corepack pnpm install --frozen-lockfile --prod=false
	else
		die "pnpm-lock.yaml is required for production builds"
	fi

	log "Building frontend assets with pnpm..."
	corepack pnpm run build
}

install_and_build_with_yarn() {
	local yarn_version

	yarn_version="$(corepack yarn --version 2>/dev/null || true)"

	if [[ "$yarn_version" == 1.* ]]; then
		log "Installing frontend dependencies with Yarn classic..."
		corepack yarn install --frozen-lockfile
	else
		log "Installing frontend dependencies with Yarn modern..."
		corepack yarn install --immutable
	fi

	log "Building frontend assets with yarn..."
	corepack yarn run build
}

run_frontend_build() {
	local package_manager

	if [ ! -f package.json ]; then
		log "package.json not found; skipping frontend build."
		return
	fi

	if ! package_json_has_build_script; then
		log "package.json has no build script; skipping frontend build."
		return
	fi

	ensure_node_toolchain

	package_manager="$(detect_package_manager)"

	log "Node: $(node --version)"
	log "npm:  $(npm --version)"
	log "Frontend package manager: $package_manager"

	case "$package_manager" in
	npm)
		install_and_build_with_npm
		;;

	pnpm)
		install_and_build_with_pnpm
		;;

	yarn)
		install_and_build_with_yarn
		;;

	*)
		die "Unsupported package manager: $package_manager"
		;;
	esac
}

generate_wayfinder_files() {
	if ! artisan_command_exists "wayfinder:generate"; then
		log "wayfinder:generate not available; skipping Wayfinder generation."
		return
	fi

	log "Generating Wayfinder files..."
	php artisan wayfinder:generate
}

main() {
	# Generate frontend route/action files before Vite compiles the JS/TS bundle.
	generate_wayfinder_files

	run_frontend_build

	trap - ERR

	log "Frontend built successfully"
}

main "$@"
