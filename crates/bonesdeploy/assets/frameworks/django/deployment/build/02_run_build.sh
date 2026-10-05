#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

install_application_packages() {
	apt-get update
	DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
		build-essential \
		default-libmysqlclient-dev \
		libpq-dev \
		libsqlite3-dev \
		pkg-config
}

main() {
	if [ ! -f manage.py ]; then
		log "manage.py not found; skipping Django build."
		exit 0
	fi

	local packages_dir=".python-packages"
	local wrapper_dir=".venv/bin"

	python_enable_toolchain
	install_application_packages
	rm -rf "$packages_dir" .venv
	mkdir -p "$packages_dir" "$wrapper_dir"

	if [ -f requirements.txt ]; then
		log "Installing Django Python dependencies..."
		python -m pip install \
			--disable-pip-version-check \
			--no-cache-dir \
			--target "$packages_dir" \
			-r requirements.txt
	else
		log "No requirements.txt found; skipping Python dependency install."
	fi
	python --version >.bonesdeploy-python-runtime

	cat >"$wrapper_dir/python" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
release_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
site_root="$(cd -- "$release_root/../.." && pwd)"
export PYTHONPATH="$release_root/.python-packages${PYTHONPATH:+:$PYTHONPATH}"
exec "$site_root/.bonesdeploy/runtimes/python" "$@"
EOF
	cat >"$wrapper_dir/gunicorn" <<'EOF'
#!/usr/bin/env bash
set -Eeuo pipefail
release_root="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/../.." && pwd)"
site_root="$(cd -- "$release_root/../.." && pwd)"
export PYTHONPATH="$release_root/.python-packages${PYTHONPATH:+:$PYTHONPATH}"
exec "$site_root/.bonesdeploy/runtimes/python" -m gunicorn "$@"
EOF
	chmod 0755 "$wrapper_dir/python" "$wrapper_dir/gunicorn"

	rm -rf deployment/build
	log "Django build complete."
}

main "$@"
