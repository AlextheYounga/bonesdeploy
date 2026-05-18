#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
workspace_root="$(cd "$script_dir/../.." && pwd)"
compose_file="$workspace_root/docker/docker-compose.yml"

docker compose -f "$compose_file" down --remove-orphans
docker compose -f "$compose_file" up -d

cargo test --manifest-path "$workspace_root/Cargo.toml" -p bonesdeploy-e2e-tests -- --ignored --nocapture "$@"
