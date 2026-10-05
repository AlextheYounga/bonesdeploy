#!/usr/bin/env bash
set -euo pipefail

tool="${1:?Runtime tool is required}"
version="${2:?Runtime version is required}"
link_path="${3:?Site runtime link is required}"
export tool version link_path

if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
	printf '%s\n' "Runtime version must be an exact X.Y.Z version: $version" >&2
	exit 1
fi

: "${MISE_BINARY:?}"
: "${MISE_BINARY_URL:?}"
: "${MISE_BINARY_SHA256:?}"
: "${MISE_DATA_DIR:?}"
: "${MISE_RUNTIME_STORE:?}"
: "${MISE_CACHE_DIR:?}"
: "${MISE_CONFIG_FILE:?}"

install -d -o root -g root -m 0755 "$(dirname "$MISE_BINARY")" "$MISE_DATA_DIR" "$MISE_RUNTIME_STORE" "$MISE_CACHE_DIR" "$(dirname "$MISE_CONFIG_FILE")"
lock_file="$MISE_DATA_DIR/install.lock"

flock "$lock_file" bash -ceu '
	if ! printf "%s  %s\\n" "$MISE_BINARY_SHA256" "$MISE_BINARY" | sha256sum --check --status -; then
		temporary_binary="$(mktemp "${MISE_BINARY}.tmp.XXXXXX")"
		trap "rm -f -- \"$temporary_binary\"" EXIT
		curl --fail --location --retry 3 --silent --show-error --output "$temporary_binary" "$MISE_BINARY_URL"
		printf "%s  %s\\n" "$MISE_BINARY_SHA256" "$temporary_binary" | sha256sum --check --status -
		install -o root -g root -m 0755 "$temporary_binary" "$MISE_BINARY"
	fi

	"$MISE_BINARY" --version | grep -Eq "^${MISE_VERSION}( |$)"
	cd /
	env -i \
		HOME=/root \
		PATH=/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin \
		MISE_SYSTEM_DATA_DIR="$MISE_DATA_DIR" \
		MISE_SYSTEM_INSTALLS_DIR="$MISE_RUNTIME_STORE" \
		MISE_CACHE_DIR="$MISE_CACHE_DIR" \
		MISE_SYSTEM_CONFIG_FILE="$MISE_CONFIG_FILE" \
		MISE_GLOBAL_CONFIG_ROOT="$(dirname "$MISE_CONFIG_FILE")" \
		MISE_GLOBAL_CONFIG_FILE="$MISE_CONFIG_FILE" \
		MISE_CONFIG_FILE="$MISE_CONFIG_FILE" \
		MISE_ALL_COMPILE=false \
		MISE_NODE_COMPILE=false \
		MISE_PYTHON_COMPILE=false \
		MISE_RUBY_COMPILE=false \
		MISE_REGISTRY_FLOATING=false \
		MISE_AUTO_UPDATE=false \
		MISE_NO_HOOKS=true \
		MISE_SAFE=true \
		MISE_OVERRIDE_CONFIG_FILENAMES=.bonesdeploy-mise-disabled \
		MISE_OVERRIDE_TOOL_VERSIONS_FILENAMES=none \
		MISE_SLSA=true \
		MISE_GITHUB_ATTESTATIONS=true \
		"$MISE_BINARY" install --system "$tool@$version"

	runtime_root="$MISE_RUNTIME_STORE/$tool/$version"
	[[ -x "$runtime_root/bin/$tool" ]]
	chown -R root:root "$runtime_root"
	chmod -R go-w "$runtime_root"
	install -d -o root -g root -m 0755 "$(dirname "$link_path")"
	temporary_link="${link_path}.tmp.$$"
	ln -sfn "$runtime_root" "$temporary_link"
	mv -Tf "$temporary_link" "$link_path"
'
