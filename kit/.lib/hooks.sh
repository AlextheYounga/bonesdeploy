	bones_init_remote_context() {
	local git_dir_input="${1:-${GIT_DIR:-.}}"
	GIT_DIR=$(cd "$git_dir_input" && pwd)
	BONES_YAML="$GIT_DIR/bones/bones.yaml"
}

bones_should_deploy_on_push() {
	if [ "${BONES_FORCE_DEPLOY:-0}" = "1" ]; then
		return 0
	fi

	local deploy_on_push
	deploy_on_push=$(grep -E '^[[:space:]]*deploy_on_push:[[:space:]]*' "$BONES_YAML" | head -1 | sed 's/#.*$//' | sed 's/^[^:]*:[[:space:]]*//' | tr -d '[:space:]')

	if [ -z "$deploy_on_push" ]; then
		return 0
	fi

	if [ "$deploy_on_push" = "false" ]; then
		return 1
	fi

	return 0
}

bones_read_config_branch() {
	grep -E '^[[:space:]]*branch:[[:space:]]*' "$BONES_YAML" | head -1 | sed 's/#.*$//' | sed 's/^[^:]*:[[:space:]]*//' | sed "s/^'//" | sed "s/'$//" | sed 's/^"//' | sed 's/"$//'
}

bones_is_zero_oid() {
	local oid="$1"
	[[ "$oid" =~ ^0+$ ]]
}

bones_resolve_deploy_push_target() {
	local branch
	branch=$(bones_read_config_branch)
	if [ -z "$branch" ]; then
		echo "[bonesdeploy] Could not read branch from $BONES_YAML"
		return 1
	fi

	local target_ref="refs/heads/$branch"
	local oldrev=""
	local newrev=""
	local refname=""

	if [ "${BONES_FORCE_DEPLOY:-0}" = "1" ]; then
		newrev=$(git --git-dir "$GIT_DIR" rev-parse "$target_ref" 2>/dev/null || true)
		if [ -z "$newrev" ]; then
			echo "[bonesdeploy] Configured deployment ref not found: $target_ref"
			return 1
		fi
	else
		while read -r oldrev newrev refname; do
			if [ "$refname" = "$target_ref" ]; then
				break
			fi
			newrev=""
		done

		if [ -z "$newrev" ]; then
			echo "[bonesdeploy] Push did not update $target_ref; skipping deployment."
			return 1
		fi

		if bones_is_zero_oid "$newrev"; then
			echo "[bonesdeploy] Push deleted $target_ref; skipping deployment."
			return 1
		fi
	fi

	export BONES_DEPLOY_REF="$target_ref"
	export BONES_DEPLOY_NEWREV="$newrev"
	return 0
}

	bones_run_doctor_remote() {
		echo "[bonesdeploy] Running remote doctor..."

		if ! bonesremote doctor; then
			echo "[bonesdeploy] Remote doctor reported issues. Push rejected."
			exit 1
		fi

	echo "[bonesdeploy] Doctor passed. Staging release..."
}

bones_stage_release() {
	if ! sudo bonesremote release stage --config "$BONES_YAML"; then
		echo "[bonesdeploy] release stage failed. Push rejected."
		exit 1
	fi

	echo "[bonesdeploy] Release staged."
}

bones_wire_release() {
	local revision="${1:-}"
	echo "[bonesdeploy] Running post-receive checkout..."
	local cmd=(bonesremote hooks post-receive --config "$BONES_YAML")
	if [ -n "$revision" ]; then
		cmd+=(--revision "$revision")
	fi

	if ! "${cmd[@]}"; then
		echo "[bonesdeploy] post-receive hook command failed."
		exit 1
	fi

	echo "[bonesdeploy] Wiring shared paths just-in-time..."
	if ! sudo bonesremote release wire --config "$BONES_YAML"; then
		echo "[bonesdeploy] release wire failed."
		exit 1
	fi

	echo "[bonesdeploy] Release wired."
}

	bones_run_deployment() {
		echo "[bonesdeploy] Running deploy hook command..."

		if ! bonesremote hooks deploy --config "$BONES_YAML"; then
			echo "[bonesdeploy] deploy hook command failed."
			exit 1
		fi

	echo "[bonesdeploy] Deploy hook command complete. Running post-deploy..."
}

bones_post_deploy() {
	echo "[bonesdeploy] Running post-deploy (hardening permissions)..."

	if ! sudo bonesremote hooks post-deploy --config "$BONES_YAML"; then
		echo "[bonesdeploy] post-deploy failed."
		exit 1
	fi

	echo "[bonesdeploy] post-deploy complete. Deployment finished."
}

bones_read_local_remote_name() {
	grep -E '^[[:space:]]*remote_name:[[:space:]]*' .bones/bones.yaml | head -1 | sed 's/#.*$//' | sed 's/^[^:]*:[[:space:]]*//' | sed "s/^'//" | sed "s/'$//" | sed 's/^"//' | sed 's/"$//'
}

bones_should_run_for_remote() {
	local pushed_remote_name="$1"
	BONES_REMOTE=$(bones_read_local_remote_name)

	if [ -z "$BONES_REMOTE" ]; then
		echo "[bonesdeploy] Warning: Could not read remote_name from .bones/bones.yaml"
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
