#!/usr/bin/env bash

# Export a remote BonesDeploy site's shared directory to this machine.
set -euo pipefail

usage() {
	cat <<'EOF'
Usage: scripts/export-shared.sh SITE HOST [OUTPUT]

SITE    Existing BonesDeploy site name.
HOST    Deployment server hostname or IPv4 address.
OUTPUT  Archive path (default: SITE-shared-YYYYMMDD_HHMMSS.zip).

The script connects as root so protected files such as shared/.env can be read.
Configure SSH with the appropriate root key. The archive contains secrets and
must be handled as sensitive data.
EOF
}

if [ "$#" -lt 2 ] || [ "$#" -gt 3 ]; then
	usage >&2
	exit 2
fi

site=$1
host=$2
output=${3:-"$site-shared-$(date -u +%Y%m%d_%H%M%S).zip"}
site_root="/srv/sites/$site"

if [[ ! $site =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
	echo "Invalid site name: $site" >&2
	exit 2
fi

if [[ ! $host =~ ^[A-Za-z0-9][A-Za-z0-9.-]*$ ]]; then
	echo "Invalid host name or IPv4 address: $host" >&2
	exit 2
fi

if ! command -v ssh >/dev/null; then
	echo "ssh is required to export the remote shared directory." >&2
	exit 1
fi

output_directory=$(dirname -- "$output")
if [ ! -d "$output_directory" ]; then
	echo "Output directory does not exist: $output_directory" >&2
	exit 1
fi

umask 077
temporary=$(mktemp "$output_directory/.shared-export.XXXXXX")
cleanup() {
	rm -f "$temporary"
}
trap cleanup EXIT

ssh "root@$host" "cd '$site_root' && test -d shared && exec zip -q -r -y - shared" >"$temporary"
ln -- "$temporary" "$output"
rm "$temporary"
trap - EXIT

echo "Shared export saved to $output"
