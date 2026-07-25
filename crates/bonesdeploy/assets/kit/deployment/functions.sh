#!/usr/bin/env bash

BUILD_NODE_TMP_DIR=""
COREPACK_VERSION="0.31.0"

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

cleanup_node_install() {
	if [ -n "${BUILD_NODE_TMP_DIR:-}" ]; then
		rm -rf "$BUILD_NODE_TMP_DIR"
	fi
}

trap cleanup_node_install EXIT

configure_build_cache() {
	[ -n "${BUILD_CACHE_DIR:-}" ] || return 0

	local directory
	for directory in \
		"$BUILD_CACHE_DIR/corepack" \
		"$BUILD_CACHE_DIR/npm" \
		"$BUILD_CACHE_DIR/pnpm" \
		"$BUILD_CACHE_DIR/yarn/cache" \
		"$BUILD_CACHE_DIR/yarn/global" \
		"$BUILD_CACHE_DIR/composer" \
		"$BUILD_CACHE_DIR/bundler" \
		"$BUILD_CACHE_DIR/node"; do
		mkdir -p "$directory"
	done

	export COREPACK_HOME="$BUILD_CACHE_DIR/corepack"
	export NPM_CONFIG_CACHE="$BUILD_CACHE_DIR/npm"
	export PNPM_STORE_DIR="$BUILD_CACHE_DIR/pnpm"
	export YARN_CACHE_FOLDER="$BUILD_CACHE_DIR/yarn/cache"
	export YARN_GLOBAL_FOLDER="$BUILD_CACHE_DIR/yarn/global"
	export COMPOSER_CACHE_DIR="$BUILD_CACHE_DIR/composer"
	export BUNDLE_USER_CACHE="$BUILD_CACHE_DIR/bundler"
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

node_configure_paths() {
	local version="$1"
	local node_arch="$2"

	: "${BUILD_CACHE_DIR:?BUILD_CACHE_DIR must be set by bonesremote}"

	NODE_DIR="$BUILD_CACHE_DIR/node/v${version}-linux-${node_arch}"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR NODE_BIN
}

node_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] && "$NODE_BIN" --version | grep -qx "v$version"
}

node_install() {
	local version="$1"
	local node_arch="$2"
	local archive="node-v${version}-linux-${node_arch}.tar.xz"
	local base_url="https://nodejs.org/dist/v${version}"
	local checksum_line

	node_configure_paths "$version" "$node_arch"
	BUILD_NODE_TMP_DIR="$(mktemp -d "$BUILD_CACHE_DIR/node/.tmp.XXXXXX")"

	log "Installing Node v${version}..."
	log "Downloading ${base_url}/${archive}"
	curl -fsSL --retry 3 --retry-delay 2 -o "$BUILD_NODE_TMP_DIR/$archive" "$base_url/$archive"
	curl -fsSL --retry 3 --retry-delay 2 -o "$BUILD_NODE_TMP_DIR/SHASUMS256.txt" "$base_url/SHASUMS256.txt"

	if ! checksum_line="$(awk -v archive="$archive" '$2 == archive { print; count++ } END { exit count != 1 }' "$BUILD_NODE_TMP_DIR/SHASUMS256.txt")"; then
		die "Node checksum entry not found or not unique for $archive"
	fi
	if ! (cd "$BUILD_NODE_TMP_DIR" && printf '%s\n' "$checksum_line" | sha256sum --check --status -); then
		die "Node archive checksum verification failed for $archive"
	fi

	mkdir "$BUILD_NODE_TMP_DIR/extracted"
	tar --no-same-owner -xJ -f "$BUILD_NODE_TMP_DIR/$archive" -C "$BUILD_NODE_TMP_DIR/extracted"
	local extracted="$BUILD_NODE_TMP_DIR/extracted/node-v${version}-linux-${node_arch}"
	[ -x "$extracted/bin/node" ] || die "Node archive did not contain the expected executable"
	"$extracted/bin/node" --version | grep -qx "v$version" || die "Node archive contained an unexpected version"

	rm -rf "$NODE_DIR"
	mv "$extracted" "$NODE_DIR"
	BUILD_NODE_TMP_DIR=""
}

node_ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	local installed_version
	installed_version="$(corepack --version 2>/dev/null || true)"
	if [ "$installed_version" != "$COREPACK_VERSION" ]; then
		log "Installing Corepack ${COREPACK_VERSION}..."
		npm install --global --prefix "$NODE_DIR" "corepack@${COREPACK_VERSION}"
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

install_node_dependencies() {
	local version
	local node_arch

	version="$(node_resolve_version)"
	node_assert_exact_version "$version"
	node_arch="$(node_architecture)"
	node_configure_paths "$version" "$node_arch"

	if node_is_installed "$version"; then
		log "Using cached Node v${version}..."
	else
		node_install "$version" "$node_arch"
	fi

	node_ensure_corepack
	log "Node: $(node --version)"
	log "npm:  $(npm --version)"
}

node_enable_toolchain() {
	local version
	local node_arch

	version="$(node_resolve_version)"
	node_assert_exact_version "$version"
	node_arch="$(node_architecture)"
	node_configure_paths "$version" "$node_arch"

	export PATH="$NODE_DIR/bin:$PATH"
	command -v node >/dev/null 2>&1 || die "node not found"
	command -v npm >/dev/null 2>&1 || die "npm not found"
	node_is_installed "$version" || die "Cached Node installation is missing or has the wrong version"
	node_ensure_corepack
}

configure_build_cache
