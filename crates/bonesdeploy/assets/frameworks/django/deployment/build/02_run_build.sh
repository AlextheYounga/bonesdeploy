#!/usr/bin/env bash

set -Eeuo pipefail

source /workspace/deployment/functions.sh

main() {
	if [ ! -f manage.py ]; then
		log "manage.py not found; skipping Django build."
		exit 0
	fi

	local python_version="${BONES_RUNTIME_PYTHON_VERSION:?BONES_RUNTIME_PYTHON_VERSION is required}"
	local python_binary="python${python_version}"
	local packages_dir=".python-packages"
	local wrapper_dir=".venv/bin"
	local production_python="/opt/bonesdeploy/python/${python_version}/bin/python${python_version}"

	command -v "$python_binary" >/dev/null 2>&1 || die "$python_binary not found"
	rm -rf "$packages_dir" .venv
	mkdir -p "$packages_dir" "$wrapper_dir"

	if [ -f requirements.txt ]; then
		log "Installing Django Python dependencies..."
		"$python_binary" -m pip install --target "$packages_dir" -r requirements.txt --quiet
	else
		log "No requirements.txt found; skipping Python dependency install."
	fi

	cat >"$wrapper_dir/python" <<EOF
#!/usr/bin/env bash
set -Eeuo pipefail
release_root=\"\$(cd -- \"\$(dirname -- \"\${BASH_SOURCE[0]}\")/../..\" && pwd)\"
export PYTHONPATH=\"\$release_root/.python-packages\${PYTHONPATH:+:\$PYTHONPATH}\"
exec "$production_python" "\$@"
EOF
	cat >"$wrapper_dir/gunicorn" <<EOF
#!/usr/bin/env bash
set -Eeuo pipefail
release_root=\"\$(cd -- \"\$(dirname -- \"\${BASH_SOURCE[0]}\")/../..\" && pwd)\"
export PYTHONPATH=\"\$release_root/.python-packages\${PYTHONPATH:+:\$PYTHONPATH}\"
exec "$production_python" -m gunicorn "\$@"
EOF
	chmod 0755 "$wrapper_dir/python" "$wrapper_dir/gunicorn"

	# The release owns dependency files and wrappers; no builder interpreter is shipped.
	rm -rf deployment/build
	log "Django build complete."
}

main "$@"
