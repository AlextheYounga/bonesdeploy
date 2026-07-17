#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

skip_unless_laravel_project() {
	if [ ! -f artisan ]; then
		log "artisan not found; skipping Laravel build dependency install."
		exit 0
	fi
}

skip_unless_node_project() {
	if [ ! -f package.json ]; then
		log "package.json not found; skipping Node install."
		exit 0
	fi
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	local version

	version="$(awk '
		$0 ~ /"volta"[[:space:]]*:[[:space:]]*{/ {
			in_section = 1
		}

		in_section {
			line = $0
			if (sub(/.*"node"[[:space:]]*:[[:space:]]*"/, "", line)) {
				sub(/".*/, "", line)
				print line
				exit
			}
		}

		in_section && $0 ~ /}/ {
			in_section = 0
		}
	' package.json)"

	if [ -n "$version" ]; then
		echo "$version"
		return
	fi

	awk '
		$0 ~ /"engines"[[:space:]]*:[[:space:]]*{/ {
			in_section = 1
		}

		in_section {
			line = $0
			if (sub(/.*"node"[[:space:]]*:[[:space:]]*"/, "", line)) {
				sub(/".*/, "", line)
				print line
				exit
			}
		}

		in_section && $0 ~ /}/ {
			in_section = 0
		}
	' package.json
}

read_node_version() {
	if [ -n "${NODE_VERSION:-}" ]; then
		echo "$NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@0.31.0
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar --no-same-owner -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	skip_unless_laravel_project
	skip_unless_node_project

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"
