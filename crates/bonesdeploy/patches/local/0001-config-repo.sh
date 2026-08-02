#!/usr/bin/env bash
set -euo pipefail

: "${BONESDEPLOY_SITE:?missing BONESDEPLOY_SITE}"
: "${BONESDEPLOY_HOST:?missing BONESDEPLOY_HOST}"
: "${BONESDEPLOY_PORT:?missing BONESDEPLOY_PORT}"
: "${BONESDEPLOY_BONES_REPO:?missing BONESDEPLOY_BONES_REPO}"

bones_dir=".bones"
printf '%s\n' '**/.env' >"$bones_dir/.gitignore"

if ! git -C "$bones_dir" rev-parse --git-dir >/dev/null 2>&1; then
	git -C "$bones_dir" init --initial-branch master
fi

if [ "$BONESDEPLOY_PORT" = "22" ]; then
	remote_url="git@${BONESDEPLOY_HOST}:${BONESDEPLOY_BONES_REPO}"
else
	remote_url="ssh://git@${BONESDEPLOY_HOST}:${BONESDEPLOY_PORT}${BONESDEPLOY_BONES_REPO}"
fi
if git -C "$bones_dir" remote get-url origin >/dev/null 2>&1; then
	actual_url=$(git -C "$bones_dir" remote get-url origin)
	if [ "$actual_url" != "$remote_url" ]; then
		printf 'origin points to %s, expected %s\n' "$actual_url" "$remote_url" >&2
		exit 1
	fi
else
	git -C "$bones_dir" remote add origin "$remote_url"
fi
