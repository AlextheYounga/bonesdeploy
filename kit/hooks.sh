bones_init_remote_context() {
	local git_dir_input="${1:-${GIT_DIR:-.}}"
	GIT_DIR=$(cd "$git_dir_input" && pwd)
	BONES_TOML="$GIT_DIR/bones/bones.toml"
}

bones_should_deploy_on_push() {
	if [ "${BONES_FORCE_DEPLOY:-0}" = "1" ]; then
		return 0
	fi

	local deploy_on_push
	deploy_on_push=$(grep -E '^deploy_on_push\s*=' "$BONES_TOML" | head -1 | sed 's/#.*$//' | sed 's/.*=\s*//' | tr -d '[:space:]')

	if [ -z "$deploy_on_push" ]; then
		return 0
	fi

	if [ "$deploy_on_push" = "false" ]; then
		return 1
	fi

	return 0
}

	bones_run_doctor_remote() {
		echo "[bonesdeploy] Running remote doctor..."

		if ! bonesremote doctor --config "$BONES_TOML"; then
			echo "[bonesdeploy] Remote doctor reported issues. Push rejected."
			exit 1
		fi

	echo "[bonesdeploy] Doctor passed. Staging release..."
}

bones_stage_release() {
	if ! sudo bonesremote release stage --config "$BONES_TOML"; then
		echo "[bonesdeploy] release stage failed. Push rejected."
		exit 1
	fi

	echo "[bonesdeploy] Release staged."
}

	bones_wire_release() {
		echo "[bonesdeploy] Running post-receive checkout + release wiring..."

		if ! bonesremote hooks post-receive --config "$BONES_TOML"; then
			echo "[bonesdeploy] post-receive hook command failed."
			exit 1
		fi

	echo "[bonesdeploy] Release wired."
}

	bones_run_deployment() {
		echo "[bonesdeploy] Running deploy hook command..."

		if ! bonesremote hooks deploy --config "$BONES_TOML"; then
			echo "[bonesdeploy] deploy hook command failed."
			exit 1
		fi

	echo "[bonesdeploy] Deploy hook command complete. Running post-deploy..."
}

bones_post_deploy() {
	echo "[bonesdeploy] Running post-deploy (hardening permissions)..."

	if ! sudo bonesremote hooks post-deploy --config "$BONES_TOML"; then
		echo "[bonesdeploy] post-deploy failed."
		exit 1
	fi

	echo "[bonesdeploy] post-deploy complete. Deployment finished."
}

bones_read_local_remote_name() {
	grep -E '^remote_name\s*=' .bones/bones.toml | head -1 | sed 's/.*=\s*"\(.*\)"/\1/'
}

bones_should_run_for_remote() {
	local pushed_remote_name="$1"
	BONES_REMOTE=$(bones_read_local_remote_name)

	if [ -z "$BONES_REMOTE" ]; then
		echo "[bonesdeploy] Warning: Could not read remote_name from .bones/bones.toml"
		return 1
	fi

	if [ "$pushed_remote_name" != "$BONES_REMOTE" ]; then
		return 1
	fi

	return 0
}

bones_run_doctor_local() {
	echo "[bonesdeploy] Pushing to bones remote '$BONES_REMOTE', running doctor..."

	if ! bonesdeploy doctor --local; then
		echo "[bonesdeploy] Doctor reported issues. Push aborted."
		exit 1
	fi

	echo "[bonesdeploy] Doctor passed."
}
