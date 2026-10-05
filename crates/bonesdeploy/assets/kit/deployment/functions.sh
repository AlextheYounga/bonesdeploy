#!/usr/bin/env bash

MISE_VERSION="2026.10.0"
MISE_BINARY_URL="https://github.com/jdx/mise/releases/download/v2026.10.0/mise-v2026.10.0-linux-x64"
MISE_BINARY_SHA256="57ced973f968b8fbab07aa8e32bd7077d4a357e200a22356d98963c723c6de0a"
COREPACK_COMPAT_VERSION="0.31.0"
COREPACK_MODERN_VERSION="0.34.5"

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
		"$BUILD_CACHE_DIR/mise/bin" \
		"$BUILD_CACHE_DIR/mise/data" \
		"$BUILD_CACHE_DIR/mise/cache" \
		"$BUILD_CACHE_DIR/mise/config"; do
		mkdir -p "$directory"
	done
	: >"$BUILD_CACHE_DIR/mise/config/config.toml"

	export COREPACK_HOME="$BUILD_CACHE_DIR/corepack"
	export NPM_CONFIG_CACHE="$BUILD_CACHE_DIR/npm"
	export PNPM_STORE_DIR="$BUILD_CACHE_DIR/pnpm"
	export YARN_CACHE_FOLDER="$BUILD_CACHE_DIR/yarn/cache"
	export YARN_GLOBAL_FOLDER="$BUILD_CACHE_DIR/yarn/global"
	export COMPOSER_CACHE_DIR="$BUILD_CACHE_DIR/composer"
	export BUNDLE_USER_CACHE="$BUILD_CACHE_DIR/bundler"
	export MISE_BIN="$BUILD_CACHE_DIR/mise/bin/mise"
	export MISE_DATA_DIR="$BUILD_CACHE_DIR/mise/data"
	export MISE_INSTALLS_DIR="$MISE_DATA_DIR/installs"
	export MISE_CACHE_DIR="$BUILD_CACHE_DIR/mise/cache"
	export MISE_CONFIG_DIR="$BUILD_CACHE_DIR/mise/config"
	export MISE_CONFIG_FILE="$MISE_CONFIG_DIR/config.toml"
	export MISE_GLOBAL_CONFIG_ROOT="$MISE_CONFIG_DIR"
	export MISE_GLOBAL_CONFIG_FILE="$MISE_CONFIG_FILE"
	export MISE_SYSTEM_CONFIG_FILE="$MISE_CONFIG_FILE"
	export MISE_ALL_COMPILE="false"
	export MISE_NODE_COMPILE="false"
	export MISE_PYTHON_COMPILE="false"
	export MISE_RUBY_COMPILE="false"
	export MISE_REGISTRY_FLOATING="false"
	export MISE_AUTO_UPDATE="false"
	export MISE_NO_HOOKS="true"
	export MISE_SAFE="true"
	export MISE_OVERRIDE_CONFIG_FILENAMES=".bonesdeploy-mise-disabled"
	export MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES="none"
}

mise_is_installed() {
	[ -x "$MISE_BIN" ] && [ "$("$MISE_BIN" --version | awk '{ print $1 }')" = "$MISE_VERSION" ]
}

mise_bootstrap() {
	: "${BUILD_CACHE_DIR:?BUILD_CACHE_DIR must be set by bonesremote}"

	if mise_is_installed; then
		return
	fi

	local temporary_binary
	temporary_binary="$(mktemp "$BUILD_CACHE_DIR/mise/bin/.mise.XXXXXX")"
	log "Installing mise ${MISE_VERSION}..."
	curl -fsSL --retry 3 --retry-delay 2 -o "$temporary_binary" "$MISE_BINARY_URL"
	printf '%s  %s\n' "$MISE_BINARY_SHA256" "$temporary_binary" | sha256sum --check --status - || die "mise binary checksum verification failed"
	chmod 0755 "$temporary_binary"
	"$temporary_binary" --version | awk '{ print $1 }' | grep -qx "$MISE_VERSION" || die "mise binary contained an unexpected version"
	mv -f "$temporary_binary" "$MISE_BIN"
}

mise_install_runtime() {
	local tool="$1"
	local version="$2"

	mise_bootstrap
	"$MISE_BIN" install "$tool@$version"
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

node_configure_paths() {
	local version="$1"

	: "${BUILD_CACHE_DIR:?BUILD_CACHE_DIR must be set by bonesremote}"

	NODE_DIR="$MISE_INSTALLS_DIR/node/$version"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR NODE_BIN
}

node_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] && "$NODE_BIN" --version | grep -qx "v$version"
}

