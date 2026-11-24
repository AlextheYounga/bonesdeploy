#!/usr/bin/env bash
set -euo pipefail

available=$1
enabled=$2

ln -sfn -- "$available" "$enabled"
if output=$(nginx -t 2>&1); then
	status=0
else
	status=$?
fi
printf '%s\n' "$output"

case "$output" in
*"conflicting server name"*) status=1 ;;
esac

if [ "$status" -ne 0 ]; then
	rm -f -- "$enabled"
	exit "$status"
fi
