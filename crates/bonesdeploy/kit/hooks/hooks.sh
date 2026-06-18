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
	deploy_on_push=$(bonesremote config --file "$BONES_TOML" deploy_on_push 2>/dev/null || true)

	if [ -z "$deploy_on_push" ]; then
		return 0
	fi

	if [ "$deploy_on_push" = "false" ]; then
		return 1
	fi

	return 0
}

bones_read_config_branch() {
	bonesremote config --file "$BONES_TOML" branch 2>/dev/null || true
}

bones_is_zero_oid() {
	local oid="$1"
	[[ "$oid" =~ ^0+$ ]]
}

bones_resolve_deploy_push_target() {
	local branch
	branch=$(bones_read_config_branch)
	if [ -z "$branch" ]; then
		echo "[bonesdeploy] Could not read branch from $BONES_TOML"
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

bones_run_remote_deploy() {
	local revision="${1:-}"
	echo "[bonesdeploy] Starting remote deploy..."
	local cmd=(bonesremote deploy --config "$BONES_TOML")
	if [ -n "$revision" ]; then
		cmd+=(--revision "$revision")
	fi

	if ! "${cmd[@]}"; then
		echo "[bonesdeploy] remote deploy failed."
		exit 1
	fi

	echo "[bonesdeploy] Deployment finished."
}

bones_read_local_remote_name() {
	bonesdeploy config --file .bones/bones.toml remote_name 2>/dev/null || true
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