node_corepack_version() {
	local node_version major minor
	node_version="$(node --version)"
	node_version="${node_version#v}"
	major="${node_version%%.*}"
	minor="${node_version#*.}"
	minor="${minor%%.*}"

	# Corepack 0.25+ requires Node >= 18.17 / >= 20.10 (uses URL.canParse).
	if [ "$major" -lt 18 ] ||
		{ [ "$major" -eq 18 ] && [ "$minor" -lt 17 ]; } ||
		{ [ "$major" -eq 20 ] && [ "$minor" -lt 10 ]; }; then
		echo "0.24.1"
	elif [ "$major" -eq 18 ]; then
		echo "$COREPACK_COMPAT_VERSION"
	else
		echo "$COREPACK_MODERN_VERSION"
	fi
}

node_ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	local target_version installed_version
	target_version="$(node_corepack_version)"
	# Corepack stores its resolved executable path in the package-manager cache.
	# Keep caches versioned so an upgrade cannot reuse incompatible metadata.
	export COREPACK_HOME="$BUILD_CACHE_DIR/corepack/$target_version"
	mkdir -p "$COREPACK_HOME"
	installed_version="$(corepack --version 2>/dev/null || true)"
	if [ "$installed_version" != "$target_version" ]; then
		log "Installing Corepack ${target_version}..."
		npm install --global --prefix "$NODE_DIR" "corepack@${target_version}"
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

install_node_dependencies() {
	local version

	version="$(node_resolve_version)"
	node_assert_exact_version "$version"
	node_configure_paths "$version"

	if node_is_installed "$version"; then
		log "Using cached Node v${version}..."
	else
		mise_install_runtime node "$version"
		node_is_installed "$version" || die "mise did not install Node v${version}"
	fi

	node_ensure_corepack
	log "Node: $(node --version)"
	log "npm:  $(npm --version)"
}

node_enable_toolchain() {
	local version

	version="$(node_resolve_version)"
	node_assert_exact_version "$version"
	node_configure_paths "$version"

	export PATH="$NODE_DIR/bin:$PATH"
	command -v node >/dev/null 2>&1 || die "node not found"
	command -v npm >/dev/null 2>&1 || die "npm not found"
	node_is_installed "$version" || die "Cached Node installation is missing or has the wrong version"
	node_ensure_corepack
}

ruby_resolve_version() {
	case "${RUBY_VERSION:-}" in
	3.2 | 3.2.8) echo "3.2.8" ;;
	3.3 | 3.3.8) echo "3.3.8" ;;
	3.4 | 3.4.8) echo "3.4.8" ;;
	*) die "Ruby requires one of 3.2.8, 3.3.8, or 3.4.8. Set RUBY_VERSION in .env.build." ;;
	esac
}

ruby_configure_paths() {
	local version="$1"

	: "${BUILD_CACHE_DIR:?BUILD_CACHE_DIR must be set by bonesremote}"

	RUBY_DIR="$MISE_INSTALLS_DIR/ruby/$version"
	RUBY_BIN="$RUBY_DIR/bin/ruby"

	export RUBY_DIR RUBY_BIN
}

ruby_is_installed() {
	local version="$1"

	[ -x "$RUBY_BIN" ] && "$RUBY_BIN" --version | grep -q "^ruby $version "
}

ruby_enable_toolchain() {
	local version

	version="$(ruby_resolve_version)"
	ruby_configure_paths "$version"
	if ! ruby_is_installed "$version"; then
		mise_install_runtime ruby "$version"
		ruby_is_installed "$version" || die "mise did not install Ruby $version"
	fi

	export PATH="$RUBY_DIR/bin:$PATH"
	command -v ruby >/dev/null 2>&1 || die "Ruby not found"
	command -v bundle >/dev/null 2>&1 || die "Bundler not found"
	ruby_is_installed "$version" || die "Cached Ruby installation is missing or has the wrong version"
	log "Ruby: $(ruby --version)"
}

python_assert_exact_version() {
	local version="$1"

	[[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Python requires an exact pinned version. Set PYTHON_VERSION in project configuration."
}

python_configure_paths() {
	local version="$1"

	: "${BUILD_CACHE_DIR:?BUILD_CACHE_DIR must be set by bonesremote}"

	PYTHON_DIR="$MISE_INSTALLS_DIR/python/$version"
	PYTHON_BIN="$PYTHON_DIR/bin/python"

	export PYTHON_DIR PYTHON_BIN
}

python_is_installed() {
	local version="$1"

	[ -x "$PYTHON_BIN" ] && "$PYTHON_BIN" --version | grep -qx "Python $version"
}

python_enable_toolchain() {
	local version="${PYTHON_VERSION:-}"

	python_assert_exact_version "$version"
	python_configure_paths "$version"
	if ! python_is_installed "$version"; then
		mise_install_runtime python "$version"
		python_is_installed "$version" || die "mise did not install Python $version"
	fi

	export PATH="$PYTHON_DIR/bin:$PATH"
	command -v python >/dev/null 2>&1 || die "Python not found"
	command -v pip >/dev/null 2>&1 || die "pip not found"
	python_is_installed "$version" || die "Cached Python installation is missing or has the wrong version"
	log "Python: $(python --version)"
}

configure_build_cache
