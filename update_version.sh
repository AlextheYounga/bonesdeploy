#!/usr/bin/env bash

set -euo pipefail

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
cd "$repo_root"

# Get new version from args
new_rust_version=${1:-}
new_python_version=${2:-}

if [ -z "$new_rust_version" ] || [ -z "$new_python_version" ]; then
	echo "Usage: $0 <new_rust_version> <new_python_version>"
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

update_python() {
	infra_cargo_file=crates/bonesdeploy/Cargo.toml
	pyproject_toml=crates/bonesinfra/python/pyproject.toml
	echo "Updating $infra_cargo_file bonesinfra dependency to $new_python_version"
	update_dependency_version "$infra_cargo_file" bonesinfra "$new_python_version"
	echo "Updating crates/bonesinfra/Cargo.toml package version to $new_python_version"
	update_package_version crates/bonesinfra/Cargo.toml "$new_python_version"

	echo "Updating $pyproject_toml package version to $new_python_version"
	update_package_version "$pyproject_toml" "$new_python_version"

	(
		cd crates/bonesinfra/python
		uv lock
		uv build --wheel --out-dir ../assets
		shopt -s nullglob
		wheel=(../assets/bonesinfra-*.whl)
		if [ "${#wheel[@]}" -ne 1 ]; then
			echo "Expected exactly one generated BonesInfra wheel" >&2
			exit 1
		fi
		mv "${wheel[0]}" ../assets/bonesinfra.whl
	)
}

update_cargo
update_python
