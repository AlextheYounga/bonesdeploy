#!/usr/bin/env bash

BONES_NODE_TMP_DIR=""

log() {
	echo "[bonesdeploy] $*"
}

die() {
	echo "[bonesdeploy] $*" >&2
	exit 1
}

on_error() {
	local status=$?
	echo "[bonesdeploy] Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2
	exit "$status"
}

trap on_error ERR

node_cleanup() {
	if [ -n "${BONES_NODE_TMP_DIR:-}" ]; then
		rm -rf "$BONES_NODE_TMP_DIR"
	fi
}

trap node_cleanup EXIT

node_configure_paths() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"

	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR NODE_BIN
}

node_read_version_from_package_json() {
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

node_read_version() {
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

	node_read_version_from_package_json
}

node_resolve_version() {
	node_read_version |
		head -n 1 |
		sed -e 's/#.*$//' -e 's/\r$//' -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//' -e 's/^v//' || true
}

node_assert_exact_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	die "Node requires an exact pinned version. Set NODE_VERSION or use .node-version, .nvmrc, .tool-versions, or package.json volta."
}

node_ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@0.31.0
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] && "$NODE_BIN" --version | grep -qx "v$version"
}

node_architecture() {
	case "$(uname -m)" in
	x86_64)
		echo "x64"
		;;
	aarch64)
		echo "arm64"
		;;
	*)
		die "Unsupported architecture for Node binary install: $(uname -m)"
		;;
	esac
}

node_install() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(node_architecture)"
	url="https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
	BONES_NODE_TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"
	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar --no-same-owner -xJ -C "$BONES_NODE_TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$BONES_NODE_TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
	rmdir "$BONES_NODE_TMP_DIR"
	BONES_NODE_TMP_DIR=""
}

node_enable_toolchain() {
	node_configure_paths
	export PATH="$NODE_DIR/bin:$PATH"

	command -v node >/dev/null 2>&1 || die "node not found"
	command -v npm >/dev/null 2>&1 || die "npm not found"
	node_ensure_corepack
}
