#!/usr/bin/env bash

# Upload a consistent SQLite snapshot into an existing BonesDeploy site's shared directory.
set -euo pipefail

usage() {
	cat <<'EOF'
Usage: scripts/push-sqlite.sh DATABASE SITE HOST [SHARED_PATH]

DATABASE     Local SQLite database to upload.
SITE         Existing BonesDeploy site name. The site runtime user and group have this name.
HOST         Deployment server hostname.
SHARED_PATH  Relative destination below shared/ (default: the database filename).

The script connects as root, requires sqlite3, and installs the database as SITE:SITE
with mode 0640. Configure SSH with the appropriate root key before running it.
EOF
}

if [ "$#" -lt 3 ] || [ "$#" -gt 4 ]; then
	usage >&2
	exit 2
fi

database=$1
site=$2
host=$3
shared_path=${4:-$(basename "$database")}

if [ ! -f "$database" ]; then
	echo "Database is not a regular file: $database" >&2
	exit 1
fi

if ! command -v sqlite3 >/dev/null; then
	echo "sqlite3 is required to create a consistent database snapshot." >&2
	exit 1
fi

if [[ ! $site =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
	echo "Invalid site name: $site" >&2
	exit 2
fi

if [[ ! $host =~ ^[A-Za-z0-9][A-Za-z0-9.-]*$ ]]; then
	echo "Invalid host name: $host" >&2
	exit 2
fi

if [[ ! $shared_path =~ ^[A-Za-z0-9][A-Za-z0-9._/-]*$ ]] || [[ $shared_path == *'//' ]] ||
	[[ $shared_path == '..' ]] || [[ $shared_path == */../* ]] || [[ $shared_path == ../* ]]; then
	echo "SHARED_PATH must be a relative path without '..': $shared_path" >&2
	exit 2
fi

snapshot=$(mktemp /tmp/bonesdeploy-sqlite.XXXXXX)
remote_snapshot=

cleanup() {
	rm -f "$snapshot"
	if [ -n "$remote_snapshot" ]; then
		ssh "root@$host" "rm -f '$remote_snapshot'" || true
	fi
}
trap cleanup EXIT

sqlite3 "$database" ".backup $snapshot"

remote_snapshot=$(ssh "root@$host" 'mktemp /tmp/bonesdeploy-sqlite.XXXXXX')
scp "$snapshot" "root@$host:$remote_snapshot"

target="/srv/sites/$site/shared/$shared_path"
ssh "root@$host" "set -eu
target='$target'
snapshot='$remote_snapshot'
directory=\$(dirname \"\$target\")
test -d \"\$directory\"
temporary=\$(mktemp \"\$directory/.sqlite-upload.XXXXXX\")
trap 'rm -f \"\$temporary\" \"\$snapshot\"' EXIT
install -o '$site' -g '$site' -m 0640 \"\$snapshot\" \"\$temporary\"
mv -f \"\$temporary\" \"\$target\"
trap - EXIT"
remote_snapshot=

echo "Installed $shared_path in $site's shared directory on $host."
