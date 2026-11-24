#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cd "$repo_root"

# Get new version from args
new_rust_version=${1:-}
new_infra_version=${2:-}

if [ -z "$new_rust_version" ] || [ -z "$new_infra_version" ]; then
	echo "Usage: $0 <new_rust_version> <new_infra_version>"
	exit 1
fi

update_package_version() {
	local file=$1
	local version=$2

	sed -i -E "0,/^version = \"[^\"]+\"$/s//version = \"$version\"/" "$file"
}

update_dependency_version() {
	local file=$1
	local dependency=$2
	local version=$3

	sed -i -E "s|^$dependency = \{ version = \"[^\"]+\"|$dependency = { version = \"$version\"|" "$file"
}

update_cargo() {
	for file in crates/bonesdeploy/Cargo.toml crates/bonesdeploy-core/Cargo.toml crates/bonesremote/Cargo.toml; do
		echo "Updating $file package version to $new_rust_version"
		update_package_version "$file" "$new_rust_version"
	done

	update_dependency_version crates/bonesdeploy/Cargo.toml bonesdeploy-core "$new_rust_version"
	update_dependency_version crates/bonesremote/Cargo.toml bonesdeploy-core "$new_rust_version"
	update_dependency_version crates/bonesinfra/Cargo.toml bonesdeploy-core "$new_rust_version"
}

update_infra() {
	infra_cargo_file=crates/bonesdeploy/Cargo.toml
	echo "Updating $infra_cargo_file bonesinfra dependency to $new_infra_version"
	update_dependency_version "$infra_cargo_file" bonesinfra "$new_infra_version"
	echo "Updating crates/bonesinfra/Cargo.toml package version to $new_infra_version"
	update_package_version crates/bonesinfra/Cargo.toml "$new_infra_version"
}

update_cargo
update_infra
