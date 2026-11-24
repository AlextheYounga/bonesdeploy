#!/usr/bin/env bash
set -euo pipefail

if output=$(nginx -t 2>&1); then
	status=0
else
	status=$?
fi
printf '%s\n' "$output"
[ "$status" -eq 0 ] || exit "$status"

case "$output" in
*"conflicting server name"*) exit 1 ;;
esac
