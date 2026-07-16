#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

log() {
	echo "$LOG_PREFIX $*"
}

install_dependencies_and_build() {
	if [ -f pnpm-lock.yaml ]; then
		corepack pnpm install --frozen-lockfile
		corepack pnpm run build
	elif [ -f yarn.lock ]; then
		corepack yarn install --frozen-lockfile
		corepack yarn run build
	elif [ -f package-lock.json ]; then
		npm ci --include=optional
		npm run build
	else
		echo "$LOG_PREFIX No Node lockfile found. Commit package-lock.json, pnpm-lock.yaml, or yarn.lock." >&2
		exit 1
	fi
}

prepare_standalone_output() {
	if [ ! -f .next/standalone/server.js ]; then
		cat >&2 <<'EOF'
[bonesdeploy] Next.js did not produce .next/standalone/server.js.

Configure standalone output in next.config.js, next.config.mjs, or next.config.ts:

  output: "standalone",
EOF
		exit 1
	fi

	mkdir -p .next/standalone/.next
	cp -R .next/static .next/standalone/.next/

	if [ -d public ]; then
		cp -R public .next/standalone/
	fi
}

main() {
	if [ ! -f package.json ]; then
		echo "$LOG_PREFIX package.json not found; this is not a Next.js project." >&2
		exit 1
	fi

	export PATH="$PROJECT_ROOT/build/node/bin:$PATH"

	install_dependencies_and_build
	prepare_standalone_output

	log "Next.js build complete."
}

main "$@"
