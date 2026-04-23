GIT_DIR="${GIT_DIR:-.}"
GIT_DIR=$(cd "$GIT_DIR" && pwd)
BONES_TOML="$GIT_DIR/bones/bones.toml"
HOOKS_DIR="$GIT_DIR/hooks"

run_doctor_remote() {
	echo "[bonesdeploy] Running remote doctor..."

	if ! sudo bonesremote doctor; then
		echo "[bonesdeploy] Remote doctor reported issues. Push rejected."
		exit 1
	fi

	echo "[bonesdeploy] Doctor passed. Staging release..."
}

stage_release() {
	if ! sudo bonesremote release stage --config "$BONES_TOML"; then
		echo "[bonesdeploy] release stage failed. Push rejected."
		exit 1
	fi

	echo "[bonesdeploy] Release staged."
}

wire_release() {
	echo "[bonesdeploy] Running post-receive checkout + release wiring..."

	if ! sudo bonesremote hooks post-receive --config "$BONES_TOML"; then
		echo "[bonesdeploy] post-receive hook command failed."
		exit 1
	fi

	echo "[bonesdeploy] Release wired. Waiting for checkout to complete..."
}

run_deployment() {
	echo "[bonesdeploy] Checkout complete. Running deploy hook command..."

	if ! sudo bonesremote hooks deploy --config "$BONES_TOML"; then
		echo "[bonesdeploy] deploy hook command failed."
		exit 1
	fi

	echo "[bonesdeploy] Deploy hook command complete. Running post-deploy..."
}

post_deploy() {
	echo "[bonesdeploy] Running post-deploy (hardening permissions)..."

	if ! sudo bonesremote hooks post-deploy --config "$BONES_TOML"; then
		echo "[bonesdeploy] post-deploy failed."
		exit 1
	fi

	echo "[bonesdeploy] post-deploy complete. Deployment finished."
}