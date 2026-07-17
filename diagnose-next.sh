#!/usr/bin/env bash

set -uo pipefail

usage() {
	cat <<'EOF'
Usage: ./diagnose-next.sh SITE [PUBLIC_URL]

Collects read-only diagnostics for a BonesDeploy Next.js site.
Run as root, or as a user with passwordless sudo, for complete output.

Examples:
  sudo ./diagnose-next.sh mysite
  sudo ./diagnose-next.sh mysite https://test.example.com/
  ./diagnose-next.sh --self-test
EOF
}

valid_site() {
	[[ "$1" =~ ^[a-z0-9][a-z0-9-]*$ ]]
}

if [[ "${1:-}" == "--self-test" ]]; then
	valid_site "next-test-1" && ! valid_site "../bad" && ! valid_site "Bad Site"
	exit
fi

SITE="${1:-}"
PUBLIC_URL="${2:-}"

if ! valid_site "$SITE"; then
	usage >&2
	exit 2
fi

if [[ -n "$PUBLIC_URL" && ! "$PUBLIC_URL" =~ ^https?:// ]]; then
	echo "PUBLIC_URL must start with http:// or https://" >&2
	exit 2
fi

if [[ $EUID -eq 0 ]]; then
	SUDO=()
elif sudo -n true 2>/dev/null; then
	SUDO=(sudo -n)
else
	SUDO=()
	echo "WARNING: not root and passwordless sudo is unavailable; some checks will fail." >&2
fi

NEXT_SERVICE="${SITE}-next.service"
NGINX_SERVICE="${SITE}-nginx.service"
PROJECT_ROOT="/srv/sites/${SITE}"
CURRENT="${PROJECT_ROOT}/current"
SITE_NGINX_CONFIG="/srv/conf/${SITE}/nginx.conf"
NGINX_SOCKET="/run/${SITE}/nginx/nginx.sock"

section() {
	printf '\n===== %s =====\n' "$1"
}

run() {
	printf '\n$'
	printf ' %q' "$@"
	printf '\n'
	"$@" 2>&1
	local status=$?
	if ((status != 0)); then
		printf '[exit %d]\n' "$status"
	fi
	return 0
}

http_probe() {
	local label="$1"
	shift
	local body
	body="$(mktemp)" || return

	section "$label"
	printf '$ curl'
	printf ' %q' "$@"
	printf '\n'
	curl --silent --show-error --max-time 10 --dump-header - --output "$body" "$@" 2>&1
	local status=$?
	printf '\n--- body (first 4000 bytes) ---\n'
	head -c 4000 "$body"
	printf '\n'
	rm -f "$body"
	if ((status != 0)); then
		printf '[curl exit %d]\n' "$status"
	fi
}

process_summary() {
	printf '\n$ ps -eo pid,ppid,user,%%cpu,%%mem,rss,stat,lstart,comm --sort=-rss | head -n 26\n'
	ps -eo pid,ppid,user,%cpu,%mem,rss,stat,lstart,comm --sort=-rss 2>&1 | head -n 26
}

section "Collection notes"
echo "Read-only checks for site: $SITE"
echo "This script does not read runtime.env, .env, or process environments."
echo "Application logs can still contain values logged by the application; review before sharing."

section "Host and resource pressure"
run date --iso-8601=seconds
run hostnamectl
run uptime
run free -h
run swapon --show
run df -h "$PROJECT_ROOT" / /tmp
process_summary

section "BonesDeploy state"
run "${SUDO[@]}" bonesremote version
run "${SUDO[@]}" bonesremote doctor --site "$SITE"
run "${SUDO[@]}" bonesremote status --site "$SITE"
run "${SUDO[@]}" bonesremote release list --site "$SITE"
run "${SUDO[@]}" readlink -f "$CURRENT"
run "${SUDO[@]}" stat "$PROJECT_ROOT" "$CURRENT" "$CURRENT/.next/standalone/server.js"
run "${SUDO[@]}" /usr/bin/node --version

section "Service state"
for service in "$NEXT_SERVICE" "$NGINX_SERVICE"; do
	run "${SUDO[@]}" systemctl status "$service" --no-pager --full
	run "${SUDO[@]}" systemctl show "$service" --no-pager \
		--property=LoadState,ActiveState,SubState,Result,MainPID,ExecMainCode,ExecMainStatus,NRestarts,TasksCurrent,MemoryCurrent,MemoryPeak
done

section "Listening ports and sockets"
run "${SUDO[@]}" ss -ltnp
run "${SUDO[@]}" stat "$NGINX_SOCKET"

section "Nginx validation and logs"
run "${SUDO[@]}" nginx -t
run "${SUDO[@]}" nginx -t -c "$SITE_NGINX_CONFIG"
run "${SUDO[@]}" tail -n 100 "/run/${SITE}/nginx/error.log"
run "${SUDO[@]}" tail -n 40 "/run/${SITE}/nginx/access.log"

NEXT_PORT="$("${SUDO[@]}" systemctl show "$NEXT_SERVICE" --property=ExecStart --value 2>/dev/null |
	sed -n 's/.*PORT=\([0-9][0-9]*\).*/\1/p' |
	head -n 1)"
NEXT_PORT="${NEXT_PORT:-3100}"
http_probe "Direct Next.js probe" "http://127.0.0.1:${NEXT_PORT}/"
http_probe "Per-site nginx socket probe" --unix-socket "$NGINX_SOCKET" http://localhost/

if [[ -n "$PUBLIC_URL" ]]; then
	http_probe "Public URL probe" "$PUBLIC_URL"
fi

section "Recent service logs"
run "${SUDO[@]}" journalctl -u "$NEXT_SERVICE" -u "$NGINX_SERVICE" -n 200 --no-pager -o short-iso

section "OOM and AppArmor evidence from this boot"
run "${SUDO[@]}" journalctl -k -b --no-pager --grep='oom|out of memory|killed process|apparmor.*denied'

section "End"
echo "Diagnostics complete."
