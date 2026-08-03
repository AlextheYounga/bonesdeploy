#!/usr/bin/env bash
set -euo pipefail

: "${BONESDEPLOY_HOST:?missing BONESDEPLOY_HOST}"
: "${BONESDEPLOY_PORT:?missing BONESDEPLOY_PORT}"
: "${BONESDEPLOY_BONES_REPO:?missing BONESDEPLOY_BONES_REPO}"

if [ "$BONESDEPLOY_PORT" = "22" ]; then
	remote_url="root@${BONESDEPLOY_HOST}:${BONESDEPLOY_BONES_REPO}"
else
	remote_url="ssh://root@${BONESDEPLOY_HOST}:${BONESDEPLOY_PORT}${BONESDEPLOY_BONES_REPO}"
fi

if git -C .bones remote get-url origin >/dev/null 2>&1; then
	git -C .bones remote set-url origin "$remote_url"
else
	git -C .bones remote add origin "$remote_url"
fi
