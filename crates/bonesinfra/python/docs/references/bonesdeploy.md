Project Path: bonesdeploy

Source Tree:

```txt
bonesdeploy
├── Cargo.toml
├── README.md
├── clippy.toml
├── compile.sh
├── crates
│   ├── bonesdeploy
│   │   ├── Cargo.toml
│   │   ├── kit
│   │   │   ├── bones.toml
│   │   │   ├── deployment
│   │   │   │   └── build
│   │   │   │       ├── 01_install_build_deps.sh
│   │   │   │       └── 02_run_build.sh
│   │   │   ├── hooks
│   │   │   │   ├── post-receive
│   │   │   │   └── pre-push
│   │   │   └── runtime.toml
│   │   ├── runtimes
│   │   │   ├── django
│   │   │   │   ├── deployment
│   │   │   │   │   └── build
│   │   │   │   │       ├── 01_install_node_deps.sh
│   │   │   │   │       └── 02_run_build.sh
│   │   │   │   ├── runtime.toml
│   │   │   │   └── secrets
│   │   │   ├── laravel
│   │   │   │   ├── deployment
│   │   │   │   │   └── build
│   │   │   │   │       ├── 01_install_node_deps.sh
│   │   │   │   │       ├── 02_install_php_deps.sh
│   │   │   │   │       ├── 03_build_frontend.sh
│   │   │   │   │       └── 04_build_laravel.sh
│   │   │   │   ├── runtime.toml
│   │   │   │   └── secrets
│   │   │   ├── next
│   │   │   │   ├── deployment
│   │   │   │   │   └── build
│   │   │   │   │       └── 01_install_node_deps.sh
│   │   │   │   ├── runtime.toml
│   │   │   │   └── secrets
│   │   │   ├── nuxt
│   │   │   │   ├── deployment
│   │   │   │   │   └── build
│   │   │   │   │       ├── 01_install_node_deps.sh
│   │   │   │   │       └── 02_run_build.sh
│   │   │   │   ├── runtime.toml
│   │   │   │   └── secrets
│   │   │   ├── rails
│   │   │   │   ├── deployment
│   │   │   │   │   └── build
│   │   │   │   │       ├── 01_install_node_deps.sh
│   │   │   │   │       └── 02_run_build.sh
│   │   │   │   ├── runtime.toml
│   │   │   │   └── secrets
│   │   │   ├── sveltekit
│   │   │   │   ├── deployment
│   │   │   │   │   └── build
│   │   │   │   │       ├── 01_install_node_deps.sh
│   │   │   │   │       └── 02_run_build.sh
│   │   │   │   ├── runtime.toml
│   │   │   │   └── secrets
│   │   │   └── vue
│   │   │       ├── deployment
│   │   │       │   └── build
│   │   │       │       ├── 01_install_node_deps.sh
│   │   │       │       └── 02_run_build.sh
│   │   │       ├── runtime.toml
│   │   │       └── secrets
│   │   └── src
│   │       ├── cli
│   │       │   ├── args.rs
│   │       │   ├── dispatch.rs
│   │       │   └── mod.rs
│   │       ├── commands
│   │       │   ├── config.rs
│   │       │   ├── deploy_project.rs
│   │       │   ├── doctor.rs
│   │       │   ├── guide.rs
│   │       │   ├── init_config.rs
│   │       │   ├── init_project.rs
│   │       │   ├── manage.rs
│   │       │   ├── mod.rs
│   │       │   ├── pull_state.rs
│   │       │   ├── push_state.rs
│   │       │   ├── remote_data.rs
│   │       │   ├── remote_runtime.rs
│   │       │   ├── remote_setup.rs
│   │       │   ├── remote_ssl.rs
│   │       │   ├── rollback.rs
│   │       │   ├── secrets.rs
│   │       │   ├── setup.rs
│   │       │   ├── status.rs
│   │       │   ├── tests
│   │       │   │   └── test_init_project.rs
│   │       │   ├── update.rs
│   │       │   ├── update_release.rs
│   │       │   └── version.rs
│   │       ├── config.rs
│   │       ├── infra
│   │       │   ├── bonesinfra.rs
│   │       │   ├── bonesinfra_cli.rs
│   │       │   ├── bootstrap_ssh.rs
│   │       │   ├── embedded.rs
│   │       │   ├── git.rs
│   │       │   ├── mod.rs
│   │       │   ├── rsync.rs
│   │       │   └── ssh.rs
│   │       ├── main.rs
│   │       └── ui
│   │           ├── mod.rs
│   │           └── prompts.rs
│   ├── bonesremote
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── cli
│   │       │   ├── args.rs
│   │       │   ├── dispatch.rs
│   │       │   └── mod.rs
│   │       ├── commands
│   │       │   ├── activate_release.rs
│   │       │   ├── deploy.rs
│   │       │   ├── doctor.rs
│   │       │   ├── drop_failed_release.rs
│   │       │   ├── hook.rs
│   │       │   ├── init.rs
│   │       │   ├── mod.rs
│   │       │   ├── post_deploy.rs
│   │       │   ├── release_build.rs
│   │       │   ├── release_checkout.rs
│   │       │   ├── release_prepare.rs
│   │       │   ├── service.rs
│   │       │   ├── site.rs
│   │       │   ├── stage_release.rs
│   │       │   ├── status.rs
│   │       │   ├── tests
│   │       │   │   └── test_doctor.rs
│   │       │   ├── version.rs
│   │       │   └── wire_shared.rs
│   │       ├── main.rs
│   │       ├── privileges.rs
│   │       ├── release
│   │       │   ├── mod.rs
│   │       │   └── scripts.rs
│   │       └── release_state.rs
│   └── shared
│       ├── Cargo.toml
│       └── src
│           ├── config.rs
│           ├── lib.rs
│           ├── paths.rs
│           └── registry.rs
├── docker
│   ├── Dockerfile
│   └── docker-compose.yml
├── docs
│   ├── PROJECT.md
│   ├── images
│   ├── references
│   ├── thinking
│   │   ├── 01_security_architecture_problems.md
│   │   ├── 02_new_architecture_approach.md
│   │   ├── 03_bonesdeploy_bonesremote_concerns.md
│   │   ├── 04_scout_migration_impact.md
│   │   ├── 05_security_model_migration_plan.md
│   │   └── 05_security_model_migration_plan_addendum.md
│   └── todo.md
├── rustfmt.toml
└── tests
    ├── ASSERTIONS.md
    └── cleancode
        ├── Cargo.toml
        └── src
            ├── cleancode_file_too_long.rs
            ├── cleancode_no_duplicated_state.rs
            ├── cleancode_no_legacy_terms.rs
            ├── cleancode_no_literal_wrapped_fallback.rs
            ├── cleancode_no_manufactured_success.rs
            └── lib.rs

```

`Cargo.toml`:

```toml
[workspace]
resolver = "3"
members = ["crates/shared", "crates/bonesdeploy", "crates/bonesremote", "tests/cleancode"]

[workspace.lints.clippy]

```

`README.md`:

````md
# BonesDeploy

## Remote release deployment tool for simple Linux servers

<div style="margin:0 auto; display: block;"><img width=600 height=600 src="docs/images/bonesdeploy.png" alt="BonesDeploy" /></div>

> WARNING: BonesDeploy is still under active development. There may be some cool bugs!

BonesDeploy deploys project releases to a remote Linux server over SSH. It scaffolds deployment configs and scripts into your repo, publishes the `.bones/` dataset into root-owned `bonesremote` site state, and runs the release lifecycle remotely without turning the bare Git repo into the control plane.

It produces two binaries:
- **`bonesdeploy`** — local CLI for init, config, deployment, and management
- **`bonesremote`** — server-side release lifecycle executor, installed on the deployment host

## Why BonesDeploy

BonesDeploy is built for developers who want direct server deploys without handing deployment over to a PaaS or rebuilding everything around Docker.

- **Drop-in** — add it to an existing repo, scaffold `.bones/`, and deploy over your existing SSH workflow
- **Simple lifecycle** — `bonesdeploy deploy` runs the full deployment pipeline on the remote server; `bonesremote deploy` owns the lifecycle
- **Permission-aware** — BonesDeploy treats deploy-user to service-user handoff as a first-class concern instead of leaving shared groups or ACL sprawl behind
- **Self-hosted and lightweight** — ideal for VPSes, old servers, and Raspberry Pis where simplicity matters more than orchestration
- **Editable by design** — the generated hooks and deployment scripts are yours; BonesDeploy gives you structure, not lock-in
- **Git optional** — git push can trigger deploys through a thin post-receive adapter, but `bonesdeploy deploy` is the primary deployment command

If you want a Heroku-style abstraction layer, use a platform. If you want a disciplined, transparent deployment tool that drops into a normal Linux box, use BonesDeploy.

## How It Works

BonesDeploy uses a two-user permission contract:

1. A **deploy user** (default: `git`) handles SSH access, owns the bare repo, and creates release artifacts. This user has restricted sudo ability (service restart only) but no password login.
2. A **runtime user** (defaults to the project name) owns runtime state — shared files, sockets, and writable directories. This user has no home folder, no login, and no sudo ability — limiting attack scope.

Permissions are a **provisioning-time contract**, not a deployment-time repair:

- The deploy user owns immutable release archives (`releases/`) with setgid so the runtime group can read them.
- The runtime user owns mutable shared state (`shared/`).
- Root creates users, directories, systemd units, and sockets during provisioning.
- No deploy step changes ownership or applies recursive chown.

The sudoers configuration is strictly limited to `bonesremote service restart`, the only command that needs elevated privileges during normal operation.

This gives you a clean privilege boundary:

- the **deploy user** can connect, stage, and activate
- the **runtime user** runs the app
- `root` provisions the machine and restarts services

## Installation

### Local (bonesdeploy)

```sh
cargo install --locked --git https://github.com/AlextheYounga/bonesdeploy.git bonesdeploy
````

### Server (bonesremote)

```sh
sudo cargo install --locked --root /usr/local --git https://github.com/AlextheYounga/bonesdeploy.git bonesremote --force
```

Then run the remote setup:

```sh
bonesdeploy remote setup
```

This installs a sudoers drop-in at `/etc/sudoers.d/bonesdeploy` so the deploy user can run only the privileged `bonesremote` commands without a password.

## Usage

### Initial Setup

In your project repository:

```sh
bonesdeploy init
```

This will:

1. Create a `.bones/` folder with deployment scripts and hooks
1. Prompt for project name, branch, remote name, host, and port
1. Add `.bones` to `.gitignore`
1. Symlink the `pre-push` hook into `.git/hooks/`
1. Create a local deployment git remote if needed

BonesDeploy assumes opinionated server defaults unless you change them in `.bones/bones.toml`:

- `port = "22"`
- `project_root = "/srv/sites/<project_name>"`
- `deploy_user = "git"`
- `runtime_user = "<project_name>"`
- `runtime_group = "<project_name>"`
- `release_group = "<project_name>-release"`

`web_root` lives in `.bones/runtime.toml` (default `"public"`), not `bones.toml`.

The `init` command creates the local `.bones/` scaffold and records project settings.
Infra operations (remote setup, runtime, SSL) are delegated to the `bonesinfra` Python package, which `bonesdeploy` clones into `~/.config/bonesdeploy/bonesinfra/` on first use. `pyinfra` is a dependency of that package and is installed into its venv automatically.
Template-based projects then use `bonesdeploy remote runtime` to prompt for a framework and scaffold runtime assets (for example: Laravel installs PHP + PHP-FPM, Django installs Python runtime packages, Node templates install Node.js).
`bonesdeploy remote setup` handles machine bootstrap as root, while `bonesdeploy remote runtime` applies per-site runtime assets such as AppArmor and nginx after a quick confirmation prompt.

To customize nginx behavior, edit the Jinja2 templates shipped with the `bonesinfra` checkout and re-run `bonesdeploy remote runtime`.

When DNS is ready, enable SSL with certbot (separate from runtime):

```sh
bonesdeploy remote ssl --domain app.example.com --email ops@example.com
```

This runs the dedicated SSL deploy to obtain a Let's Encrypt certificate and configure the runtime nginx router for HTTPS. SSL is fully decoupled from runtime configuration.

### Syncing Configuration

After editing hooks or deployment scripts in `.bones/`:

```sh
bonesdeploy push
```

This archives the local `.bones/` dataset and streams it to `bonesremote site import --site <project>`. The remote site state is stored under `/root/.config/bonesremote/sites/<project>/`, not inside the bare repo.

### Deploying

Deploy the configured project release:

```sh
bonesdeploy deploy
```

This SSHs into the host and runs `bonesremote deploy --site <project>`, which orchestrates the current server-side pipeline: stage release → export source from the bare repo into a temp build context → run `deployment/build/*.sh` in Podman → promote the release → wire shared paths → activate → restart services → prune old releases.

To roll back to the previous release without rebuilding:

```sh
bonesdeploy rollback
```

### Git-triggered deploy (optional)

If `deploy_on_push = true`:

```sh
git push production master
# post-receive forwards stdin to sudo bonesremote hook post-receive --site <project>
```

Git post-receive is a thin adapter — it does not orchestrate the deployment lifecycle. `bonesremote deploy` owns the lifecycle regardless of the trigger.

If `deploy_on_push = false` (the default), pushes only update refs. Run `bonesdeploy deploy` when ready.

### Health Checks

```sh
bonesdeploy doctor          # check local + remote
bonesdeploy doctor --local  # check local only
```

### Updating

Update BonesDeploy binaries to the latest release:

```sh
bonesdeploy update
```

This rebuilds both local (`bonesdeploy`) and remote (`bonesremote`) from the git source via `cargo install --locked --git https://github.com/AlextheYounga/bonesdeploy.git`. The remote update runs over SSH as root and also ensures `/srv/sites` exists with the correct ownership and permissions.

## Configuration

`bonesdeploy init` generates `.bones/bones.toml`:

```toml
remote_name = "production"
project_name = "myproject"
repo_path = "/home/git/myproject.git"
project_root = "/srv/sites/myproject"
port = '22'
branch = 'master'
domain = ''
preview_domain = ""
email = ''
deploy_on_push = false
ssl_enabled = false
releases = 5
```

`host` and `repo_path` are inferred from the deployment remote URL when possible; if parsing fails, init asks only for those missing values.

## Project Structure

```
.bones/
├── bones.toml           # project configuration
├── runtime.toml         # framework runtime configuration
├── hooks/
│   ├── hooks.sh         # (legacy) shared hook functions imported by hook entrypoints
│   ├── pre-push         # symlinked to .git/hooks/pre-push
│   └── post-receive     # thin adapter → calls bonesremote deploy
└── deployment/
    └── build/
        └── 01_*.sh      # build scripts (run sequentially in Podman)
```

Hooks are written to `.bones/hooks/` once during init. `pre-push` is now a self-contained guard; remote `post-receive` is a thin trigger that delegates to `sudo bonesremote hook post-receive --site <project>`. After that they belong to you — edit freely. Build scripts in `.bones/deployment/build/` must be numbered (e.g. `01_install_deps.sh`, `02_build.sh`) and are always run in order inside the `build_image` configured in `.bones/runtime.toml`.

Git hooks exist as an optional transport — `bonesdeploy deploy` is the primary deployment command. `post-receive` is a thin adapter that delegates to `bonesremote hook post-receive`, which resolves policy from bonesremote-managed site state.

## Good Fit

BonesDeploy is a strong fit when you want:

- direct Linux deploys over SSH
- simple app hosting on one machine at a time
- explicit provisioning-time permission contracts with setgid group inheritance
- a lightweight alternative to container-first deployment stacks
- something you can run comfortably on low-cost hosts and Raspberry Pis

BonesDeploy can still deploy Docker-based apps if your deployment scripts call `docker compose`, but Docker is optional rather than the foundation.

## License

MIT

## Coverage

Coverage is driven with `cargo-llvm-cov` using cargo aliases in `.cargo/config.toml`.

Install once:

```sh
cargo install cargo-llvm-cov
```

Generate a terminal summary:

```sh
cargo cov
```

Generate lcov output for CI tooling:

```sh
cargo cov-lcov
```

Generate an HTML report:

```sh
cargo cov-html
```

Reports are written under `target/coverage/`.

````

`clippy.toml`:

```toml
too-many-lines-threshold = 60
cognitive-complexity-threshold = 15
too-many-arguments-threshold = 4
````

`compile.sh`:

```sh
cargo build --release
mkdir -p ./bin
cp ./target/release/bonesdeploy ./bin/
cp ./target/release/bonesremote ./bin/

```

`crates/bonesdeploy/Cargo.toml`:

```toml
[package]
name = "bonesdeploy"
version = "0.5.1"
edition = "2024"

[dependencies]
anyhow = "1.0.102"
clap = { version = "4.6.1", features = ["derive"] }
console = "0.15"
inquire = "0.9.4"
openssh = { version = "0.11.6", features = ["native-mux"] }
oxc-toml = "0.14.4"
rust-embed = { version = "8.11.0", features = ["include-exclude"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
toml_edit = "0.22"
tempfile = "3.0"
tokio = { version = "1.52.1", features = ["rt", "rt-multi-thread", "macros"] }
shared = { path = "../shared" }

[lints.clippy]
# broad groups (lower priority so individual lint overrides take effect)
correctness = { level = "deny", priority = -1 }
suspicious = { level = "deny", priority = -1 }
complexity = { level = "warn", priority = -1 }
style = { level = "allow", priority = -1 }
perf = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }

# readability / naming
similar_names = "warn"
many_single_char_names = "warn"
module_name_repetitions = "warn"
enum_variant_names = "warn"
struct_field_names = "warn"
disallowed_names = "deny"
wildcard_imports = "warn"
module_inception = "warn"

# complexity / API readability
cognitive_complexity = "warn"
too_many_arguments = "deny"
fn_params_excessive_bools = "warn"
large_types_passed_by_value = "warn"
trivially_copy_pass_by_ref = "warn"

# panic / debug leftovers
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
# This is a CLI binary: stdout/stderr output is the intended UX.
print_stdout = "allow"
print_stderr = "allow"

# docs / contracts
missing_panics_doc = "deny"
missing_errors_doc = "deny"
missing_safety_doc = "deny"
undocumented_unsafe_blocks = "deny"

# numeric safety
cast_possible_truncation = "warn"
cast_sign_loss = "warn"
cast_possible_wrap = "warn"
checked_conversions = "warn"

# anti-mess / consistency
absolute_paths = "warn"
allow_attributes = "deny"
redundant_clone = "warn"
implicit_clone = "warn"
semicolon_if_nothing_returned = "warn"
match_same_arms = "warn"
needless_pass_by_value = "warn"
cloned_instead_of_copied = "warn"
flat_map_option = "warn"
from_iter_instead_of_collect = "warn"
inefficient_to_string = "warn"
manual_let_else = "warn"
manual_ok_or = "warn"
map_unwrap_or = "warn"
unnecessary_wraps = "warn"

```

`crates/bonesdeploy/kit/bones.toml`:

```toml
# project_name = "mysite"
# remote_name = "production"
# host = ''
ssh_user = "root"
port = '22'
branch = 'master'
domain = ''
preview_domain = ""
email = ''
deploy_on_push = false
ssl_enabled = false
releases = 5

```

`crates/bonesdeploy/kit/deployment/build/01_install_build_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
NVM_INSTALL_URL="https://raw.githubusercontent.com/nvm-sh/nvm/v0.40.5/install.sh"

if [ -s "$NVM_DIR/nvm.sh" ]; then
	exit 0
fi

if command -v curl >/dev/null 2>&1; then
	curl -fsSL "$NVM_INSTALL_URL" | PROFILE=/dev/null NVM_DIR="$NVM_DIR" bash
else
	echo "curl or wget is required to install nvm."
	exit 1
fi

```

`crates/bonesdeploy/kit/deployment/build/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

# Load nvm if .nvmrc is present
if [ -f "./.nvmrc" ]; then
	export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
	if [ -s "$NVM_DIR/nvm.sh" ]; then
		# shellcheck disable=SC1090
		source "$NVM_DIR/nvm.sh"
	elif [ -s "$HOME/.config/nvm/nvm.sh" ]; then
		# shellcheck disable=SC1090
		source "$HOME/.config/nvm/nvm.sh"
	fi
	nvm install
fi

# Clean install and build
rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
	npm install -g pnpm
	pnpm install --frozen-lockfile
	pnpm build
elif [ -f "./yarn.lock" ]; then
	command -v corepack >/dev/null 2>&1 && corepack enable || true
	yarn install --frozen-lockfile
	yarn build
elif [ -f "./package-lock.json" ]; then
	npm install --include=optional
	npm run build
else
	echo "No lockfile found. Run your package manager locally first."
	exit 1
fi

```

`crates/bonesdeploy/kit/hooks/post-receive`:

```
#!/usr/bin/env bash
#
# post-receive — Remote hook, runs after refs are updated.
#
# Resolves the pushed deployment ref and delegates the full remote
# deployment lifecycle to bonesremote.

set -euo pipefail

GIT_DIR="${GIT_DIR:-.}"
GIT_DIR=$(cd "$GIT_DIR" && pwd)
SITE=$(basename "$GIT_DIR")
SITE=${SITE%.git}

if [ -z "$SITE" ] || [ "$SITE" = "." ] || [ "$SITE" = "/" ]; then
	echo "[bonesdeploy] Could not derive site name from GIT_DIR=$GIT_DIR"
	exit 1
fi

exec sudo bonesremote hook post-receive --site "$SITE"

```

`crates/bonesdeploy/kit/hooks/pre-push`:

```
#!/usr/bin/env bash
#
# pre-push — Local hook, symlinked to .git/hooks/pre-push
#
# Checks if we are pushing to the bonesdeploy-managed remote.
# If so, runs `bonesdeploy doctor` and aborts on any failure.
#
# Git passes the remote name and URL as arguments:
#   $1 = remote name
#   $2 = remote URL

set -euo pipefail
REMOTE_NAME="$1"
REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"
HOOKS_LIB="$REPO_ROOT/.bones/hooks/hooks.sh"

if [ ! -f "$HOOKS_LIB" ]; then
	echo "[bonesdeploy] Missing hook library: $HOOKS_LIB"
	exit 1
fi

# shellcheck source=/dev/null
. "$HOOKS_LIB"

main() {
	if bones_should_run_for_remote "$REMOTE_NAME"; then
		bones_run_doctor_local
	fi
}

main

```

`crates/bonesdeploy/kit/runtime.toml`:

```toml
runtime_user = ""
runtime_group = ""
release_group = ""
build_image = "docker.io/library/buildpack-deps:bookworm"

[permissions]
paths = [{ path = "*", type = "dir", mode = "750" }, { path = "*", type = "file", mode = "640" }]

```

`crates/bonesdeploy/runtimes/django/deployment/build/01_install_node_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	command -v php >/dev/null 2>&1 || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];

		if (isset($package["volta"]["node"])) {
			echo $package["volta"]["node"];
			exit;
		}

		if (isset($package["engines"]["node"])) {
			echo $package["engines"]["node"];
			exit;
		}
	'
}

read_node_version() {
	if [ -n "${BONES_NODE_VERSION:-}" ]; then
		echo "$BONES_NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"

```

`crates/bonesdeploy/runtimes/django/deployment/build/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

command -v python3 >/dev/null 2>&1 || {
	echo "python3 not found"
	exit 1
}

# Activate virtualenv
VENV_DIR="${VENV_DIR:-venv}"
if [ -d "./$VENV_DIR" ]; then
	# shellcheck disable=SC1090
	source "./$VENV_DIR/bin/activate"
else
	echo "Virtual environment not found at ./$VENV_DIR"
	echo "Create one on the server: python3 -m venv $VENV_DIR"
	exit 1
fi

# Install dependencies
if [ -f "./requirements.txt" ]; then
	pip install -r requirements.txt --quiet
elif [ -f "./pyproject.toml" ]; then
	pip install . --quiet
fi

# Run migrations
python3 manage.py migrate --noinput

# Collect static files
python3 manage.py collectstatic --noinput

# Restart gunicorn via systemd
SERVICE_NAME="$PROJECT_NAME"
if ! command -v systemctl >/dev/null 2>&1; then
	echo "systemctl not found. Restart your app server manually."
elif systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
	systemctl restart "$SERVICE_NAME"
elif systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
	systemctl start "$SERVICE_NAME"
else
	echo "No systemd service found for $SERVICE_NAME. Restart your app server manually."
fi

```

`crates/bonesdeploy/runtimes/django/runtime.toml`:

```toml
template = "django"
web_root = "public"
build_image = "docker.io/library/python:3.13-bookworm"

[permissions]
paths = [
  { path = "*", type = "dir", mode = "750" },
  { path = "*", type = "file", mode = "640" },
  { path = "static", type = "dir", mode = "750", recursive = true },
  { path = "media", type = "dir", mode = "770", recursive = true },
]

```

`crates/bonesdeploy/runtimes/laravel/deployment/build/01_install_node_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

skip_unless_laravel_project() {
	if [ ! -f artisan ]; then
		log "artisan not found; skipping Laravel build dependency install."
		exit 0
	fi
}

skip_unless_node_project() {
	if [ ! -f package.json ]; then
		log "package.json not found; skipping Node install."
		exit 0
	fi
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	command -v php >/dev/null 2>&1 || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];

		if (isset($package["volta"]["node"])) {
			echo $package["volta"]["node"];
			exit;
		}

		if (isset($package["engines"]["node"])) {
			echo $package["engines"]["node"];
			exit;
		}
	'
}

read_node_version() {
	if [ -n "${BONES_NODE_VERSION:-}" ]; then
		echo "$BONES_NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	skip_unless_laravel_project
	skip_unless_node_project

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"

```

`crates/bonesdeploy/runtimes/laravel/deployment/build/02_install_php_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

on_error() {
	local status=$?
	echo "$LOG_PREFIX Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2
	exit "$status"
}

trap on_error ERR

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_command() {
	local name="$1"

	command -v "$name" >/dev/null 2>&1 || die "$name not found"
}

require_file() {
	local file="$1"
	local message="$2"

	[ -f "$file" ] || die "$message"
}

require_environment() {
	require_file artisan "artisan not found"
	require_command php
	require_command composer

	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_environment() {
	export COMPOSER_ALLOW_SUPERUSER="${COMPOSER_ALLOW_SUPERUSER:-1}"
	export CI=1
	export COREPACK_ENABLE_DOWNLOAD_PROMPT=0
}

artisan_command_exists() {
	local command_name="$1"

	php artisan list --raw 2>/dev/null |
		awk '{ print $1 }' |
		grep -qx -- "$command_name"
}

install_composer_dependencies() {
	log "Installing Composer dependencies..."

	composer install \
		--no-dev \
		--prefer-dist \
		--no-interaction \
		--optimize-autoloader
}

main() {
	require_environment
	configure_environment

	install_composer_dependencies

	trap - ERR

	log "Successfully installed php dependencies."
}

main "$@"

```

`crates/bonesdeploy/runtimes/laravel/deployment/build/03_build_frontend.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

on_error() {
	local status=$?
	echo "$LOG_PREFIX Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2
	exit "$status"
}

trap on_error ERR

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_command() {
	local name="$1"

	command -v "$name" >/dev/null 2>&1 || die "$name not found"
}

package_json_package_manager() {
	[ -f package.json ] || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];
		$packageManager = $package["packageManager"] ?? "";

		if ($packageManager !== "") {
			echo explode("@", $packageManager)[0], PHP_EOL;
		}
	' 2>/dev/null || true
}

package_json_has_build_script() {
	[ -f package.json ] || return 1

	[ "$(
		php -r '
			$package = json_decode(file_get_contents("package.json"), true) ?: [];
			echo isset($package["scripts"]["build"]) ? "1" : "0";
		' 2>/dev/null || true
	)" = "1" ]
}

detect_package_manager() {
	local package_manager

	package_manager="$(package_json_package_manager)"

	if [ -n "$package_manager" ]; then
		echo "$package_manager"
		return
	fi

	if [ -f pnpm-lock.yaml ]; then
		echo "pnpm"
	elif [ -f yarn.lock ]; then
		echo "yarn"
	else
		echo "npm"
	fi
}

ensure_node_toolchain() {
	export PATH="$PROJECT_ROOT/build/node/bin:$PATH"

	require_command node
	require_command npm

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$(dirname "$(command -v node)")" 2>/dev/null || true
}

install_and_build_with_npm() {
	if [ -f package-lock.json ] || [ -f npm-shrinkwrap.json ]; then
		log "Installing frontend dependencies with npm ci..."
		npm ci --include=dev
	else
		log "package-lock.json not found; falling back to npm install..."
		npm install
	fi

	log "Building frontend assets with npm..."
	npm run build
}

install_and_build_with_pnpm() {
	if [ -f pnpm-lock.yaml ]; then
		log "Installing frontend dependencies with pnpm frozen lockfile..."
		corepack pnpm install --frozen-lockfile --prod=false
	else
		log "pnpm-lock.yaml not found; falling back to non-frozen pnpm install..."
		corepack pnpm install --prod=false
	fi

	log "Building frontend assets with pnpm..."
	corepack pnpm run build
}

install_and_build_with_yarn() {
	local yarn_version

	yarn_version="$(corepack yarn --version 2>/dev/null || true)"

	if [[ "$yarn_version" == 1.* ]]; then
		log "Installing frontend dependencies with Yarn classic..."
		corepack yarn install --frozen-lockfile
	else
		log "Installing frontend dependencies with Yarn modern..."
		corepack yarn install --immutable
	fi

	log "Building frontend assets with yarn..."
	corepack yarn run build
}

run_frontend_build() {
	local package_manager

	if [ ! -f package.json ]; then
		log "package.json not found; skipping frontend build."
		return
	fi

	if ! package_json_has_build_script; then
		log "package.json has no build script; skipping frontend build."
		return
	fi

	ensure_node_toolchain

	package_manager="$(detect_package_manager)"

	log "Node: $(node --version)"
	log "npm:  $(npm --version)"
	log "Frontend package manager: $package_manager"

	case "$package_manager" in
	npm)
		install_and_build_with_npm
		;;

	pnpm)
		install_and_build_with_pnpm
		;;

	yarn)
		install_and_build_with_yarn
		;;

	*)
		die "Unsupported package manager: $package_manager"
		;;
	esac
}

generate_wayfinder_files() {
	log "Generating Wayfinder files..."
	php artisan wayfinder:generate
}

main() {
	# Generate frontend route/action files before Vite compiles the JS/TS bundle.
	generate_wayfinder_files

	run_frontend_build

	trap - ERR

	log "Frontend built successfully"
}

main "$@"

```

`crates/bonesdeploy/runtimes/laravel/deployment/build/04_build_laravel.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

on_error() {
	local status=$?
	echo "$LOG_PREFIX Failed at line $LINENO: $BASH_COMMAND (status $status)" >&2
	exit "$status"
}

trap on_error ERR

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_command() {
	local name="$1"

	command -v "$name" >/dev/null 2>&1 || die "$name not found"
}

artisan_command_exists() {
	local command_name="$1"

	php artisan list --raw 2>/dev/null |
		awk '{ print $1 }' |
		grep -qx -- "$command_name"
}

ensure_app_key() {
	if [ ! -f .env ] || ! grep -Eq '^APP_KEY=base64:' .env; then
		log "Generating Laravel APP_KEY..."
		php artisan key:generate --force
	fi
}

ensure_storage_link() {
	log "Ensuring Laravel storage link exists..."
	php artisan storage:link --force || true
}

run_migrations() {
	if [ "${BONES_LARAVEL_SKIP_MIGRATIONS:-0}" = "1" ]; then
		log "Skipping migrations because BONES_LARAVEL_SKIP_MIGRATIONS=1."
		return
	fi

	log "Running migrations..."
	php artisan migrate --force
}

restart_queue_workers() {
	if artisan_command_exists "queue:restart"; then
		php artisan queue:restart || true
	fi
}

finish_laravel_build() {
	php artisan optimize:clear
	restart_queue_workers
	php artisan up
}

main() {
	ensure_app_key
	ensure_storage_link
	run_migrations
	finish_laravel_build

	trap - ERR

	log "Laravel build complete."
}

main "$@"

```

`crates/bonesdeploy/runtimes/laravel/runtime.toml`:

```toml
template = "laravel"
web_root = "public"
php_version = "8.5"
build_image = "docker.io/library/composer:2"

[permissions]
paths = [
  { path = "*", type = "dir", mode = "750" },
  { path = "*", type = "file", mode = "640" },
  { path = "bootstrap/cache", type = "dir", mode = "770", recursive = true },
]

```

`crates/bonesdeploy/runtimes/next/deployment/build/01_install_node_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	command -v php >/dev/null 2>&1 || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];

		if (isset($package["volta"]["node"])) {
			echo $package["volta"]["node"];
			exit;
		}

		if (isset($package["engines"]["node"])) {
			echo $package["engines"]["node"];
			exit;
		}
	'
}

read_node_version() {
	if [ -n "${BONES_NODE_VERSION:-}" ]; then
		echo "$BONES_NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"

```

`crates/bonesdeploy/runtimes/next/runtime.toml`:

```toml
template = "next"
web_root = "public"
build_image = "docker.io/library/node:24-bookworm"

[permissions]
paths = [
  { path = "*", type = "dir", mode = "750" },
  { path = "*", type = "file", mode = "640" },
  { path = ".next", type = "dir", mode = "770", recursive = true },
]

```

`crates/bonesdeploy/runtimes/nuxt/deployment/build/01_install_node_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	command -v php >/dev/null 2>&1 || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];

		if (isset($package["volta"]["node"])) {
			echo $package["volta"]["node"];
			exit;
		}

		if (isset($package["engines"]["node"])) {
			echo $package["engines"]["node"];
			exit;
		}
	'
}

read_node_version() {
	if [ -n "${BONES_NODE_VERSION:-}" ]; then
		echo "$BONES_NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"

```

`crates/bonesdeploy/runtimes/nuxt/deployment/build/02_run_build.sh`:

```sh
#!/usr/bin/env bash
set -Eeuo pipefail

export PATH="$PROJECT_ROOT/build/node/bin:$PATH"

if ! command -v corepack >/dev/null 2>&1; then
	npm install -g corepack@latest
fi

corepack enable --install-directory "$(dirname "$(command -v node)")" 2>/dev/null || true

if [ -f "./pnpm-lock.yaml" ]; then
	corepack pnpm install --frozen-lockfile
	corepack pnpm generate
elif [ -f "./yarn.lock" ]; then
	corepack yarn install --frozen-lockfile
	corepack yarn generate
elif [ -f "./package-lock.json" ]; then
	npm ci --include=optional
	npm run generate
else
	echo "No lockfile found. Run your package manager locally first."
	exit 1
fi

```

`crates/bonesdeploy/runtimes/nuxt/runtime.toml`:

```toml
template = "nuxt"
web_root = ".output/public"
build_image = "docker.io/library/node:24-bookworm"

[permissions]
paths = [
  { path = "*", type = "dir", mode = "750" },
  { path = "*", type = "file", mode = "640" },
  { path = ".output", type = "dir", mode = "770", recursive = true },
  { path = ".nuxt", type = "dir", mode = "770", recursive = true },
]

```

`crates/bonesdeploy/runtimes/rails/deployment/build/01_install_node_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	command -v php >/dev/null 2>&1 || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];

		if (isset($package["volta"]["node"])) {
			echo $package["volta"]["node"];
			exit;
		}

		if (isset($package["engines"]["node"])) {
			echo $package["engines"]["node"];
			exit;
		}
	'
}

read_node_version() {
	if [ -n "${BONES_NODE_VERSION:-}" ]; then
		echo "$BONES_NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"

```

`crates/bonesdeploy/runtimes/rails/deployment/build/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

command -v ruby >/dev/null 2>&1 || {
	echo "ruby not found"
	exit 1
}
command -v bundle >/dev/null 2>&1 || {
	echo "bundler not found"
	exit 1
}

# Load rbenv if available
if [ -d "$HOME/.rbenv" ]; then
	export PATH="$HOME/.rbenv/bin:$PATH"
	eval "$(rbenv init -)"
fi

# Install Ruby version from .ruby-version if rbenv is available
if [ -f "./.ruby-version" ] && command -v rbenv >/dev/null 2>&1; then
	rbenv install --skip-existing
fi

# Install dependencies
bundle install --deployment --without development test

# Precompile assets
if bundle exec rails assets:precompile 2>/dev/null; then
	echo "Assets precompiled."
fi

# Run database migrations
RAILS_ENV=production bundle exec rails db:migrate

# Restart puma via systemd
SERVICE_NAME="$PROJECT_NAME"
if ! command -v systemctl >/dev/null 2>&1; then
	echo "systemctl not found. Restart your app server manually."
elif systemctl is-active --quiet "$SERVICE_NAME" 2>/dev/null; then
	systemctl restart "$SERVICE_NAME"
elif systemctl list-unit-files | grep -q "$SERVICE_NAME"; then
	systemctl start "$SERVICE_NAME"
else
	echo "No systemd service found for $SERVICE_NAME. Restart your app server manually."
fi

```

`crates/bonesdeploy/runtimes/rails/runtime.toml`:

```toml
template = "rails"
web_root = "public"
build_image = "docker.io/library/ruby:3.4-bookworm"

[permissions]
paths = [
  { path = "*", type = "dir", mode = "750" },
  { path = "*", type = "file", mode = "640" },
  { path = "tmp", type = "dir", mode = "770", recursive = true },
  { path = "log", type = "dir", mode = "770", recursive = true },
  { path = "storage", type = "dir", mode = "770", recursive = true },
  { path = "public/assets", type = "dir", mode = "750", recursive = true },
]

```

`crates/bonesdeploy/runtimes/sveltekit/deployment/build/01_install_node_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	command -v php >/dev/null 2>&1 || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];

		if (isset($package["volta"]["node"])) {
			echo $package["volta"]["node"];
			exit;
		}

		if (isset($package["engines"]["node"])) {
			echo $package["engines"]["node"];
			exit;
		}
	'
}

read_node_version() {
	if [ -n "${BONES_NODE_VERSION:-}" ]; then
		echo "$BONES_NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Laravel frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"

```

`crates/bonesdeploy/runtimes/sveltekit/deployment/build/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export PATH="$PROJECT_ROOT/build/node/bin:$PATH"

if ! command -v corepack >/dev/null 2>&1; then
	npm install -g corepack@latest
fi

corepack enable --install-directory "$(dirname "$(command -v node)")" 2>/dev/null || true

rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
	corepack pnpm install --frozen-lockfile
	corepack pnpm build
elif [ -f "./yarn.lock" ]; then
	corepack yarn install --frozen-lockfile
	corepack yarn build
elif [ -f "./package-lock.json" ]; then
	npm install --include=optional
	npm run build
else
	echo "No lockfile found. Run your package manager locally first."
	exit 1
fi

```

`crates/bonesdeploy/runtimes/sveltekit/runtime.toml`:

```toml
template = "sveltekit"
web_root = "build"
build_image = "docker.io/library/node:24-bookworm"

[permissions]
paths = [
  { path = "*", type = "dir", mode = "750" },
  { path = "*", type = "file", mode = "640" },
  { path = "build", type = "dir", mode = "770", recursive = true },
]

```

`crates/bonesdeploy/runtimes/vue/deployment/build/01_install_node_deps.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

readonly LOG_PREFIX="[bonesdeploy]"

TMP_DIR=""

on_error() {
	local status="$1"
	local line="$2"
	local command="$3"

	echo "$LOG_PREFIX Failed at line $line: $command (status $status)" >&2
	exit "$status"
}

cleanup() {
	if [ -n "${TMP_DIR:-}" ]; then
		rm -rf "$TMP_DIR"
	fi
}

trap 'on_error "$?" "$LINENO" "$BASH_COMMAND"' ERR
trap cleanup EXIT

log() {
	echo "$LOG_PREFIX $*"
}

die() {
	echo "$LOG_PREFIX $*" >&2
	exit 1
}

require_environment() {
	: "${PROJECT_ROOT:?PROJECT_ROOT must be set by bonesremote}"
}

configure_paths() {
	NODE_DIR="$PROJECT_ROOT/build/node"
	NODE_BIN="$NODE_DIR/bin/node"

	export NODE_DIR
	export NODE_BIN
}

read_node_version_from_package_json() {
	command -v php >/dev/null 2>&1 || return 0

	php -r '
		$package = json_decode(file_get_contents("package.json"), true) ?: [];

		if (isset($package["volta"]["node"])) {
			echo $package["volta"]["node"];
			exit;
		}

		if (isset($package["engines"]["node"])) {
			echo $package["engines"]["node"];
			exit;
		}
	'
}

read_node_version() {
	if [ -n "${BONES_NODE_VERSION:-}" ]; then
		echo "$BONES_NODE_VERSION"
		return
	fi

	if [ -f .node-version ]; then
		head -n 1 .node-version
		return
	fi

	if [ -f .nvmrc ]; then
		head -n 1 .nvmrc
		return
	fi

	if [ -f .tool-versions ]; then
		awk '$1 == "nodejs" || $1 == "node" { print $2; exit }' .tool-versions
		return
	fi

	read_node_version_from_package_json
}

normalize_node_version() {
	sed \
		-e 's/#.*$//' \
		-e 's/\r$//' \
		-e 's/^[[:space:]]*//' \
		-e 's/[[:space:]]*$//' \
		-e 's/^v//'
}

resolve_node_version() {
	read_node_version |
		head -n 1 |
		normalize_node_version || true
}

assert_exact_node_version() {
	local version="$1"

	if [[ "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
		return
	fi

	cat >&2 <<'EOF'
[bonesdeploy] Vue frontend build requires an exact pinned Node version.

Add one of these to the project:
  .node-version        example: 24.17.0
  .nvmrc              example: 24.17.0
  .tool-versions      example: nodejs 24.17.0
  package.json volta  example: "volta": { "node": "24.17.0" }

Or set:
  BONES_NODE_VERSION=24.17.0

Aliases and ranges are intentionally rejected:
  node
  latest
  lts/*
  24
  >=20
EOF

	exit 1
}

ensure_corepack() {
	export PATH="$NODE_DIR/bin:$PATH"

	if ! command -v corepack >/dev/null 2>&1; then
		log "corepack not found in Node install; installing corepack..."
		npm install -g corepack@latest
	fi

	corepack enable --install-directory "$NODE_DIR/bin" 2>/dev/null || true
}

node_version_is_installed() {
	local version="$1"

	[ -x "$NODE_BIN" ] &&
		"$NODE_BIN" --version | grep -qx "v$version"
}

detect_node_architecture() {
	local arch

	arch="$(uname -m)"

	case "$arch" in
	x86_64)
		echo "x64"
		;;

	aarch64)
		echo "arm64"
		;;

	*)
		die "Unsupported architecture for Node binary install: $arch"
		;;
	esac
}

node_download_url() {
	local version="$1"
	local node_arch="$2"

	echo "https://nodejs.org/dist/v${version}/node-v${version}-linux-${node_arch}.tar.xz"
}

install_node() {
	local version="$1"
	local node_arch
	local url

	node_arch="$(detect_node_architecture)"
	url="$(node_download_url "$version" "$node_arch")"

	TMP_DIR="$(mktemp -d)"

	log "Installing Node v${version}..."
	log "Downloading $url"

	mkdir -p "$(dirname "$NODE_DIR")"

	curl -fsSL --retry 3 --retry-delay 2 "$url" |
		tar -xJ -C "$TMP_DIR"

	rm -rf "$NODE_DIR"
	mv "$TMP_DIR/node-v${version}-linux-${node_arch}" "$NODE_DIR"
}

print_installed_versions() {
	log "Node installed: $(node --version)"
	log "npm installed:  $(npm --version)"
}

main() {
	local version

	require_environment
	configure_paths

	version="$(resolve_node_version)"
	assert_exact_node_version "$version"

	if node_version_is_installed "$version"; then
		log "Node v${version} already installed."
		ensure_corepack
		return
	fi

	install_node "$version"
	ensure_corepack
	print_installed_versions
}

main "$@"

```

`crates/bonesdeploy/runtimes/vue/deployment/build/02_run_build.sh`:

```sh
#!/usr/bin/env bash

set -Eeuo pipefail

export PATH="$PROJECT_ROOT/build/node/bin:$PATH"

if ! command -v corepack >/dev/null 2>&1; then
	npm install -g corepack@latest
fi

corepack enable --install-directory "$(dirname "$(command -v node)")" 2>/dev/null || true

rm -rf node_modules

if [ -f "./pnpm-lock.yaml" ]; then
	corepack pnpm install --frozen-lockfile
	corepack pnpm build
elif [ -f "./yarn.lock" ]; then
	corepack yarn install --frozen-lockfile
	corepack yarn build
elif [ -f "./package-lock.json" ]; then
	npm install --include=optional
	npm run build
else
	echo "No lockfile found. Run your package manager locally first."
	exit 1
fi

```

`crates/bonesdeploy/runtimes/vue/runtime.toml`:

```toml
template = "vue"
web_root = "dist"
build_image = "docker.io/library/node:24-bookworm"

[permissions]
paths = [
  { path = "*", type = "dir", mode = "750" },
  { path = "*", type = "file", mode = "640" },
  { path = "dist", type = "dir", mode = "755", recursive = true },
]

```

`crates/bonesdeploy/src/cli/args.rs`:

```rs
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "bonesdeploy", about = "Remote release deployment tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Set up bonesdeploy in the current repository. Run this once per project.
    Init {
        /// Skip all interactive prompts; required fields must be provided via flags
        #[arg(long)]
        non_interactive: bool,
        /// Run remote setup after init (instead of prompting)
        #[arg(long)]
        setup_remote: bool,
        /// Project name (default: current directory name)
        #[arg(long)]
        project_name: Option<String>,
        /// Git branch to deploy
        #[arg(long)]
        branch: Option<String>,
        /// Deployment remote name (default: production)
        #[arg(short = 'r', long)]
        remote: Option<String>,
        /// Server hostname or IP
        #[arg(short = 'H', long)]
        host: Option<String>,
        /// SSH port (default: 22)
        #[arg(long)]
        port: Option<String>,
    },
    /// Run the full first-time deployment setup
    Setup {
        /// Skip runtime confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Check local and remote environment health
    Doctor {
        /// Skip remote checks
        #[arg(long)]
        local: bool,
    },
    /// Show the current deployment state and next steps
    Status,
    /// Suggest the next prompt-free command to run
    Guide {
        /// Output format
        #[arg(long, value_enum, default_value_t = GuideFormat::Text)]
        format: GuideFormat,
    },
    /// Publish .bones/ into bonesremote's remote control-plane state
    Push,
    /// Recover .bones/ from bonesremote's remote control-plane state
    Pull,
    /// Manage encrypted local secrets and push them to remote shared/
    Secrets {
        #[command(subcommand)]
        command: SecretsCommand,
    },
    /// Deploy the configured project release to the remote server
    Deploy,
    /// Update bonesdeploy and bonesremote to the latest version
    Update {
        /// Skip local update
        #[arg(long)]
        skip_local: bool,
        /// Skip remote update
        #[arg(long)]
        skip_remote: bool,
    },
    /// Remote operations
    Remote {
        #[command(subcommand)]
        command: RemoteCommand,
    },
    /// Open remote server management TUI
    Manage,
    /// Roll back current release to the previous one
    Rollback,
    /// Get a config value from a TOML file
    Config {
        /// Path to TOML config file
        #[arg(long)]
        file: String,
        /// Key to read
        key: String,
    },
    /// Print the version
    Version,
}

#[derive(Subcommand)]
pub enum SecretsCommand {
    /// Create the local secrets config and storage directory
    Init,
    /// Decrypt the .env secret, edit it, then re-encrypt it
    Edit,
    /// Decrypt local secrets and write them into remote shared/
    Push,
}

#[derive(Subcommand)]
pub enum RemoteCommand {
    /// Run remote bootstrap against configured host
    #[command(alias = "setup")]
    Bootstrap,
    /// Apply the configured runtime against configured host
    Runtime {
        /// Skip runtime confirmation prompts
        #[arg(long)]
        yes: bool,
    },
    /// Obtain and configure SSL certificates with certbot
    Ssl {
        /// Skip SSL confirmation prompts
        #[arg(long)]
        yes: bool,
        /// Domain name for the certificate (e.g. app.example.com)
        #[arg(long)]
        domain: Option<String>,
        /// Email used for Let's Encrypt registration and notices
        #[arg(long)]
        email: Option<String>,
    },
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum GuideFormat {
    Text,
    Json,
}

```

`crates/bonesdeploy/src/cli/dispatch.rs`:

```rs
use anyhow::Result;

use crate::cli::args::{Cli, Command, RemoteCommand, SecretsCommand};
use crate::commands::{
    config, deploy_project, doctor, guide, init_config, init_project, manage, pull_state, push_state, remote_runtime,
    remote_setup, remote_ssl, rollback, secrets, setup, status, update, version,
};

pub async fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init { non_interactive, setup_remote, project_name, branch, remote, host, port } => {
            let remote_setup_ran = init_project::run(&init_config::InitArgs {
                non_interactive: *non_interactive,
                setup_remote: *setup_remote,
                project_name: project_name.clone(),
                branch: branch.clone(),
                remote: remote.clone(),
                host: host.clone(),
                port: port.clone(),
            })?;
            if remote_setup_ran {
                push_state::run(false)?;
            }
            Ok(())
        }
        Command::Setup { yes } => setup::run(*yes).await,
        Command::Doctor { local } => doctor::run(*local).await,
        Command::Status => status::run().await,
        Command::Guide { format } => guide::run(*format).await,
        Command::Push => push_state::run(true),
        Command::Pull => pull_state::run(),
        Command::Secrets { command } => match command {
            SecretsCommand::Init => secrets::init(),
            SecretsCommand::Edit => secrets::edit(),
            SecretsCommand::Push => secrets::push().await,
        },
        Command::Deploy => deploy_project::run().await,
        Command::Update { skip_local, skip_remote } => {
            update::run(update::Options { skip_local: *skip_local, skip_remote: *skip_remote }).await
        }
        Command::Manage => manage::run(),
        Command::Remote { command } => match command {
            RemoteCommand::Bootstrap => remote_setup::run(),
            RemoteCommand::Runtime { yes } => remote_runtime::run(*yes),
            RemoteCommand::Ssl { yes, domain, email } => remote_ssl::run(*yes, domain.clone(), email.clone()),
        },
        Command::Rollback => rollback::run().await,
        Command::Config { file, key } => config::run(file, key),
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}

```

`crates/bonesdeploy/src/cli/mod.rs`:

```rs
pub mod args;
pub mod dispatch;

```

`crates/bonesdeploy/src/commands/config.rs`:

```rs
use std::fs;

use anyhow::{Context, Result, bail};

pub fn run(file: &str, key: &str) -> Result<()> {
    let content = fs::read_to_string(file).with_context(|| format!("Failed to read config file: {file}"))?;
    let value: toml::Value = toml::from_str(&content).with_context(|| format!("Failed to parse TOML: {file}"))?;

    let result = value.get(key);
    let Some(toml_value) = result else {
        bail!("Key '{key}' not found in {file}");
    };

    match toml_value {
        toml::Value::String(s) => print!("{s}"),
        toml::Value::Boolean(b) => print!("{b}"),
        toml::Value::Integer(i) => print!("{i}"),
        toml::Value::Float(f) => print!("{f}"),
        _ => bail!("Unsupported value type for key '{key}'"),
    }

    Ok(())
}

```

`crates/bonesdeploy/src/commands/deploy_project.rs`:

```rs
use std::path::Path;

use anyhow::{Context, Result};

use crate::commands::push_state;
use crate::config;
use crate::infra::ssh;
use shared::paths;

pub fn local_bones_load_error() -> String {
    format!("Failed to load {}", paths::LOCAL_BONES_TOML)
}

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml).context(local_bones_load_error())?;

    println!("Deploying {} to {}...", cfg.project_name, cfg.host);

    // Ensure local .bones/ is published into the remote control plane before triggering deploy.
    push_state::sync_bones_directory(&cfg).context("Failed to publish .bones to bonesremote.")?;

    let session = ssh::connect_privileged(&cfg).await?;

    let command = format!("bonesremote deploy --site '{}'", single_quote(&cfg.project_name));
    ssh::stream_cmd(&session, &command).await?;

    session.close().await?;

    println!("Deployment complete.");

    Ok(())
}

fn single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

```

`crates/bonesdeploy/src/commands/doctor.rs`:

```rs
use std::fs;
use std::path::Path;

use anyhow::Result;
use shared::paths::bonesremote_registry_path;
use tokio::runtime::Runtime;

use crate::config;
use crate::infra::ssh;
use shared::paths;

pub async fn run(local_only: bool) -> Result<()> {
    println!("Checking deployment...");

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML)).ok();
    let deploy_on_push = cfg.as_ref().is_some_and(|c| c.deploy_on_push);

    let mut issues = 0usize;

    issues += print_check(".bones config", check_bones_config(), Some("run bonesdeploy init"));
    issues += print_check(
        "deployment scripts",
        check_deployment_scripts(),
        Some("rename it with a numeric prefix, like 01_build.sh"),
    );

    if deploy_on_push {
        issues += print_check("pre-push hook", check_pre_push_hook(), Some("run bonesdeploy init"));
    }

    if !local_only {
        match &cfg {
            Some(cfg) => {
                let remote_ssh_issue = check_remote_ssh(cfg).await;
                issues +=
                    print_check("remote SSH", remote_ssh_issue.clone(), Some("check host, port, and SSH access."));
                if remote_ssh_issue.is_none() {
                    issues += print_check(
                        "bonesremote",
                        check_bonesremote(cfg).await,
                        Some("run bonesdeploy remote bootstrap"),
                    );
                    issues += print_check(".bones sync", check_bones_sync(cfg), Some("run bonesdeploy push"));
                }
            }
            None => {
                issues += print_failure("remote SSH", "Missing .bones config", Some("run bonesdeploy init"));
            }
        }
    }

    if issues == 0 {
        println!();
        println!("All checks passed.");
        Ok(())
    } else {
        println!();
        let issue_word = if issues == 1 { "issue" } else { "issues" };
        anyhow::bail!("Doctor found {issues} {issue_word}.");
    }
}

fn print_check(label: &str, issue: Option<String>, next: Option<&str>) -> usize {
    match issue {
        None => {
            println!("✓ {label}");
            0
        }
        Some(issue) => print_failure(label, &issue, next),
    }
}

fn print_failure(label: &str, issue: &str, next: Option<&str>) -> usize {
    println!("✗ {label}");
    let issue = issue.replace('\n', "\n  ");
    println!("  {issue}");
    if let Some(next) = next {
        println!("  Next: {next}");
    }
    1
}

fn check_bones_config() -> Option<String> {
    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);

    if !bones_dir.exists() {
        return Some(String::from("Missing .bones config"));
    }

    if !bones_dir.is_symlink() {
        return Some(String::from(".bones is not managed by bonesdeploy"));
    }

    if !Path::new(paths::LOCAL_BONES_TOML).exists() {
        return Some(format!("Missing {}", paths::LOCAL_BONES_TOML));
    }

    None
}

fn check_deployment_scripts() -> Option<String> {
    let deployment_dir = Path::new(paths::LOCAL_BONES_DEPLOYMENT_DIR);
    if !deployment_dir.exists() {
        return None;
    }

    let entries = fs::read_dir(deployment_dir).ok()?;
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let has_numeric_prefix = name.chars().take_while(char::is_ascii_digit).count() > 0;
        if !has_numeric_prefix {
            return Some(format!("Deployment script is not ordered: {name}"));
        }
    }

    None
}

fn check_pre_push_hook() -> Option<String> {
    let link = Path::new(paths::GIT_PRE_PUSH_HOOK);

    if !link.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
        return Some(String::from("pre-push hook is not installed"));
    }

    let target = fs::read_link(link).ok()?;
    let expected = Path::new(paths::PRE_PUSH_HOOK_TARGET);
    if target != expected {
        return Some(String::from("pre-push hook is not installed"));
    }

    None
}

async fn check_remote_ssh(cfg: &config::Bones) -> Option<String> {
    match ssh::connect(cfg).await {
        Ok(session) => {
            let _ = session.close().await;
            None
        }
        Err(error) => Some(format!("Cannot connect to remote\n  {error}")),
    }
}

async fn check_bonesremote(cfg: &config::Bones) -> Option<String> {
    let session = ssh::connect(cfg).await.ok()?;
    let result = ssh::run_cmd(&session, "command -v bonesremote").await;
    let _ = session.close().await;

    if result.is_ok() { None } else { Some(String::from("bonesremote is missing")) }
}

fn check_bones_sync(cfg: &config::Bones) -> Option<String> {
    let registry_path = bonesremote_registry_path(&cfg.project_name);
    let command = format!("test -r {}", shell_quote(&registry_path.display().to_string()));
    let Ok(runtime) = Runtime::new() else {
        return Some(String::from("Could not start runtime to check remote site state"));
    };
    let Ok(session) = runtime.block_on(ssh::connect_privileged(cfg)) else {
        return Some(String::from("Could not connect to remote site state"));
    };
    let ok = runtime.block_on(ssh::run_cmd(&session, &command)).is_ok();
    let _ = runtime.block_on(session.close());

    if ok { None } else { Some(String::from("remote bonesremote site state is missing; run bonesdeploy push")) }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::shell_quote;
    use crate::commands::push_state;

    #[test]
    fn doctor_points_at_new_remote_import_flow() {
        assert_eq!(push_state::remote_import_command("acme"), "bonesremote site import --site 'acme'");
    }

    #[test]
    fn shell_quote_escapes_single_quotes() {
        assert_eq!(shell_quote("a'b"), "'a'\\''b'");
    }
}

```

`crates/bonesdeploy/src/commands/guide.rs`:

```rs
use std::path::Path;

use anyhow::{Context, Result};
use serde::Serialize;
use shared::config as shared_config;
use shared::paths;
use shared::paths::bonesremote_registry_path;

use crate::cli::args::GuideFormat;
use crate::config;
use crate::infra::ssh;

#[derive(Clone, Debug, Serialize)]
pub struct Report {
    pub project: String,
    pub state: String,
    pub state_label: String,
    pub missing: Vec<String>,
    pub commands: Vec<String>,
    pub next: NextCommand,
    #[serde(skip)]
    pub cfg: Option<config::Bones>,
}

#[derive(Clone, Debug, Serialize)]
pub struct NextCommand {
    pub command: String,
    pub mutates: bool,
    pub contacts_remote: bool,
    pub prompt_free_command: String,
}

pub async fn run(format: GuideFormat) -> Result<()> {
    let report = build_report().await?;

    match format {
        GuideFormat::Text => print_text(&report),
        GuideFormat::Json => println!("{}", serde_json::to_string_pretty(&report)?),
    }

    Ok(())
}

pub async fn build_report() -> Result<Report> {
    let project = config::repo_directory_name()?;
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);

    if !bones_toml.exists() {
        return Ok(uninitialized_report(&project));
    }

    let cfg = config::load(bones_toml).with_context(|| format!("Failed to read {}", bones_toml.display()))?;
    let web_root = runtime_web_root()?;
    let setup_complete = remote_setup_complete(&cfg, &web_root).await.unwrap_or(false);

    if !setup_complete {
        return Ok(initialized_report(cfg, false));
    }

    let ssl_enabled = cfg.ssl_enabled || remote_ssl_enabled(&cfg, &web_root).await.unwrap_or(false);

    if ssl_enabled { Ok(ready_report(cfg)) } else { Ok(initialized_report(cfg, true)) }
}

pub(crate) fn prompt_free_init_command(project: &str) -> String {
    format!("bonesdeploy init --yes --project-name {project} --host <host>")
}

fn uninitialized_report(project: &str) -> Report {
    let command = prompt_free_init_command(project);
    Report {
        project: project.to_string(),
        state: String::from("uninitialized"),
        state_label: String::from("not initialized."),
        missing: vec![String::from("init")],
        commands: vec![command.clone()],
        next: next_command(&command, true, false),
        cfg: None,
    }
}

fn initialized_report(cfg: config::Bones, setup_complete: bool) -> Report {
    let (state, state_label, missing, commands) = if setup_complete {
        let command = ssl_command(&cfg);
        (
            String::from("setup_complete_ssl_missing"),
            String::from("setup complete, HTTPS missing."),
            vec![String::from("ssl")],
            vec![command, String::from("bonesdeploy deploy")],
        )
    } else {
        (
            String::from("initialized_setup_missing"),
            String::from("initialized, setup not complete."),
            vec![
                String::from("remote_bootstrap"),
                String::from("runtime"),
                String::from("bones_sync"),
                String::from("doctor_pass"),
            ],
            vec![String::from("bonesdeploy setup --yes"), ssl_command(&cfg), String::from("bonesdeploy deploy")],
        )
    };

    let next = next_command(&commands[0], true, true);

    Report { project: cfg.project_name.clone(), state, state_label, missing, commands, next, cfg: Some(cfg) }
}

fn ready_report(cfg: config::Bones) -> Report {
    let command = String::from("bonesdeploy deploy");

    Report {
        project: cfg.project_name.clone(),
        state: String::from("ready"),
        state_label: String::from("ready."),
        missing: Vec::new(),
        commands: vec![command.clone()],
        next: next_command(&command, true, true),
        cfg: Some(cfg),
    }
}

fn next_command(command: &str, mutates: bool, contacts_remote: bool) -> NextCommand {
    NextCommand { command: command.to_string(), mutates, contacts_remote, prompt_free_command: command.to_string() }
}

fn ssl_command(cfg: &config::Bones) -> String {
    let domain = if cfg.domain.is_empty() { String::from("<domain>") } else { cfg.domain.clone() };
    let email = if cfg.email.is_empty() { String::from("<email>") } else { cfg.email.clone() };
    format!("bonesdeploy remote ssl --yes --domain {domain} --email {email}")
}

fn print_text(report: &Report) {
    println!("Project: {}", report.project);
    println!("State: {}", report.state_label);
    println!();

    for (index, command) in report.commands.iter().enumerate() {
        if index == 0 {
            println!("Next: {command}");
        } else {
            println!("Then: {command}");
        }
    }
}

fn runtime_web_root() -> Result<String> {
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;
    Ok(runtime.web_root)
}

async fn remote_setup_complete(cfg: &config::Bones, web_root: &str) -> Result<bool> {
    let Ok(session) = ssh::connect_privileged(cfg).await else {
        return Ok(false);
    };

    if ssh::run_cmd(&session, "command -v bonesremote >/dev/null 2>&1").await.is_err() {
        session.close().await?;
        return Ok(false);
    }

    let registry_path = bonesremote_registry_path(&cfg.project_name);
    let sync_ok =
        ssh::run_cmd(&session, &format!("test -r {}", shell_quote(&registry_path.display().to_string()))).await.is_ok();

    let paths = cfg.deployment_paths(web_root);
    let current_ok = ssh::run_cmd(&session, &format!("test -e {}", shell_quote(&paths.current))).await.is_ok();

    session.close().await?;

    Ok(sync_ok && current_ok)
}

pub(crate) async fn remote_ssl_enabled(cfg: &config::Bones, web_root: &str) -> Result<bool> {
    if cfg.domain.is_empty() {
        return Ok(false);
    }

    let session = ssh::connect_privileged(cfg).await?;
    let path = cfg.deployment_paths(web_root).nginx_site_available;
    let command = format!(
        "test -r {path} && grep -Fq {domain} {path} && grep -Fq 'listen 443 ssl;' {path}",
        path = shell_quote(&path),
        domain = shell_quote(&format!("server_name {};", cfg.domain)),
    );
    let enabled = ssh::run_cmd(&session, &command).await.is_ok();
    session.close().await?;

    Ok(enabled)
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{initialized_report, prompt_free_init_command, ready_report, uninitialized_report};
    use crate::config::Bones;

    fn sample_cfg() -> Bones {
        Bones {
            project_name: String::from("atlas"),
            host: String::from("deploy.example.com"),
            port: String::from("22"),
            repo_path: String::from("/home/git/atlas.git"),
            project_root: String::from("/srv/sites/atlas"),
            branch: String::from("main"),
            ssl_enabled: false,
            ..Default::default()
        }
    }

    #[test]
    fn init_command_is_prompt_free() {
        assert_eq!(prompt_free_init_command("atlas"), "bonesdeploy init --yes --project-name atlas --host <host>");
    }

    #[test]
    fn uninitialized_report_starts_with_init() {
        let report = uninitialized_report("atlas");
        assert_eq!(report.commands[0], "bonesdeploy init --yes --project-name atlas --host <host>");
    }

    #[test]
    fn initialized_report_suggests_setup_then_ssl_then_deploy() {
        let report = initialized_report(sample_cfg(), false);
        assert_eq!(report.commands[0], "bonesdeploy setup --yes");
        assert_eq!(report.commands[1], "bonesdeploy remote ssl --yes --domain <domain> --email <email>");
        assert_eq!(report.commands[2], "bonesdeploy deploy");
    }

    #[test]
    fn ready_report_suggests_deploy() {
        let mut cfg = sample_cfg();
        cfg.ssl_enabled = true;
        let report = ready_report(cfg);
        assert_eq!(report.commands, vec![String::from("bonesdeploy deploy")]);
    }
}

```

`crates/bonesdeploy/src/commands/init_config.rs`:

```rs
use crate::config;
use crate::infra::git;

use anyhow::{Result, anyhow};
use shared::config::validate_host;
use shared::paths;

pub struct InitArgs {
    pub non_interactive: bool,
    pub setup_remote: bool,
    pub project_name: Option<String>,
    pub branch: Option<String>,
    pub remote: Option<String>,
    pub host: Option<String>,
    pub port: Option<String>,
}

pub(crate) fn collect_non_interactive(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &InitArgs,
) -> Result<config::Bones> {
    let project_name = resolve_project_name(args, existing_config, project_name_hint)?;
    let remote_name = resolve_remote_name(args, existing_config);
    let inferred_remote = infer_remote_details(&remote_name)?;
    let host = resolve_host(args, existing_config, inferred_remote.as_ref())?;
    let branch = resolve_branch(args, existing_config);
    let port = resolve_port(args, existing_config, inferred_remote.as_ref());
    validate_host(&host)?;

    let repo_path = resolve_repo_path(&project_name, existing_config, inferred_remote.as_ref());
    let project_root = existing_path_override(
        existing_config,
        |cfg| &cfg.project_root,
        &project_name,
        config::default_project_root_for,
    );
    let deploy_on_push = existing_config.is_none_or(|cfg| cfg.deploy_on_push);
    let releases_keep = existing_config.map_or(5, |cfg| cfg.releases_keep.max(1));

    let ssl_enabled = existing_config.is_some_and(|cfg| cfg.ssl_enabled);
    let domain = existing_config.map_or_else(String::new, |cfg| cfg.domain.clone());
    let email = existing_config.map_or_else(String::new, |cfg| cfg.email.clone());

    Ok(config::Bones {
        remote_name,
        project_name,
        host,
        port,
        repo_path,
        project_root,
        branch,
        deploy_on_push,
        releases_keep,
        ssl_enabled,
        domain,
        email,
        ..Default::default()
    })
}

fn resolve_project_name(
    args: &InitArgs,
    existing_config: Option<&config::Bones>,
    project_name_hint: &str,
) -> Result<String> {
    args.project_name
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.project_name)))
        .or_else(|| {
            let name = project_name_hint.to_string();
            (!name.is_empty()).then_some(name)
        })
        .ok_or_else(|| {
            anyhow!(
                "{} --project-name is required in non-interactive mode.\n\
                 Usage: bonesdeploy init --non-interactive --project-name <name> --host <host>",
                console::style("Error:").red().bold(),
            )
        })
}

fn resolve_remote_name(args: &InitArgs, existing_config: Option<&config::Bones>) -> String {
    args.remote
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.remote_name)))
        .unwrap_or_else(|| String::from("production"))
}

fn infer_remote_details(remote_name: &str) -> Result<Option<git::RemoteConnectionDetails>> {
    if git::remote_exists(remote_name)? { git::infer_remote_connection_details(remote_name) } else { Ok(None) }
}

fn resolve_host(
    args: &InitArgs,
    existing_config: Option<&config::Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    args.host
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.host)))
        .or_else(|| inferred_remote.map(|details| details.host.clone()))
        .ok_or_else(|| {
            anyhow!(
                "{} --host is required in non-interactive mode.\n\
                 Usage: bonesdeploy init --non-interactive --project-name <name> --host <host>",
                console::style("Error:").red().bold(),
            )
        })
}

fn resolve_branch(args: &InitArgs, existing_config: Option<&config::Bones>) -> String {
    args.branch
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.branch)))
        .unwrap_or_else(|| String::from("main"))
}

fn resolve_port(
    args: &InitArgs,
    existing_config: Option<&config::Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> String {
    args.port
        .clone()
        .filter(|v| !v.is_empty())
        .or_else(|| existing_config.and_then(|cfg| non_empty(&cfg.port)))
        .or_else(|| inferred_remote.map(|details| details.port.clone()))
        .unwrap_or_else(|| String::from("22"))
}

pub fn non_empty(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

pub fn resolve_repo_path(
    project_name: &str,
    existing_config: Option<&config::Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> String {
    if let Some(details) = inferred_remote {
        return details.repo_path.clone();
    }

    let configured_repo_path = existing_config.as_ref().map(|cfg| cfg.repo_path.as_str());

    let repo_path = match configured_repo_path {
        Some(path) if !path.is_empty() => path.replace("<project_name>", project_name),
        _ => paths::default_repo_path_for(project_name),
    };

    repo_path
}

pub fn existing_path_override(
    existing_config: Option<&config::Bones>,
    field: impl Fn(&config::Bones) -> &String,
    current_project_name: &str,
    default_for: fn(&str) -> String,
) -> String {
    let Some(cfg) = existing_config else { return String::new() };
    let value = field(cfg);
    if value.is_empty() {
        return String::new();
    }
    let resolved = value.replace("<project_name>", current_project_name);
    if resolved == default_for(current_project_name) { String::new() } else { resolved }
}

```

`crates/bonesdeploy/src/commands/init_project.rs`:

```rs
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use crate::commands::init_config;
pub use crate::commands::init_config::InitArgs;
use crate::commands::remote_setup;
use crate::config;
use crate::infra::bonesinfra;
use crate::infra::bonesinfra_cli;
use crate::infra::embedded;
use crate::infra::git;
use crate::ui::prompts;
use anyhow::{Context, Result, bail};
use shared::config::{bonesinfra_input, default_deploy_user, release_group_for, runtime_group_for, runtime_user_for};
use shared::paths;

pub fn run(args: &InitArgs) -> Result<bool> {
    run_with_prefetch(args, bonesinfra::prefetch)
}

fn run_with_prefetch(args: &InitArgs, prefetch_bonesinfra: impl FnOnce() -> Result<()>) -> Result<bool> {
    git::ensure_git_repository()?;

    println!("Initializing bonesdeploy...");
    prefetch_bonesinfra()?;

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    let had_bones_entry = fs::symlink_metadata(bones_dir).is_ok();
    let has_live_bones_dir = bones_dir.exists();
    let is_fresh = !has_live_bones_dir;

    let mut initial_project_name: Option<String> = None;

    if is_fresh {
        let project_name = resolve_project_name(args)?;
        let config_dir = config::bones_config_dir(&project_name);

        if config_dir.exists() && !config_dir.is_dir() {
            fs::remove_file(&config_dir)
                .with_context(|| format!("Stale file at {}, cannot create directory", config_dir.display()))?;
        }
        fs::create_dir_all(&config_dir)?;
        embedded::scaffold(&config_dir)?;

        if had_bones_entry {
            fs::remove_file(bones_dir)
                .with_context(|| format!("Failed to remove stale {} symlink", bones_dir.display()))?;
        }
        unix_fs::symlink(&config_dir, bones_dir)?;

        let existing = config::Bones { project_name: project_name.clone(), ..Default::default() };
        config::save(&existing, Path::new(paths::LOCAL_BONES_TOML))?;

        initial_project_name = Some(project_name);
    } else {
        println!("Using existing .bones config.");
    }

    update_gitignore()?;

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = load_or_collect_config(bones_toml, args)?;
    ensure_config_gitignore(&cfg.project_name)?;

    if let Some(ref initial) = initial_project_name
        && cfg.project_name != *initial
    {
        let old_dir = config::bones_config_dir(initial);
        let new_dir = config::bones_config_dir(&cfg.project_name);
        fs::rename(&old_dir, &new_dir)?;
        fs::remove_file(bones_dir)?;
        unix_fs::symlink(&new_dir, bones_dir)?;
    }

    config::save(&cfg, bones_toml)?;

    if is_fresh {
        println!("bonesdeploy initialized.");
    } else {
        println!("bonesdeploy config updated.");
    }

    if is_fresh {
        let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
        existing_runtime_config(args, &cfg.project_name, bones_dir, runtime_toml)?;
    }
    ensure_local_remote(&cfg)?;

    symlink_pre_push()?;

    let remote_setup_ran = args.setup_remote || (!args.non_interactive && prompts::confirm_remote_setup()?);
    if remote_setup_ran {
        remote_setup::run()?;
    } else {
        print_follow_up_hint();
    }

    Ok(remote_setup_ran)
}

fn print_follow_up_hint() {
    println!();
    println!("Next: run bonesdeploy setup.");
}

fn existing_runtime_config(args: &InitArgs, project_name: &str, bones_dir: &Path, runtime_toml: &Path) -> Result<()> {
    let template = if args.non_interactive {
        None
    } else {
        let available = embedded::runtime_names();
        prompts::choose_template(&available)?
    };

    if let Some(ref template_name) = template {
        let defaults = embedded::runtime_defaults(template_name)?;
        let answers = if args.non_interactive {
            serde_json::Value::Object(defaults.clone())
        } else {
            let questions = bonesinfra_cli::runtime_questions(template_name)?;
            prompts::prompt_runtime_questions(&questions, &serde_json::Value::Object(defaults.clone()))?
        };
        let mut map = answers.as_object().cloned().unwrap_or(defaults);
        inject_runtime_identity(&mut map, project_name);
        config::save_runtime(&map, runtime_toml)?;
        embedded::scaffold_runtime_deployment(template_name, bones_dir)?;
        embedded::scaffold_runtime_secrets(template_name, bones_dir)?;
        println!("Runtime template: {template_name}");
    } else {
        let mut vars = embedded::base_runtime_defaults()?;
        inject_runtime_identity(&mut vars, project_name);
        config::save_runtime(&vars, runtime_toml)?;
        println!("Runtime template: custom");
    }

    Ok(())
}

fn inject_runtime_identity(vars: &mut serde_json::Map<String, serde_json::Value>, project_name: &str) {
    vars.insert(bonesinfra_input::RUNTIME_USER.into(), serde_json::Value::String(runtime_user_for(project_name)));
    vars.insert(bonesinfra_input::RUNTIME_GROUP.into(), serde_json::Value::String(runtime_group_for(project_name)));
    vars.insert(bonesinfra_input::RELEASE_GROUP.into(), serde_json::Value::String(release_group_for(project_name)));
}

fn collect_from_existing(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &InitArgs,
) -> Result<config::Bones> {
    let project_name = cli_or_prompt(
        args.project_name.as_ref(),
        existing_config.and_then(|cfg| init_config::non_empty(&cfg.project_name)),
        || prompts::prompt_project_name(project_name_hint, existing_config),
    )?;
    let branch = cli_or_prompt(args.branch.as_ref(), None, || prompts::prompt_branch(existing_config))?;
    let remote_name = cli_or_prompt(args.remote.as_ref(), None, || prompts::prompt_remote_name(existing_config))?;
    let inferred_remote =
        if git::remote_exists(&remote_name)? { git::infer_remote_connection_details(&remote_name)? } else { None };
    let host =
        cli_or_prompt(args.host.as_ref(), None, || prompts::prompt_host(existing_config, inferred_remote.as_ref()))?;
    let port =
        cli_or_prompt(args.port.as_ref(), None, || prompts::prompt_port(existing_config, inferred_remote.as_ref()))?;
    let repo_path = init_config::resolve_repo_path(&project_name, existing_config, inferred_remote.as_ref());
    let project_root = init_config::existing_path_override(
        existing_config,
        |cfg| &cfg.project_root,
        &project_name,
        config::default_project_root_for,
    );
    let deploy_on_push = existing_config.is_none_or(|cfg| cfg.deploy_on_push);
    let releases_keep = existing_config.map_or(5, |cfg| cfg.releases_keep.max(1));

    let ssl_enabled = existing_config.is_some_and(|cfg| cfg.ssl_enabled);
    let domain = existing_config.map_or_else(String::new, |cfg| cfg.domain.clone());
    let email = existing_config.map_or_else(String::new, |cfg| cfg.email.clone());

    Ok(config::Bones {
        remote_name,
        project_name,
        host,
        port,
        repo_path,
        project_root,
        branch,
        deploy_on_push,
        releases_keep,
        ssl_enabled,
        domain,
        email,
        ..Default::default()
    })
}

#[cfg(test)]
fn collect_non_interactive(
    project_name_hint: &str,
    existing_config: Option<&config::Bones>,
    args: &InitArgs,
) -> Result<config::Bones> {
    init_config::collect_non_interactive(project_name_hint, existing_config, args)
}

fn resolve_project_name(args: &InitArgs) -> Result<String> {
    if let Some(name) = args.project_name.as_ref().filter(|v| !v.is_empty()) {
        return Ok(name.trim().to_string());
    }
    if args.non_interactive {
        bail!(
            "--project-name is required in non-interactive mode.\nUsage: bonesdeploy init --non-interactive --project-name <name> --host <host>"
        );
    }
    let hint = config::repo_directory_name()?;
    prompts::prompt_project_name(&hint, None)
}

fn cli_or_prompt(
    cli_value: Option<&String>,
    existing_value: Option<String>,
    prompt: impl FnOnce() -> Result<String>,
) -> Result<String> {
    match cli_value {
        Some(v) if !v.is_empty() => Ok(v.trim().to_string()),
        _ => existing_value.map_or_else(prompt, Ok),
    }
}

fn load_or_collect_config(bones_toml: &Path, args: &InitArgs) -> Result<config::Bones> {
    if bones_toml.exists() {
        let existing = config::load(bones_toml)?;
        if config::is_configured(&existing) {
            return Ok(existing);
        }
        let project_name = config::repo_directory_name()?;
        if args.non_interactive {
            return init_config::collect_non_interactive(&project_name, Some(&existing), args);
        }
        return collect_from_existing(&project_name, Some(&existing), args);
    }

    let project_name = config::repo_directory_name()?;

    if args.non_interactive {
        return init_config::collect_non_interactive(&project_name, None, args);
    }

    collect_from_existing(&project_name, None, args)
}

fn ensure_config_gitignore(project_name: &str) -> Result<()> {
    let gitignore = paths::bones_config_root().join(".gitignore");
    let project_entry = format!("{project_name}.bones");

    if gitignore.exists() {
        let content = fs::read_to_string(&gitignore)?;
        let mut missing = Vec::new();
        for entry in ["gnupg", &project_entry] {
            if !content.lines().any(|line| line.trim() == entry) {
                missing.push(entry);
            }
        }
        if missing.is_empty() {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        let mut append = String::new();
        for entry in &missing {
            append.push_str(entry);
            append.push('\n');
        }
        fs::write(&gitignore, format!("{content}{separator}{append}"))?;
    } else {
        let mut content = String::new();
        for entry in ["gnupg", &project_entry] {
            content.push_str(entry);
            content.push('\n');
        }
        fs::write(&gitignore, content)?;
    }

    Ok(())
}

fn update_gitignore() -> Result<()> {
    let gitignore = Path::new(".gitignore");
    let entry = paths::LOCAL_BONES_DIR;

    if gitignore.exists() {
        let content = fs::read_to_string(gitignore)?;
        if content.lines().any(|line| line.trim() == entry) {
            return Ok(());
        }
        let separator = if content.ends_with('\n') { "" } else { "\n" };
        fs::write(gitignore, format!("{content}{separator}{entry}\n"))?;
    } else {
        fs::write(gitignore, format!("{entry}\n"))?;
    }

    Ok(())
}

pub(crate) fn symlink_pre_push() -> Result<()> {
    let hooks_dir = Path::new(paths::GIT_HOOKS_DIR);
    fs::create_dir_all(hooks_dir)?;

    let link = hooks_dir.join(paths::PRE_PUSH_HOOK_NAME);
    let target = Path::new(paths::PRE_PUSH_HOOK_TARGET);

    if fs::symlink_metadata(&link).is_ok() {
        fs::remove_file(&link).with_context(|| format!("Failed to remove existing {}", link.display()))?;
    }

    unix_fs::symlink(target, &link).with_context(|| format!("Failed to symlink {}", link.display()))?;

    Ok(())
}

fn ensure_local_remote(cfg: &config::Bones) -> Result<()> {
    if git::remote_exists(&cfg.remote_name)? {
        return Ok(());
    }

    let remote_url = format!("{}@{}:{}", default_deploy_user(), cfg.host, cfg.repo_path);
    git::add_remote(&cfg.remote_name, &remote_url)?;
    Ok(())
}

#[cfg(test)]
#[path = "tests/test_init_project.rs"]
mod test_init_project;

```

`crates/bonesdeploy/src/commands/manage.rs`:

```rs
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use shared::config::default_deploy_user;
use shared::paths;

pub fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).repo_bones_toml;
    let remote_command = format!("bonesremote manage --config {}", shell_quote_single(&remote_bones_toml));

    let target = format!("{}@{}", default_deploy_user(), cfg.host);

    println!("Opening remote manage session...");

    let status = Command::new("ssh")
        .arg("-t")
        .arg("-p")
        .arg(&cfg.port)
        .arg(&target)
        .arg(&remote_command)
        .status()
        .context("Failed to launch ssh for remote manage session")?;

    if !status.success() {
        bail!("Could not open remote manage session.\n\nNext: run bonesdeploy status or check SSH access.");
    }

    Ok(())
}

fn shell_quote_single(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::shell_quote_single;
    use shared::paths;

    /// Wraps a plain value in single quotes to prevent whitespace and token splitting.
    #[test]
    fn shell_quote_single_wraps_plain_value_in_single_quotes() {
        let path = paths::default_project_root_for("acme");
        assert_eq!(shell_quote_single(&path), format!("'{path}'"));
    }

    /// Escapes embedded single quotes safely for remote shell execution.
    #[test]
    fn shell_quote_single_escapes_embedded_single_quotes() {
        assert_eq!(shell_quote_single("it'works"), "'it'\"'\"'works'");
    }

    /// Returns an explicit empty string for empty input, not a zero-length argument.
    #[test]
    fn shell_quote_single_handles_empty_string() {
        assert_eq!(shell_quote_single(""), "''");
    }
}

```

`crates/bonesdeploy/src/commands/mod.rs`:

```rs
pub(crate) mod config;
pub(crate) mod init_config;

pub mod deploy_project;
pub mod doctor;
pub mod guide;
pub mod init_project;
pub mod manage;
pub mod pull_state;
pub mod push_state;
pub mod remote_data;
pub mod remote_runtime;
pub mod remote_setup;
pub mod remote_ssl;
pub mod rollback;
pub mod secrets;
pub mod setup;
pub mod status;
pub mod update;
pub mod update_release;
pub mod version;

pub use crate::cli::args::Cli;
pub use crate::cli::dispatch::run;

```

`crates/bonesdeploy/src/commands/pull_state.rs`:

```rs
use std::fs;
use std::io::Write as _;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use std::process::{Command, Stdio};

use crate::config;
use crate::infra::git;
use crate::infra::ssh;
use anyhow::{Context, Result, bail};
use shared::paths;

use crate::commands::init_project;

pub fn run() -> Result<()> {
    git::ensure_git_repository()?;

    let target = resolve_pull_target()?;

    println!("Pulling .bones...");

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    if !bones_dir.exists() {
        let project_name = config::repo_directory_name()?;
        let config_dir = config::bones_config_dir(&project_name);
        if config_dir.exists() && !config_dir.is_dir() {
            fs::remove_file(&config_dir)
                .with_context(|| format!("Stale file at {}, cannot create directory", config_dir.display()))?;
        }
        fs::create_dir_all(&config_dir)?;
        unix_fs::symlink(&config_dir, bones_dir)?;
    }

    let archive = fetch_remote_archive(&target)?;
    clear_managed_bones_entries(bones_dir)?;
    extract_bones_archive(bones_dir, &archive)?;
    init_project::symlink_pre_push()?;

    println!(".bones pulled.");
    println!();
    println!("Next: run bonesdeploy doctor.");
    Ok(())
}

fn resolve_pull_target() -> Result<git::RemoteConnectionDetails> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if bones_toml.exists() {
        let cfg = config::load(bones_toml)?;
        return Ok(git::RemoteConnectionDetails {
            user: cfg.ssh_user,
            host: cfg.host,
            port: cfg.port,
            repo_path: cfg.repo_path,
        });
    }

    let remote_name = resolve_remote_name()?;
    let details = git::infer_remote_connection_details(&remote_name)?
        .with_context(|| format!("Remote '{remote_name}' must use an SSH-style URL ending in .git"))?;

    Ok(git::RemoteConnectionDetails { user: String::from("root"), ..details })
}

fn resolve_remote_name() -> Result<String> {
    if git::remote_exists("production")? {
        return Ok(String::from("production"));
    }

    let remotes = git::list_remotes()?;
    match remotes.as_slice() {
        [] => bail!("No git remotes configured. Add a deployment remote before running bonesdeploy pull."),
        [remote] => Ok(remote.clone()),
        _ => {
            bail!(
                "Multiple git remotes configured. Keep {} or name the deployment remote 'production'.",
                paths::LOCAL_BONES_TOML
            )
        }
    }
}

fn fetch_remote_archive(target: &git::RemoteConnectionDetails) -> Result<Vec<u8>> {
    let site = site_name_from_repo_path(&target.repo_path)?;
    let output = ssh::external_command(&target.user, &target.host, &target.port)
        .arg(format!("bonesremote site export --site '{site}'"))
        .output()
        .context("Failed to export remote site state")?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to export remote site state\n{stderr}")
}

fn clear_managed_bones_entries(bones_dir: &Path) -> Result<()> {
    for name in [paths::BONES_TOML, paths::RUNTIME_TOML, paths::DEPLOYMENT_DIR, paths::HOOKS_DIR] {
        let path = bones_dir.join(name);
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            fs::remove_dir_all(&path).with_context(|| format!("Failed to remove {}", path.display()))?;
        } else {
            fs::remove_file(&path).with_context(|| format!("Failed to remove {}", path.display()))?;
        }
    }
    Ok(())
}

fn extract_bones_archive(bones_dir: &Path, archive: &[u8]) -> Result<()> {
    let mut child = Command::new("tar")
        .args(["-xzf", "-", "-C"])
        .arg(bones_dir)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to run tar for pull")?;

    let mut stdin = child.stdin.take().context("tar stdin was not piped")?;
    stdin.write_all(archive).context("Failed to write pulled archive to tar")?;
    drop(stdin);

    let output = child.wait_with_output().context("Failed to finish extracting .bones")?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to extract .bones\n{stderr}")
}

fn site_name_from_repo_path(repo_path: &str) -> Result<String> {
    let repo_name = Path::new(repo_path)
        .file_name()
        .and_then(|name| name.to_str())
        .context("Remote repo path must end in a site name")?;
    repo_name.strip_suffix(".git").map(ToOwned::to_owned).context("Remote repo path must end in .git")
}

```

`crates/bonesdeploy/src/commands/push_state.rs`:

```rs
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use std::io::Write as _;

use crate::config;
use crate::infra::ssh;
use shared::paths;

pub fn run(show_next: bool) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    println!("Publishing .bones...");
    sync_bones_directory(&cfg).context("Failed to publish .bones.")?;

    println!(".bones published.");
    if show_next {
        println!();
        println!("Next: run bonesdeploy doctor.");
    }

    Ok(())
}

pub(crate) fn sync_bones_directory(cfg: &config::Bones) -> Result<()> {
    let archive = archive_bones_directory()?;
    let mut child = ssh::external_command(&cfg.ssh_user, &cfg.host, &cfg.port)
        .arg(remote_import_command(&cfg.project_name))
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to start remote site import")?;

    let mut stdin = child.stdin.take().context("ssh stdin was not piped")?;
    stdin.write_all(&archive).context("Failed to stream .bones archive to remote host")?;
    drop(stdin);

    let output = child.wait_with_output().context("Failed to finish remote site import")?;
    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to import remote site state\n{stderr}")
}

pub(crate) fn remote_import_command(site: &str) -> String {
    format!("bonesremote site import --site '{site}'")
}

fn archive_bones_directory() -> Result<Vec<u8>> {
    let output = Command::new("tar")
        .args(["-czf", "-", "--exclude", "./secrets", "-C", paths::LOCAL_BONES_DIR, "."])
        .output()
        .context("Failed to run tar for .bones")?;

    if output.status.success() {
        return Ok(output.stdout);
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    bail!("Failed to archive .bones\n{stderr}")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::remote_import_command;
    use shared::paths;

    #[test]
    fn local_secrets_path_stays_under_bones_dir() {
        let path = Path::new(paths::LOCAL_BONES_SECRETS_DIR);
        assert_eq!(path.parent(), Some(Path::new(paths::LOCAL_BONES_DIR)));
    }

    #[test]
    fn remote_import_command_targets_control_plane_import() {
        assert_eq!(remote_import_command("acme"), "bonesremote site import --site 'acme'");
    }
}

```

`crates/bonesdeploy/src/commands/remote_data.rs`:

```rs
use anyhow::Result;
use serde_json::{Map, Value};
use shared::config as shared_config;
use shared::paths::{ssl_certificate_key_path, ssl_certificate_path};

use crate::config;

pub(super) fn base(cfg: &config::Bones, web_root: &str) -> Result<Map<String, Value>> {
    let paths = cfg.deployment_paths(web_root);
    let mut vars = Map::new();

    vars.insert(String::from(shared_config::bonesinfra_input::SSH_PORT), Value::String(cfg.port.clone()));
    vars.insert(
        String::from(shared_config::bonesinfra_input::DEPLOY_USER),
        Value::String(shared_config::default_deploy_user()),
    );
    vars.insert(
        String::from(shared_config::bonesinfra_input::RUNTIME_USER),
        Value::String(shared_config::runtime_user_for(&cfg.project_name)),
    );
    vars.insert(
        String::from(shared_config::bonesinfra_input::RUNTIME_GROUP),
        Value::String(shared_config::runtime_group_for(&cfg.project_name)),
    );
    vars.insert(
        String::from(shared_config::bonesinfra_input::RELEASE_GROUP),
        Value::String(shared_config::release_group_for(&cfg.project_name)),
    );
    vars.insert(String::from("project_root_parent"), Value::String(paths.project_root_parent.clone()));
    vars.insert(String::from(shared_config::bonesinfra_input::PROJECT_ROOT), Value::String(cfg.project_root.clone()));
    vars.insert(String::from("web_root"), Value::String(web_root.to_string()));
    vars.insert(String::from("project_name"), Value::String(cfg.project_name.clone()));
    vars.insert(String::from("preview_domain"), Value::String(cfg.preview_domain.clone()));
    vars.insert(String::from("repo_path"), Value::String(cfg.repo_path.clone()));
    vars.insert(String::from("paths"), serde_json::to_value(paths)?);

    Ok(vars)
}

pub fn ssl(cfg: &config::Bones, web_root: &str, domain: &str, email: &str) -> Result<Value> {
    let mut vars = base(cfg, web_root)?;
    vars.insert(String::from("ssl_domain"), Value::String(domain.to_string()));
    vars.insert(String::from("ssl_email"), Value::String(email.to_string()));
    vars.insert(String::from("nginx_ssl_certificate_path"), Value::String(ssl_certificate_path(domain)));
    vars.insert(String::from("nginx_ssl_certificate_key_path"), Value::String(ssl_certificate_key_path(domain)));
    Ok(Value::Object(vars))
}

#[cfg(test)]
mod tests {
    use crate::config::Bones;

    use super::{base, ssl};

    fn test_cfg() -> Bones {
        Bones {
            project_name: String::from("test"),
            repo_path: String::from("/home/git/test.git"),
            project_root: String::from("/srv/test"),
            host: String::from("example.com"),
            port: String::from("22"),
            branch: String::from("master"),
            remote_name: String::from("production"),
            deploy_on_push: true,
            ..Default::default()
        }
    }

    /// Passes the SSL domain and email into the deploy data sent to bonesinfra
    #[test]
    fn ssl_data_includes_domain_and_email() -> anyhow::Result<()> {
        let cfg = test_cfg();
        let vars = ssl(&cfg, "public", "app.example.com", "ops@example.com")?;

        assert_eq!(vars.get("ssl_domain"), Some(&serde_json::Value::String(String::from("app.example.com"))));
        assert_eq!(vars.get("ssl_email"), Some(&serde_json::Value::String(String::from("ops@example.com"))));
        Ok(())
    }

    #[test]
    fn base_data_includes_preview_domain() -> anyhow::Result<()> {
        let mut cfg = test_cfg();
        cfg.preview_domain = String::from("test-example-com.nip.io");

        let vars = base(&cfg, "public")?;

        assert_eq!(
            vars.get("preview_domain"),
            Some(&serde_json::Value::String(String::from("test-example-com.nip.io")))
        );
        Ok(())
    }
}

```

`crates/bonesdeploy/src/commands/remote_runtime.rs`:

```rs
use std::path::Path;

use anyhow::{Result, bail};

use crate::infra::bonesinfra_cli;
use crate::infra::git;
use crate::ui::prompts;
use shared::paths;

pub fn run(yes: bool) -> Result<()> {
    git::ensure_git_repository()?;

    let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
    if !runtime_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_RUNTIME_TOML);
    }

    if !yes && !prompts::confirm_remote_runtime()? {
        println!("Skipped runtime setup.");
        println!();
        println!("Next: run bonesdeploy remote runtime when ready.");
        return Ok(());
    }

    println!("Applying runtime...");

    bonesinfra_cli::run(&[
        "runtime",
        "apply",
        "--config",
        paths::LOCAL_BONES_TOML,
        "--runtime-config",
        paths::LOCAL_BONES_RUNTIME_TOML,
    ])?;

    println!("Runtime applied.");
    println!();
    println!("Next: run bonesdeploy push.");
    Ok(())
}

```

`crates/bonesdeploy/src/commands/remote_setup.rs`:

```rs
use std::path::Path;

use anyhow::{Context, Result};
use serde_json::Value;

use shared::config as shared_config;
use shared::paths;

use super::remote_data;
use crate::config;
use crate::infra::bonesinfra_cli;
use crate::infra::bootstrap_ssh;

pub fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;

    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));

    println!("Bootstrapping remote server...");

    let mut deploy_data = Value::Object(remote_data::base(&cfg, &runtime.web_root)?);
    let host = cfg.host;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from(shared_config::bonesinfra_input::SSH_USER), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(host));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    bonesinfra_cli::run_with_stdin(&["setup", "apply", "--config", paths::LOCAL_BONES_TOML], &json)?;

    println!("Remote bootstrap complete.");
    println!();
    println!("Next: run bonesdeploy remote runtime.");

    Ok(())
}

```

`crates/bonesdeploy/src/commands/remote_ssl.rs`:

```rs
use std::path::Path;

use anyhow::{Context, Result, bail};
use serde_json::Value;

use shared::config as shared_config;
use shared::paths;

use super::push_state;
use super::remote_data;
use crate::config;
use crate::infra::bonesinfra_cli;
use crate::infra::bootstrap_ssh;
use crate::ui::prompts;

pub fn run(yes: bool, domain: Option<String>, email: Option<String>) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let mut cfg = config::load(bones_toml)?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;

    if let Some(value) = domain {
        cfg.domain = value.trim().to_string();
    } else if cfg.domain.is_empty() && !yes {
        cfg.domain = prompts::prompt_ssl_domain(Some(&cfg))?;
    }

    if let Some(value) = email {
        cfg.email = value.trim().to_string();
    } else if cfg.email.is_empty() && !yes {
        cfg.email = prompts::prompt_ssl_email(Some(&cfg))?;
    }

    if cfg.domain.is_empty() {
        bail!("SSL domain is missing. Pass --domain or set domain in .bones/bones.toml");
    }

    if cfg.email.is_empty() {
        bail!("SSL email is missing. Pass --email or set email in .bones/bones.toml");
    }

    config::save(&cfg, bones_toml)?;

    if !yes && !prompts::confirm_remote_ssl()? {
        println!("Skipped HTTPS setup.");
        println!();
        println!("Next: run bonesdeploy remote ssl when DNS is ready.");
        return Ok(());
    }

    println!("Configuring HTTPS for {}...", cfg.domain);

    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));
    let mut deploy_data = remote_data::ssl(&cfg, &runtime.web_root, &cfg.domain, &cfg.email)?;
    if let Value::Object(ref mut map) = deploy_data {
        map.insert(String::from(shared_config::bonesinfra_input::SSH_USER), Value::String(ssh_user));
        map.insert(String::from("host"), Value::String(cfg.host.clone()));
        map.insert(String::from(shared_config::bonesinfra_input::SSH_PORT), Value::String(cfg.port.clone()));
    }

    let json = serde_json::to_string(&deploy_data).context("Failed to serialize deploy data")?;
    bonesinfra_cli::run_with_stdin(&["ssl", "apply", "--config", paths::LOCAL_BONES_TOML], &json)?;

    cfg.ssl_enabled = true;
    config::save(&cfg, bones_toml)?;
    push_state::sync_bones_directory(&cfg)?;

    println!("HTTPS configured.");
    println!();
    println!("Next: run bonesdeploy deploy.");

    Ok(())
}

```

`crates/bonesdeploy/src/commands/rollback.rs`:

```rs
use std::path::Path;

use anyhow::{Context, Result};

use crate::config;
use crate::infra::ssh;
use shared::paths;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml).context(super::deploy_project::local_bones_load_error())?;

    println!("Rolling back {} on {}...", cfg.project_name, cfg.host);

    let session = ssh::connect_privileged(&cfg).await?;
    let command = format!("bonesremote release rollback --site '{}'", single_quote(&cfg.project_name));
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("Rollback complete.");
    println!();
    println!("Next: run bonesdeploy status.");

    Ok(())
}

fn single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

```

`crates/bonesdeploy/src/commands/secrets.rs`:

```rs
use std::env;
use std::fs::{self, OpenOptions, Permissions};
use std::io::{ErrorKind, Write as IoWrite};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process;
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config;
use crate::infra::{bootstrap_ssh, ssh};
use shared::config as shared_config;
use shared::config::parse_port;
use shared::paths;

const LOCAL_ENV_SECRET: &str = ".bones/secrets/.env.gpg";
const DEFAULT_SECRET_MODE: &str = "640";

fn gpg_home() -> PathBuf {
    paths::bones_config_root().join("gnupg")
}

fn gpg_command() -> Command {
    let mut cmd = Command::new("gpg");
    cmd.arg("--homedir").arg(gpg_home().as_os_str());
    cmd
}

pub fn init() -> Result<()> {
    ensure_gpg_installed()?;

    let bones_dir = Path::new(paths::LOCAL_BONES_DIR);
    if !bones_dir.is_dir() {
        bail!("Missing .bones config\n\nNext: run bonesdeploy init.");
    }

    let secrets_toml = Path::new(".bones/secrets.toml");
    if secrets_toml.exists() {
        bail!("Missing encrypted secrets\n\nNext: run bonesdeploy secrets edit.");
    }

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let _key_fingerprint = ensure_project_key(&cfg.project_name)?;

    fs::create_dir_all(paths::LOCAL_BONES_SECRETS_DIR)
        .with_context(|| format!("Failed to create {}", paths::LOCAL_BONES_SECRETS_DIR))?;

    println!("Secrets initialized.");
    println!();
    println!("Next: run bonesdeploy secrets edit.");
    Ok(())
}

pub fn edit() -> Result<()> {
    ensure_gpg_installed()?;

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let key_fingerprint = ensure_project_key(&cfg.project_name)?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);

    if let Some(parent) = encrypted_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let temp_path = create_temp_edit_path()?;

    if encrypted_path.is_file() {
        run_gpg(&[
            "--batch",
            "--yes",
            "--decrypt",
            "--output",
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
        ])?;
    }

    // ponytail: plaintext briefly exists on local disk during edit; upgrade path is stricter temp-file handling or an in-memory editor flow.
    let edit_result = open_editor(&temp_path);
    let encrypt_result = if edit_result.is_ok() {
        run_gpg(&[
            "--batch",
            "--yes",
            "--output",
            encrypted_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid encrypted path"))?,
            "--encrypt",
            "--recipient",
            &key_fingerprint,
            temp_path.to_str().ok_or_else(|| anyhow::anyhow!("Invalid temp path"))?,
        ])
    } else {
        Ok(())
    };

    let cleanup_result = fs::remove_file(&temp_path);

    edit_result?;
    encrypt_result?;
    if let Err(error) = cleanup_result
        && error.kind() != ErrorKind::NotFound
    {
        eprintln!("Warning: could not remove temporary secret file: {}", temp_path.display());
    }

    println!("Secrets updated.");
    println!();
    println!("Next: run bonesdeploy secrets push.");
    Ok(())
}

pub async fn push() -> Result<()> {
    ensure_gpg_installed()?;

    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;
    let runtime_group = if runtime.runtime_group.is_empty() {
        shared_config::runtime_group_for(&cfg.project_name)
    } else {
        runtime.runtime_group
    };

    let deployment = cfg.deployment_paths(paths::DEFAULT_WEB_ROOT);
    let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as(&ssh_user, &cfg.host, port).await?;

    let encrypted_path = Path::new(LOCAL_ENV_SECRET);
    if !encrypted_path.is_file() {
        bail!("Missing encrypted secrets\n\nNext: run bonesdeploy secrets edit.");
    }

    let plaintext = decrypt_secret(encrypted_path)?;
    let target = Path::new(&deployment.shared).join(paths::DOT_ENV);
    let parent = target.parent().ok_or_else(|| anyhow::anyhow!("Remote target has no parent: {}", target.display()))?;
    let parent_s = shell_quote_single(&parent.display().to_string());
    let target_s = shell_quote_single(&target.display().to_string());
    let group_s = shell_quote_single(&runtime_group);
    let cmd = format!(
        "mkdir -p {parent_s} && tmp=$(mktemp {target_s}.XXXXXX) && cat > \"$tmp\" && chown root:{group_s} \"$tmp\" && chmod {DEFAULT_SECRET_MODE} \"$tmp\" && mv \"$tmp\" {target_s}",
    );

    ssh::run_cmd_with_stdin(&session, &cmd, &plaintext).await?;
    session.close().await?;
    println!("Secrets pushed.");
    Ok(())
}

fn ensure_gpg_installed() -> Result<()> {
    let output = Command::new("gpg").arg("--version").output().context("gpg is required.")?;
    if !output.status.success() {
        bail!("gpg is required.")
    }
    Ok(())
}

fn ensure_gpg_home() -> Result<()> {
    let home = gpg_home();
    fs::create_dir_all(&home).with_context(|| format!("Failed to create {}", home.display()))?;
    fs::set_permissions(&home, Permissions::from_mode(0o700))
        .with_context(|| format!("Failed to chmod 0700 {}", home.display()))?;
    Ok(())
}

fn ensure_project_key(project_name: &str) -> Result<String> {
    ensure_gpg_home()?;

    let uid = format!("BonesDeploy secrets: {project_name}");

    if let Some(fingerprint) = find_key_fingerprint(&uid)? {
        return Ok(fingerprint);
    }

    generate_project_key(project_name, &uid)
}

fn find_key_fingerprint(uid: &str) -> Result<Option<String>> {
    let mut cmd = gpg_command();
    cmd.args(["--list-keys", "--with-colons", "--with-fingerprint", uid]);
    let output = cmd.output().context("Failed to run gpg --list-keys")?;

    if !output.status.success() {
        return Ok(None);
    }

    Ok(extract_fingerprint(&String::from_utf8_lossy(&output.stdout)))
}

fn extract_fingerprint(output: &str) -> Option<String> {
    for line in output.lines() {
        if line.starts_with("fpr:") {
            let parts: Vec<&str> = line.split(':').collect();
            if parts.len() >= 10 {
                return Some(parts[9].to_string());
            }
        }
    }
    None
}

// ponytail: MVP uses an unprotected local project key inside the private
// BonesDeploy GPG home; upgrade path is optional passphrase / OS keychain
// integration.
fn generate_project_key(project_name: &str, uid: &str) -> Result<String> {
    let email = format!("{project_name}@bonesdeploy.local");
    let params = format!(
        "Key-Type: RSA\n\
         Key-Length: 4096\n\
         Key-Usage: cert\n\
         Subkey-Type: RSA\n\
         Subkey-Length: 4096\n\
         Subkey-Usage: encrypt\n\
         Name-Real: {uid}\n\
         Name-Email: {email}\n\
         %no-protection\n\
         %commit\n"
    );

    let mut child = gpg_command()
        .args(["--batch", "--generate-key"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn gpg --generate-key")?;

    {
        let mut stdin = child.stdin.take().ok_or_else(|| anyhow::anyhow!("stdin was not piped"))?;
        stdin.write_all(params.as_bytes()).context("Failed to write batch key params to gpg")?;
    }

    let output = child.wait_with_output().context("Failed to wait for gpg --generate-key")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to generate GPG key: {stderr}");
    }

    find_key_fingerprint(uid)?.ok_or_else(|| anyhow::anyhow!("Key was generated but fingerprint could not be found"))
}

fn open_editor(path: &Path) -> Result<()> {
    let editor = env::var("EDITOR")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| anyhow::anyhow!("$EDITOR is not set. Set it before running `bonesdeploy secrets edit`."))?;

    let status = Command::new("sh")
        .arg("-c")
        .arg("${EDITOR:?EDITOR is not set} \"$1\"")
        .arg("sh")
        .arg(path)
        .env("EDITOR", editor)
        .status()
        .context("Failed to launch editor")?;

    if !status.success() {
        bail!("Editor exited with status {status}");
    }

    Ok(())
}

fn create_temp_edit_path() -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesdeploy-env-{}-{nonce}", process::id()));

    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .with_context(|| format!("Failed to create temp file {}", path.display()))?;

    Ok(path)
}

fn decrypt_secret(path: &Path) -> Result<Vec<u8>> {
    let mut cmd = gpg_command();
    cmd.args(["--batch", "--yes", "--decrypt"]).arg(path);
    let output = cmd.output().with_context(|| format!("Failed to run gpg for {}", path.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to decrypt {}\n{stderr}", path.display());
    }

    Ok(output.stdout)
}

fn run_gpg(args: &[&str]) -> Result<()> {
    let mut cmd = gpg_command();
    cmd.args(args);
    let status = cmd.status().context("Failed to run gpg")?;
    if !status.success() {
        bail!("gpg failed with status {status}");
    }
    Ok(())
}

fn shell_quote_single(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

#[cfg(test)]
mod tests {
    use super::{extract_fingerprint, gpg_home};
    use shared::paths;

    #[test]
    fn gpg_home_resolves_under_bones_config_root() {
        assert_eq!(gpg_home(), paths::bones_config_root().join("gnupg"));
    }

    #[test]
    fn extract_fingerprint_parses_fpr_line() {
        let output = "tru::1:1754651437:0:3:1:3\nfpr:::::::::ABCDEF1234567890ABCDEF1234567890ABCDEF:\nuid:::::::::Test <test@example.com>:\n";
        assert_eq!(extract_fingerprint(output).as_deref(), Some("ABCDEF1234567890ABCDEF1234567890ABCDEF"));
    }

    #[test]
    fn extract_fingerprint_returns_none_without_fpr_line() {
        let output = "tru::1:1754651437:0:3:1:3\nuid:::::::::Test <test@example.com>:\n";
        assert_eq!(extract_fingerprint(output), None);
    }
}

```

`crates/bonesdeploy/src/commands/setup.rs`:

```rs
use std::path::Path;

use anyhow::{Context, Result};
use shared::paths;

use crate::commands::{doctor, push_state, remote_runtime, remote_setup};
use crate::config;

pub async fn run(_yes: bool) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    println!("Setting up deployment...");

    remote_setup::run().with_context(|| setup_error("bootstrapping remote server"))?;
    remote_runtime::run(true).with_context(|| setup_error("applying runtime"))?;
    push_state::run(false).with_context(|| setup_error("syncing .bones"))?;
    doctor::run(false).await.with_context(|| setup_error("checking deployment"))?;

    println!();
    println!("Setup complete.");
    println!();
    if cfg.ssl_enabled {
        println!("Next: run bonesdeploy deploy.");
    } else {
        println!("Next: run bonesdeploy remote ssl to configure HTTPS.");
    }

    Ok(())
}

fn setup_error(step: &str) -> String {
    format!("Setup failed while {step}.\n\nNext: fix the error above, then run bonesdeploy setup again.")
}

```

`crates/bonesdeploy/src/commands/status.rs`:

```rs
use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use shared::config as shared_config;
use shared::paths;

use crate::commands::guide;
use crate::config;
use crate::infra::ssh;

#[derive(Debug, Deserialize)]
struct RemoteReport {
    current_release: String,
    ssl: RemoteSslStatus,
    services: Vec<RemoteServiceStatus>,
}

#[derive(Debug, Deserialize)]
struct RemoteSslStatus {
    enabled: bool,
    domain: String,
}

#[derive(Debug, Deserialize)]
struct RemoteServiceStatus {
    name: String,
    state: String,
    enabled: String,
}

pub async fn run() -> Result<()> {
    let report = guide::build_report().await?;
    let cfg = report.cfg.as_ref();

    println!("Project: {}", report.project);

    if let Some(cfg) = cfg {
        println!("Host: {}", cfg.host);
        println!("Branch: {}", cfg.branch);
    }

    println!("State: {}", report.state_label);

    let remote = if let Some(cfg) = cfg {
        remote_status(cfg).await.unwrap_or_else(|_| fallback_remote_status(cfg))
    } else {
        empty_remote_status()
    };

    println!("Current release: {}", remote.current_release);
    println!("SSL: {}", ssl_state(&remote.ssl));
    println!();
    println!("Services:");
    for service in &remote.services {
        println!("{} {} {}/{}", service_marker(&service.state), service.name, service.state, service.enabled);
    }
    println!();
    println!("Next: {}", report.commands[0]);

    Ok(())
}

async fn remote_status(cfg: &config::Bones) -> Result<RemoteReport> {
    let session = ssh::connect_privileged(cfg).await?;
    let command = format!("bonesremote status --site '{}'", shell_quote(&cfg.project_name));
    let output = ssh::run_cmd(&session, &command).await;
    session.close().await?;

    Ok(serde_json::from_str(&output?)?)
}

fn fallback_remote_status(cfg: &config::Bones) -> RemoteReport {
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR)).ok();
    let current_release = runtime.as_ref().map_or_else(
        || String::from("unknown"),
        |runtime| {
            let deployment = cfg.deployment_paths(&runtime.web_root);
            release_name(&deployment.current)
        },
    );

    RemoteReport {
        current_release,
        ssl: RemoteSslStatus { enabled: cfg.ssl_enabled, domain: cfg.domain.clone() },
        services: vec![RemoteServiceStatus {
            name: paths::nginx_service_name(&cfg.project_name),
            state: String::from("unknown"),
            enabled: String::from("unknown"),
        }],
    }
}

fn empty_remote_status() -> RemoteReport {
    RemoteReport {
        current_release: String::from("unknown"),
        ssl: RemoteSslStatus { enabled: false, domain: String::new() },
        services: Vec::new(),
    }
}

fn ssl_state(ssl: &RemoteSslStatus) -> String {
    if ssl.enabled {
        if ssl.domain.is_empty() { String::from("enabled") } else { format!("enabled ({})", ssl.domain) }
    } else {
        String::from("disabled")
    }
}

fn service_marker(state: &str) -> &'static str {
    if state == "active" { "✓" } else { "✗" }
}

fn release_name(value: &str) -> String {
    Path::new(value).file_name().map_or_else(|| value.to_string(), |name| name.to_string_lossy().to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{RemoteSslStatus, service_marker, ssl_state};

    #[test]
    fn ssl_state_uses_remote_state_when_local_flag_is_stale() {
        let ssl = RemoteSslStatus { domain: String::from("app.example.com"), enabled: true };

        assert_eq!(ssl_state(&ssl), "enabled (app.example.com)");
    }

    #[test]
    fn service_marker_marks_only_active_as_ok() {
        assert_eq!(service_marker("active"), "✓");
        assert_eq!(service_marker("failed"), "✗");
        assert_eq!(service_marker("unknown"), "✗");
    }
}

```

`crates/bonesdeploy/src/commands/tests/test_init_project.rs`:

```rs
use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use super::{collect_non_interactive, run_with_prefetch};
use crate::commands::init_config::InitArgs;

use anyhow::{Result, bail};
use shared::paths;
use tempfile::TempDir;

use crate::config::Bones;

fn test_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

struct TestEnvironment {
    _lock: MutexGuard<'static, ()>,
    original_dir: PathBuf,
    original_home: Option<String>,
    original_xdg_config_home: Option<String>,
}

impl TestEnvironment {
    fn enter(repo_dir: &Path, home_dir: &Path) -> Result<Self> {
        let lock = test_lock().lock().map_err(|_| anyhow::anyhow!("test lock poisoned"))?;
        let original_dir = env::current_dir()?;
        let original_home = env::var("HOME").ok();
        let original_xdg_config_home = env::var("XDG_CONFIG_HOME").ok();

        env::set_current_dir(repo_dir)?;

        // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
        unsafe {
            env::set_var("HOME", home_dir);
            env::set_var("XDG_CONFIG_HOME", home_dir.join(".config"));
        }

        Ok(Self { _lock: lock, original_dir, original_home, original_xdg_config_home })
    }
}

impl Drop for TestEnvironment {
    fn drop(&mut self) {
        let _ = env::set_current_dir(&self.original_dir);

        match &self.original_home {
            Some(home) => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::set_var("HOME", home);
                }
            }
            None => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::remove_var("HOME");
                }
            }
        }

        match &self.original_xdg_config_home {
            Some(home) => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::set_var("XDG_CONFIG_HOME", home);
                }
            }
            None => {
                // Safety: these tests serialize access with a process-wide mutex and restore env vars on drop.
                unsafe {
                    env::remove_var("XDG_CONFIG_HOME");
                }
            }
        }
    }
}

fn init_args() -> InitArgs {
    InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: Some(String::from("atlas")),
        branch: None,
        remote: None,
        host: Some(String::from("deploy.example.com")),
        port: None,
    }
}

fn run_init() -> Result<bool> {
    run_with_prefetch(&init_args(), || Ok(()))
}

fn create_git_repo(path: &Path) -> Result<()> {
    let status = Command::new("git").args(["init", "--quiet"]).current_dir(path).status()?;
    if !status.success() {
        bail!("git init failed with status {status}");
    }
    Ok(())
}

fn with_temp_repo(test: impl FnOnce(&Path, &Path) -> Result<()>) -> Result<()> {
    let temp = TempDir::new()?;
    let repo_dir = temp.path().join("repo");
    let home_dir = temp.path().join("home");

    fs::create_dir_all(&repo_dir)?;
    fs::create_dir_all(&home_dir)?;
    create_git_repo(&repo_dir)?;

    let _environment = TestEnvironment::enter(&repo_dir, &home_dir)?;

    test(&repo_dir, &home_dir)
}

fn incomplete_existing(project_name: &str) -> Bones {
    Bones {
        remote_name: String::from("production"),
        project_name: String::from(project_name),
        host: String::new(),
        port: String::from("22"),
        repo_path: String::new(),
        project_root: String::new(),
        branch: String::from("main"),
        deploy_on_push: true,
        ..Default::default()
    }
}

/// Uses existing config and CLI values without prompting when non-interactive mode is active.
#[test]
fn collect_non_interactive_uses_existing_and_cli_values_without_prompting() -> Result<()> {
    let existing = incomplete_existing("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: Some(String::from("deploy.example.com")),
        port: None,
    };

    let cfg = collect_non_interactive("workspace", Some(&existing), &args)?;

    assert_eq!(cfg.project_name, "atlas");
    assert_eq!(cfg.host, "deploy.example.com");
    assert_eq!(cfg.branch, "main");
    assert_eq!(cfg.remote_name, "production");
    assert_eq!(cfg.repo_path, paths::default_repo_path_for("atlas"));

    Ok(())
}

/// Requires a host when neither existing config nor CLI provide one.
#[test]
fn collect_non_interactive_requires_host_when_existing_and_cli_are_missing_it() -> Result<()> {
    let existing = incomplete_existing("atlas");
    let args = InitArgs {
        non_interactive: true,
        setup_remote: false,
        project_name: None,
        branch: None,
        remote: None,
        host: None,
        port: None,
    };

    let result = collect_non_interactive("workspace", Some(&existing), &args);
    let Err(err) = result else {
        bail!("missing host should fail");
    };
    assert!(err.to_string().contains("--host is required"));

    Ok(())
}

/// Materializes the base bonesdeploy kit and runtime config during init.
#[test]
fn init_materializes_base_bones_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run_init()?;

        let bones_dir = repo_dir.join(".bones");
        assert!(bones_dir.join("bones.toml").is_file());
        assert!(bones_dir.join("runtime.toml").is_file());
        assert!(bones_dir.join("hooks/pre-push").is_file());
        assert!(bones_dir.join("hooks/post-receive").is_file());
        let deploy_dir = bones_dir.join("deployment");
        assert!(deploy_dir.is_dir());
        assert!(deploy_dir.read_dir()?.next().is_some(), "deployment directory should have scripts");
        let runtime_toml = fs::read_to_string(bones_dir.join("runtime.toml"))?;
        assert!(runtime_toml.contains("runtime_user = \"atlas\""));
        assert!(runtime_toml.contains("permissions"));

        let config_root = paths::bones_config_root().join("atlas.bones");
        assert!(config_root.join("hooks/pre-push").is_file());
        assert!(config_root.join("hooks/post-receive").is_file());

        let config_gitignore = paths::bones_config_root().join(".gitignore");
        assert!(config_gitignore.is_file());
        let gitignore_content = fs::read_to_string(config_gitignore)?;
        assert!(gitignore_content.contains("gnupg"));
        assert!(gitignore_content.contains("atlas.bones"));

        Ok(())
    })
}

/// Keeps an already materialized local bones scaffold intact when init is run again.
#[test]
fn init_rerun_preserves_existing_bones_assets() -> Result<()> {
    with_temp_repo(|repo_dir, _home_dir| {
        run_init()?;

        let sentinel = repo_dir.join(".bones/hooks/pre-push");
        let original = fs::read_to_string(&sentinel)?;

        run_init()?;

        assert!(sentinel.is_file());
        assert_eq!(fs::read_to_string(&sentinel)?, original);

        Ok(())
    })
}

/// Repairs a dangling .bones symlink instead of failing with EEXIST.
#[test]
fn init_repairs_dangling_bones_symlink() -> Result<()> {
    with_temp_repo(|repo_dir, home_dir| {
        let config_root = home_dir.join(".config/bonesdeploy");
        fs::create_dir_all(&config_root)?;
        symlink(config_root.join("missing.bones"), repo_dir.join(".bones"))?;

        run_init()?;

        let bones_dir = repo_dir.join(".bones");
        assert!(bones_dir.join("bones.toml").is_file());
        assert_eq!(fs::read_link(&bones_dir)?, paths::bones_config_root().join("atlas.bones"));

        Ok(())
    })
}

```

`crates/bonesdeploy/src/commands/update.rs`:

```rs
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use tempfile::TempDir;

use super::update_release;
use shared::paths;

const SOURCE_REPO_URL: &str = "https://github.com/AlextheYounga/bonesdeploy.git";
const SOURCE_BRANCH: &str = "master";

#[derive(Clone, Copy)]
pub struct Options {
    pub skip_local: bool,
    pub skip_remote: bool,
}

pub async fn run(options: Options) -> Result<()> {
    println!("Checking for updates...");
    let current_local = update_release::current_local_version();
    let current_remote = update_release::current_remote_version();

    if options.skip_local && options.skip_remote {
        println!("Already up to date.");
        return Ok(());
    }

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    let source_dir = clone_master_source(temp_path)?;
    let master_versions = read_master_versions(&source_dir)?;

    let mut updated = false;

    if !options.skip_local {
        if current_local != master_versions.bonesdeploy {
            println!("Updating bonesdeploy...");
            update_release::update_local_from_source(SOURCE_REPO_URL)?;
            updated = true;
        }

        refresh_local_bones_from_source(&source_dir, Path::new(paths::LOCAL_BONES_DIR))?;
    }

    if !options.skip_remote && current_remote != master_versions.bonesremote {
        println!("Updating bonesremote...");
        update_release::update_remote_from_source(SOURCE_REPO_URL, &master_versions.bonesremote).await?;
        updated = true;
    }

    if updated {
        println!("Update complete.");
    } else {
        println!("Already up to date.");
    }

    Ok(())
}

fn clone_master_source(temp_path: &Path) -> Result<PathBuf> {
    let source_dir = temp_path.join("source");

    let clone_status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", SOURCE_BRANCH, SOURCE_REPO_URL])
        .arg(&source_dir)
        .status()
        .context("Failed to clone bonesdeploy repository")?;

    if !clone_status.success() {
        bail!("Failed to clone {SOURCE_REPO_URL} branch {SOURCE_BRANCH}");
    }

    Ok(source_dir)
}

struct MasterVersions {
    bonesdeploy: String,
    bonesremote: String,
}

fn read_master_versions(source_dir: &Path) -> Result<MasterVersions> {
    let bonesdeploy = read_package_version(&source_dir.join("crates/bonesdeploy/Cargo.toml"))?;
    let bonesremote = read_package_version(&source_dir.join("crates/bonesremote/Cargo.toml"))?;

    Ok(MasterVersions { bonesdeploy, bonesremote })
}

fn read_package_version(manifest: &Path) -> Result<String> {
    let content = fs::read_to_string(manifest).with_context(|| format!("Failed to read {}", manifest.display()))?;
    parse_package_version(&content)
        .with_context(|| format!("Failed to parse package version from {}", manifest.display()))
}

fn parse_package_version(manifest: &str) -> Result<String> {
    let value: toml::Value = toml::from_str(manifest)?;
    value
        .get("package")
        .and_then(|package| package.get("version"))
        .and_then(toml::Value::as_str)
        .filter(|version| !version.is_empty())
        .map(String::from)
        .ok_or_else(|| anyhow::anyhow!("missing [package] version"))
}

fn refresh_local_bones_from_source(source_dir: &Path, bones_dir: &Path) -> Result<()> {
    if !bones_dir.exists() {
        return Ok(());
    }

    let kit_root = source_dir.join("crates/bonesdeploy/kit");
    sync_tree(&kit_root.join("hooks"), &bones_dir.join("hooks"), true)?;
    sync_tree(&deployment_source_root(source_dir, bones_dir), &bones_dir.join("deployment"), true)?;

    Ok(())
}

fn deployment_source_root(source_dir: &Path, bones_dir: &Path) -> PathBuf {
    let runtime_toml = bones_dir.join(paths::RUNTIME_TOML);
    let Some(template) = selected_runtime_template(&runtime_toml) else {
        return source_dir.join("crates/bonesdeploy/kit/deployment");
    };

    let runtime_deployment = source_dir.join("crates/bonesdeploy/runtimes").join(template).join("deployment");
    if runtime_deployment.is_dir() { runtime_deployment } else { source_dir.join("crates/bonesdeploy/kit/deployment") }
}

fn selected_runtime_template(runtime_toml: &Path) -> Option<String> {
    let content = fs::read_to_string(runtime_toml).ok()?;
    let value: toml::Value = toml::from_str(&content).ok()?;
    value.get("template")?.as_str().map(String::from)
}

fn sync_tree(source_root: &Path, dest_root: &Path, executable: bool) -> Result<()> {
    if !source_root.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(source_root).with_context(|| format!("Failed to read {}", source_root.display()))? {
        let entry = entry.with_context(|| format!("Failed to read entry in {}", source_root.display()))?;
        let file_type =
            entry.file_type().with_context(|| format!("Failed to read file type for {}", entry.path().display()))?;
        let source_path = entry.path();
        let dest_path = dest_root.join(entry.file_name());

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path).with_context(|| format!("Failed to create {}", dest_path.display()))?;
            sync_tree(&source_path, &dest_path, executable)?;
            continue;
        }

        copy_file(&source_path, &dest_path, executable)?;
    }

    Ok(())
}

fn copy_file(source: &Path, dest: &Path, executable: bool) -> Result<()> {
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::copy(source, dest).with_context(|| format!("Failed to copy {} to {}", source.display(), dest.display()))?;

    if executable {
        fs::set_permissions(dest, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;

    use anyhow::Result;
    use tempfile::TempDir;

    use super::{SOURCE_BRANCH, SOURCE_REPO_URL, parse_package_version, refresh_local_bones_from_source};

    /// Verifies the update source repository and branch constants are set to the canonical values.
    #[test]
    fn update_uses_master_branch_source_repository() {
        assert_eq!(SOURCE_REPO_URL, "https://github.com/AlextheYounga/bonesdeploy.git");
        assert_eq!(SOURCE_BRANCH, "master");
    }

    /// Extracts the package version from the `[package]` section of a Cargo manifest.
    #[test]
    fn parses_package_version_from_manifest_package_section() -> anyhow::Result<()> {
        let manifest = r#"
[package]
name = "bonesdeploy"
version = "0.2.8"
edition = "2024"

[dependencies]
version = "not-this"
"#;

        assert_eq!(parse_package_version(manifest)?, "0.2.8");
        Ok(())
    }

    /// Returns an error when the manifest has no `[package]` section with a version field.
    #[test]
    fn rejects_manifest_without_package_version() {
        let result = parse_package_version("[dependencies]\nversion = \"0.2.8\"\n");
        assert!(result.is_err());
    }

    /// Refreshes .bones scaffold assets from the cloned source tree without overwriting local config files.
    #[test]
    fn refresh_local_bones_updates_scaffold_without_touching_configs() -> Result<()> {
        let temp = TempDir::new()?;
        let source_dir = temp.path().join("source");
        let bones_dir = temp.path().join(".bones");

        write(&source_dir.join("crates/bonesdeploy/kit/hooks/pre-push"), "new hook")?;
        write(&source_dir.join("crates/bonesdeploy/kit/deployment/build/01_build.sh"), "generic deploy")?;
        write(&source_dir.join("crates/bonesdeploy/runtimes/laravel/deployment/build/01_build.sh"), "laravel deploy")?;

        write(&bones_dir.join("bones.toml"), "keep = 'config'\n")?;
        write(&bones_dir.join("runtime.toml"), "template = 'laravel'\n")?;

        refresh_local_bones_from_source(&source_dir, &bones_dir)?;

        assert_eq!(fs::read_to_string(bones_dir.join("bones.toml"))?, "keep = 'config'\n");
        assert_eq!(fs::read_to_string(bones_dir.join("runtime.toml"))?, "template = 'laravel'\n");
        assert_eq!(fs::read_to_string(bones_dir.join("hooks/pre-push"))?, "new hook");
        assert_eq!(fs::read_to_string(bones_dir.join("deployment/build/01_build.sh"))?, "laravel deploy");

        let hook_mode = fs::metadata(bones_dir.join("hooks/pre-push"))?.permissions().mode() & 0o777;
        let deploy_mode = fs::metadata(bones_dir.join("deployment/build/01_build.sh"))?.permissions().mode() & 0o777;
        assert_eq!(hook_mode, 0o755);
        assert_eq!(deploy_mode, 0o755);

        Ok(())
    }

    fn write(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }
}

```

`crates/bonesdeploy/src/commands/update_release.rs`:

```rs
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::config;
use crate::infra::ssh;
use shared::config::{default_deploy_user, parse_port};

pub fn current_local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn current_remote_version() -> String {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if !bones_toml.exists() {
        return String::from("unknown");
    }

    let Ok(cfg) = config::load(bones_toml) else {
        return String::from("unknown");
    };

    let host = format!("{}@{}", default_deploy_user(), cfg.host);
    let output = Command::new("ssh").args(["-p", &cfg.port]).args([&host, "bonesremote", "version"]).output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().strip_prefix("bonesremote ").unwrap_or("unknown").to_string()
        }
        _ => String::from("unknown"),
    }
}

pub fn update_local_from_source(repo_url: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args(["install", "--locked", "--git", repo_url, paths::BONESDEPLOY_BINARY, "--force"])
        .status()
        .context("Failed to run cargo install for bonesdeploy")?;

    if !status.success() {
        bail!("Failed to install bonesdeploy from {repo_url}");
    }

    Ok(())
}

pub async fn update_remote_from_source(repo_url: &str, _version: &str) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if !bones_toml.exists() {
        bail!("No {} found. Run from a bonesdeploy project directory.", paths::LOCAL_BONES_TOML);
    }

    let cfg = config::load(bones_toml)?;
    let port = parse_port(&cfg.port)?;
    let session = ssh::connect_as("root", &cfg.host, port).await?;

    let install_root = paths::USR_LOCAL_BIN.trim_end_matches("/bin");
    ssh::stream_cmd(
        &session,
        &format!("cargo install --locked --git {repo_url} bonesremote --force --root {install_root}"),
    )
    .await?;

    ssh::stream_cmd(
        &session,
        &format!(
            "mkdir -p {root} && chown root:root {root} && chmod 711 {root}",
            root = paths::DEFAULT_PROJECT_ROOT_PARENT
        ),
    )
    .await?;

    session.close().await?;

    Ok(())
}

```

`crates/bonesdeploy/src/commands/version.rs`:

```rs
pub fn run() {
    println!("bonesdeploy {}", env!("CARGO_PKG_VERSION"));
}

```

`crates/bonesdeploy/src/config.rs`:

```rs
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde_json::{Map, Value};
use shared::config as shared_config;
use shared::paths;
use toml_edit::ser::to_string as toml_to_string;

pub use shared::config::{Bones, load};

pub fn is_configured(config: &Bones) -> bool {
    !config.remote_name.is_empty()
        && !config.project_name.is_empty()
        && !config.host.is_empty()
        && !config.repo_path.is_empty()
}

pub fn default_project_root_for(project_name: &str) -> String {
    paths::default_project_root_for(project_name)
}

pub fn bones_config_dir(project_name: &str) -> PathBuf {
    paths::bones_config_root().join(format!("{project_name}.bones"))
}

pub fn repo_directory_name() -> Result<String> {
    let cwd = env::current_dir()?;
    Ok(cwd.file_name().map_or_else(|| String::from("project"), |n| n.to_string_lossy().to_string()))
}

pub fn save(config: &Bones, path: &Path) -> Result<()> {
    let mut to_serialize = config.clone();
    shared_config::apply_derived_defaults(&mut to_serialize);

    let raw = toml_to_string(&to_serialize).context("Failed to serialize bones config")?;
    let toml_str = oxc_toml::format(&raw, oxc_toml::Options::default());
    fs::write(path, toml_str).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

pub fn save_runtime(runtime: &Map<String, Value>, path: &Path) -> Result<()> {
    let raw = toml_to_string(runtime).context("Failed to serialize runtime config")?;
    let toml_str = oxc_toml::format(&raw, oxc_toml::Options::default());
    fs::write(path, toml_str).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env::temp_dir;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;

    use super::{Bones, default_project_root_for, save};
    use shared::config::load;

    fn temp_path(file_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        temp_dir().join(format!("bonesdeploy_config_test_{}_{}_{}", process::id(), nanos, file_name))
    }

    fn minimal_toml(project_name: &str) -> String {
        format!(
            "remote_name = \"production\"\nproject_name = \"{project_name}\"\nhost = \"deploy.example.com\"\nport = \"22\"\nrepo_path = \"{}\"\nbranch = \"master\"\ndeploy_on_push = true\n",
            paths::default_repo_path_for(project_name)
        )
    }

    fn sample_config(project_name: &str) -> Bones {
        Bones {
            remote_name: String::from("production"),
            project_name: String::from(project_name),
            host: String::from("deploy.example.com"),
            port: String::from("22"),
            repo_path: paths::default_repo_path_for(project_name),
            project_root: default_project_root_for(project_name),
            branch: String::from("master"),
            deploy_on_push: true,
            ..Default::default()
        }
    }

    #[test]
    fn load_applies_default_repo_path_from_project_name() -> Result<()> {
        let path = temp_path("repo_path.toml");
        fs::write(&path, minimal_toml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.repo_path, paths::default_repo_path_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn load_applies_default_project_root_from_project_name() -> Result<()> {
        let path = temp_path("project_root.toml");
        fs::write(&path, minimal_toml("atlas"))?;

        let cfg = load(&path)?;
        assert_eq!(cfg.project_root, paths::default_project_root_for("atlas"));

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn save_includes_derived_repo_and_project_root() -> Result<()> {
        let config = sample_config("phoenix");
        let path = temp_path("save_derived_defaults.toml");

        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(content.contains("repo_path ="), "save should include repo_path");
        assert!(content.contains("project_root ="), "save should include project_root");

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn save_persists_ssl_settings() -> Result<()> {
        let mut config = sample_config("phoenix");
        config.ssl_enabled = true;
        config.domain = String::from("app.example.com");
        config.email = String::from("ops@example.com");

        let path = temp_path("save_ssl_settings.toml");
        save(&config, &path)?;
        let content = fs::read_to_string(&path)?;

        assert!(content.contains("ssl_enabled = true"));
        assert!(content.contains("domain = \"app.example.com\""));
        assert!(content.contains("email = \"ops@example.com\""));

        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn load_preserves_explicit_repo_and_project_root_overrides() -> Result<()> {
        let path = temp_path("overrides.toml");
        let toml = format!(
            "remote_name = \"production\"\nproject_name = \"app\"\nhost = \"deploy.example.com\"\nport = \"22\"\nrepo_path = \"{}\"\nproject_root = \"/custom/deploy\"\nbranch = \"master\"\ndeploy_on_push = false\n",
            paths::default_repo_path_for("app")
        );

        fs::write(&path, toml)?;
        let cfg = load(Path::new(&path))?;

        assert_eq!(cfg.repo_path, paths::default_repo_path_for("app"));
        assert_eq!(cfg.project_root, "/custom/deploy");

        fs::remove_file(path)?;
        Ok(())
    }
}

```

`crates/bonesdeploy/src/infra/bonesinfra.rs`:

```rs
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

const REPOSITORY_URL: &str = "https://github.com/AlextheYounga/bonesinfra.git";
const CHECKOUT_DIR: &str = "bonesinfra";

pub fn prefetch() -> Result<()> {
    ensure_available().map(|_| ())
}

pub(super) fn executable_path() -> Result<PathBuf> {
    ensure_available()
}

fn ensure_available() -> Result<PathBuf> {
    let checkout = checkout_dir();
    let venv_python = checkout.join(".venv").join("bin").join("python");

    if let Ok(metadata) = fs::symlink_metadata(&checkout)
        && !metadata.file_type().is_dir()
    {
        reset_checkout(&checkout)?;
    }

    if !checkout.is_dir() {
        install_checkout(&checkout)?;
    }

    if venv_python.is_file() {
        return Ok(venv_python);
    }

    setup_venv(&checkout)?;

    if venv_python.is_file() {
        return Ok(venv_python);
    }

    let contents: Vec<_> = fs::read_dir(&checkout)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path().display().to_string())
        .collect();
    if contents.is_empty() {
        bail!("bonesinfra setup finished but checkout is empty at {}.", checkout.display());
    }

    bail!(
        "bonesinfra setup finished at {}, but {} is missing.\nContents of checkout:\n  {}",
        checkout.display(),
        venv_python.display(),
        contents.join("\n  ")
    );
}

fn reset_checkout(checkout: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(checkout)
        .with_context(|| format!("Failed to inspect stale bonesinfra checkout at {}", checkout.display()))?;

    if metadata.file_type().is_dir() {
        fs::remove_dir_all(checkout)
            .with_context(|| format!("Failed to remove stale bonesinfra checkout at {}", checkout.display()))?;
        return Ok(());
    }

    fs::remove_file(checkout)
        .with_context(|| format!("Failed to remove stale bonesinfra checkout at {}", checkout.display()))?;
    Ok(())
}

fn install_checkout(checkout: &Path) -> Result<()> {
    if let Some(parent) = checkout.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let status = Command::new("git")
        .args(["clone", "--depth", "1", REPOSITORY_URL, &checkout.to_string_lossy()])
        .status()
        .context("Failed to run git clone for bonesinfra install")?;

    if !status.success() {
        bail!("Failed to install bonesinfra from {} into {}.", REPOSITORY_URL, checkout.display());
    }

    Ok(())
}

fn setup_venv(checkout: &Path) -> Result<()> {
    let venv_python = checkout.join(".venv").join("bin").join("python");

    if !venv_python.is_file() {
        let status = Command::new("python3")
            .args(["-m", "venv", ".venv"])
            .current_dir(checkout)
            .status()
            .with_context(|| format!("Failed to create venv in {}", checkout.display()))?;

        if !status.success() {
            bail!("Failed to create venv in {}.", checkout.display());
        }
    }

    let status = Command::new(&venv_python)
        .args(["-m", "pip", "install", "--upgrade", "pip"])
        .status()
        .with_context(|| format!("Failed to upgrade pip in {}", checkout.display()))?;

    if !status.success() {
        bail!("Failed to upgrade pip in {}.", checkout.display());
    }

    let status = Command::new(&venv_python)
        .args(["-m", "pip", "install", "-e", "."])
        .current_dir(checkout)
        .status()
        .with_context(|| format!("Failed to install bonesinfra dependencies in {}", checkout.display()))?;

    if !status.success() {
        bail!("Failed to install bonesinfra dependencies in {}.", checkout.display());
    }

    Ok(())
}

fn checkout_dir() -> PathBuf {
    paths::bones_config_root().join(CHECKOUT_DIR)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use shared::paths;
    use tempfile::TempDir;

    use super::{checkout_dir, reset_checkout};

    #[test]
    fn checkout_dir_lives_under_bones_config_root() {
        assert_eq!(checkout_dir(), paths::bones_config_root().join("bonesinfra"));
    }

    #[test]
    fn reset_checkout_removes_stale_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let checkout = temp_dir.path().join("bonesinfra");
        fs::create_dir_all(checkout.join("nested"))?;

        reset_checkout(&checkout)?;

        assert!(!checkout.exists());
        Ok(())
    }

    #[test]
    fn reset_checkout_removes_stale_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let checkout = temp_dir.path().join("bonesinfra");
        fs::write(&checkout, "stale")?;

        reset_checkout(&checkout)?;

        assert!(!checkout.exists());
        Ok(())
    }
}

```

`crates/bonesdeploy/src/infra/bonesinfra_cli.rs`:

```rs
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::Value;

pub fn run(args: &[&str]) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    run_interactive(&executable, args, None)
}

pub fn run_with_stdin(args: &[&str], stdin_json: &str) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    run_interactive(&executable, args, Some(stdin_json))
}
pub fn run_json(args: &[&str]) -> Result<Value> {
    let executable = super::bonesinfra::executable_path()?;
    let stdout = run_captured(&executable, args)?;
    parse_json_output(&stdout)
}

fn parse_json_output(stdout: &str) -> Result<Value> {
    serde_json::from_str(stdout).context("Failed to parse JSON output from bonesinfra")
}

fn base_command(executable: &Path) -> Command {
    let mut cmd = Command::new(executable);
    cmd.args(["-m", "bonesinfra"]);
    cmd
}

fn run_interactive(executable: &Path, args: &[&str], stdin_json: Option<&str>) -> Result<String> {
    let mut command = base_command(executable);
    command.args(args);
    if stdin_json.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if let Some(stdin_json) = stdin_json {
        let mut stdin = child.stdin.take().context("Failed to capture bonesinfra stdin")?;
        stdin.write_all(stdin_json.as_bytes()).context("Failed to write JSON data to bonesinfra stdin")?;
    }

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait on bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !status.success() {
        bail!("bonesinfra failed");
    }

    Ok(String::new())
}

fn run_captured(executable: &Path, args: &[&str]) -> Result<String> {
    let output = base_command(executable)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("bonesinfra failed:\n{}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Returns the questions for a given runtime from bonesinfra.
pub fn runtime_questions(runtime: &str) -> Result<Value> {
    run_json(&["runtime", "questions", runtime])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Result;

    use super::{base_command, parse_json_output};

    #[test]
    fn base_command_launches_venv_python_with_module_flag() {
        let command = base_command(Path::new("/tmp/bonesinfra/.venv/bin/python"));

        assert_eq!(command.get_program().to_string_lossy(), "/tmp/bonesinfra/.venv/bin/python");
        let args: Vec<_> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert_eq!(args, vec!["-m", "bonesinfra"]);
    }

    #[test]
    fn parse_json_output_reads_cli_stdout() -> Result<()> {
        let parsed = parse_json_output("[\"django\",\"rails\"]")?;
        assert_eq!(parsed, serde_json::json!(["django", "rails"]));
        Ok(())
    }
}

```

`crates/bonesdeploy/src/infra/bootstrap_ssh.rs`:

```rs
use std::env;

pub(crate) fn resolve(config_ssh_user: Option<&str>) -> String {
    // env override takes highest precedence
    if let Ok(env_user) = env::var("BONES_BOOTSTRAP_SSH_USER") {
        let trimmed = env_user.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }
    resolve_from(config_ssh_user)
}

fn resolve_from(value: Option<&str>) -> String {
    value
        .and_then(|raw| {
            let trimmed = raw.trim().to_string();
            if trimmed.is_empty() { None } else { Some(trimmed) }
        })
        .unwrap_or_else(|| String::from("root"))
}

#[cfg(test)]
mod tests {
    use super::resolve_from;

    #[test]
    fn defaults_to_root() {
        let user = resolve_from(None);
        assert_eq!(user, "root");
    }

    #[test]
    fn uses_config_value() {
        let user = resolve_from(Some("ubuntu"));
        assert_eq!(user, "ubuntu");
    }

    #[test]
    fn trims_and_rejects_blank_values() {
        let user = resolve_from(Some("   "));
        assert_eq!(user, "root");

        let user = resolve_from(Some("  ubuntu  "));
        assert_eq!(user, "ubuntu");
    }
}

```

`crates/bonesdeploy/src/infra/embedded.rs`:

```rs
use std::collections::BTreeSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::str;

use anyhow::{Context, Result, anyhow, bail};
use rust_embed::Embed;
use serde_json::{Map, Value};

use shared::paths;

#[derive(Embed)]
#[folder = "./kit/"]
struct Kit;

#[derive(Embed)]
#[folder = "./runtimes/"]
struct RuntimeAssets;

pub fn scaffold(bones_dir: &Path) -> Result<()> {
    for file_path in Kit::iter() {
        let Some(asset) = Kit::get(&file_path) else {
            continue;
        };
        write_asset(bones_dir, file_path.as_ref(), asset.data.as_ref())?;
    }

    Ok(())
}

pub fn runtime_names() -> Vec<String> {
    RuntimeAssets::iter()
        .filter_map(|path| path.split('/').next().map(str::to_string))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

pub fn base_runtime_defaults() -> Result<Map<String, Value>> {
    runtime_defaults_from_bytes("kit/runtime.toml", Kit::get(paths::RUNTIME_TOML).map(|asset| asset.data))
}

pub fn runtime_defaults(runtime: &str) -> Result<Map<String, Value>> {
    let asset_path = format!("{runtime}/runtime.toml");
    runtime_defaults_from_bytes(&asset_path, RuntimeAssets::get(&asset_path).map(|asset| asset.data))
}

pub fn scaffold_runtime_deployment(runtime: &str, bones_dir: &Path) -> Result<()> {
    let deploy_dir = bones_dir.join(paths::DEPLOYMENT_DIR);
    if deploy_dir.exists() {
        fs::remove_dir_all(&deploy_dir)
            .with_context(|| format!("Failed to clear deployment dir: {}", deploy_dir.display()))?;
    }
    scaffold_runtime_assets(runtime, bones_dir, paths::KIT_DEPLOYMENT_DIR)
}

pub fn scaffold_runtime_secrets(runtime: &str, bones_dir: &Path) -> Result<()> {
    scaffold_runtime_assets(runtime, bones_dir, paths::KIT_SECRETS_DIR)
}

fn scaffold_runtime_assets(runtime: &str, bones_dir: &Path, asset_prefix: &str) -> Result<()> {
    let runtime_prefix = format!("{runtime}/");

    for file_path in RuntimeAssets::iter() {
        let Some(stripped) = file_path.strip_prefix(&runtime_prefix) else {
            continue;
        };

        if !stripped.starts_with(asset_prefix) {
            continue;
        }

        let Some(asset) = RuntimeAssets::get(&file_path) else {
            continue;
        };

        write_asset(bones_dir, stripped, asset.data.as_ref())?;
    }

    Ok(())
}

fn runtime_defaults_from_bytes(asset_path: &str, bytes: Option<impl AsRef<[u8]>>) -> Result<Map<String, Value>> {
    let Some(bytes) = bytes else {
        bail!("Missing embedded runtime defaults at {asset_path}");
    };

    let content =
        str::from_utf8(bytes.as_ref()).with_context(|| format!("Embedded asset {asset_path} is not valid UTF-8"))?;
    let toml_value: toml::Value = toml::from_str(content)
        .with_context(|| format!("Failed to parse embedded runtime defaults at {asset_path}"))?;
    let json_value = serde_json::to_value(toml_value)
        .with_context(|| format!("Failed to convert embedded runtime defaults at {asset_path} to JSON"))?;

    json_value
        .as_object()
        .cloned()
        .ok_or_else(|| anyhow!("Embedded runtime defaults at {asset_path} are not a TOML table"))
}

fn write_asset(bones_dir: &Path, relative_path: &str, bytes: &[u8]) -> Result<()> {
    let dest = bones_dir.join(relative_path);

    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    fs::write(&dest, bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    if relative_path.starts_with(paths::KIT_HOOKS_DIR) || relative_path.starts_with(paths::KIT_DEPLOYMENT_DIR) {
        fs::set_permissions(&dest, fs::Permissions::from_mode(0o755))
            .with_context(|| format!("Failed to set permissions on {}", dest.display()))?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use tempfile::TempDir;

    use super::{base_runtime_defaults, runtime_defaults, runtime_names, scaffold, scaffold_runtime_deployment};
    use serde_json::Value;

    #[test]
    fn runtime_names_come_from_embedded_assets() {
        let runtimes = runtime_names();

        assert!(runtimes.contains(&String::from("laravel")));
        assert!(runtimes.contains(&String::from("next")));
    }

    #[test]
    fn runtime_defaults_read_local_runtime_toml() -> Result<()> {
        let defaults = runtime_defaults("laravel")?;

        assert_eq!(defaults.get("template"), Some(&Value::String(String::from("laravel"))));
        assert_eq!(defaults.get("php_version"), Some(&Value::String(String::from("8.5"))));
        Ok(())
    }

    #[test]
    fn vue_runtime_uses_vite_dist_web_root() -> Result<()> {
        let defaults = runtime_defaults("vue")?;

        assert_eq!(defaults.get("web_root"), Some(&Value::String(String::from("dist"))));
        Ok(())
    }

    #[test]
    fn base_runtime_defaults_read_embedded_kit_runtime() -> Result<()> {
        let defaults = base_runtime_defaults()?;

        assert!(defaults.contains_key("permissions"));
        Ok(())
    }

    #[test]
    fn scaffold_runtime_deployment_writes_runtime_scripts() -> Result<()> {
        let temp = TempDir::new()?;

        scaffold_runtime_deployment("laravel", temp.path())?;

        let deploy_dir = temp.path().join("deployment");
        assert!(deploy_dir.is_dir());
        let build_script = deploy_dir.join("build/02_install_php_deps.sh");
        assert!(
            fs::read_to_string(&build_script)?.contains("composer install"),
            "{} does not contain 'composer install'",
            build_script.display()
        );

        Ok(())
    }

    #[test]
    fn scaffold_runtime_deployment_replaces_kit_scripts() -> Result<()> {
        let temp = TempDir::new()?;

        scaffold(temp.path())?;
        let deploy_dir = temp.path().join("deployment");
        assert!(deploy_dir.join("build/02_run_build.sh").exists(), "kit script should exist after scaffold");

        scaffold_runtime_deployment("laravel", temp.path())?;

        let build_dir = deploy_dir.join("build");
        let entries: Vec<String> = fs::read_dir(&build_dir)?
            .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
            .collect();
        assert!(
            !entries.iter().any(|n| n == "02_run_build.sh"),
            "kit script '02_run_build.sh' should be removed after runtime scaffold, got: {entries:?}"
        );
        assert!(
            entries.iter().any(|n| n == "02_install_php_deps.sh"),
            "laravel script should exist after runtime scaffold, got: {entries:?}"
        );

        Ok(())
    }
}

```

`crates/bonesdeploy/src/infra/git.rs`:

```rs
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

#[derive(Debug, Clone)]
pub struct RemoteConnectionDetails {
    pub user: String,
    pub host: String,
    pub port: String,
    pub repo_path: String,
}

#[derive(Debug, Clone)]
pub struct RemoteInfo {
    pub name: String,
    pub url: String,
}

pub fn ensure_git_repository() -> Result<()> {
    let output =
        Command::new("git").args(["rev-parse", "--is-inside-work-tree"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    Ok(())
}

pub fn remote_exists(remote_name: &str) -> Result<bool> {
    let output = Command::new("git").args(["remote", "get-url", remote_name]).output().context("Failed to run git")?;
    Ok(output.status.success())
}

pub fn add_remote(remote_name: &str, remote_url: &str) -> Result<()> {
    let status = Command::new("git")
        .args(["remote", "add", remote_name, remote_url])
        .status()
        .with_context(|| format!("Failed to add git remote '{remote_name}'"))?;

    if !status.success() {
        bail!("Failed to add git remote '{remote_name}'");
    }

    Ok(())
}

pub fn list_remotes() -> Result<Vec<String>> {
    let output = Command::new("git").args(["remote"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Failed to list git remotes");
    }

    let remotes = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();

    Ok(remotes)
}

pub fn list_remotes_with_urls() -> Result<Vec<RemoteInfo>> {
    let output = Command::new("git").args(["remote", "-v"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Failed to list git remotes");
    }

    let mut remotes = Vec::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let mut parts = line.split_whitespace();
        let Some(name) = parts.next() else {
            continue;
        };
        let Some(url) = parts.next() else {
            continue;
        };
        let Some(kind) = parts.next() else {
            continue;
        };
        if kind != "(fetch)" {
            continue;
        }
        remotes.push(RemoteInfo { name: name.to_string(), url: url.to_string() });
    }

    Ok(remotes)
}

pub fn remote_url(remote_name: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["remote", "get-url", remote_name])
        .output()
        .with_context(|| format!("Failed to read URL for remote '{remote_name}'"))?;

    if !output.status.success() {
        bail!("Failed to read URL for remote '{remote_name}'");
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if url.is_empty() {
        bail!("Git remote '{remote_name}' has an empty URL");
    }

    Ok(url)
}

pub fn infer_remote_connection_details(remote_name: &str) -> Result<Option<RemoteConnectionDetails>> {
    let url = remote_url(remote_name)?;
    Ok(parse_remote_url(&url))
}

fn parse_remote_url(url: &str) -> Option<RemoteConnectionDetails> {
    parse_ssh_style_url(url.trim()).or_else(|| parse_scp_style_url(url.trim()))
}

fn parse_ssh_style_url(url: &str) -> Option<RemoteConnectionDetails> {
    if !url.starts_with("ssh://") {
        return None;
    }

    let without_scheme = &url[6..];
    let slash_index = without_scheme.find('/')?;
    let authority = without_scheme[..slash_index].trim();
    let path = without_scheme[slash_index..].trim();

    let host_port = authority.rsplit_once('@').map_or(authority, |(_, host)| host);
    let (host, port) = match host_port.split_once(':') {
        Some((host, port)) if !host.trim().is_empty() && !port.trim().is_empty() => {
            (host.trim().to_string(), port.trim().to_string())
        }
        _ => (host_port.trim().to_string(), String::from("22")),
    };

    if host.is_empty() || !has_git_extension(path) {
        return None;
    }

    Some(RemoteConnectionDetails { user: parse_user(authority), host, port, repo_path: path.to_string() })
}

fn parse_scp_style_url(url: &str) -> Option<RemoteConnectionDetails> {
    if url.contains("://") {
        return None;
    }

    let (left, right) = url.split_once(':')?;
    let host = left.rsplit_once('@').map_or(left, |(_, host)| host).trim();
    let repo_path = right.trim();
    if !repo_path.starts_with('/') {
        return None;
    }
    let repo_path = repo_path.to_string();

    if host.is_empty() || !has_git_extension(&repo_path) {
        return None;
    }

    Some(RemoteConnectionDetails {
        user: parse_user(left),
        host: host.to_string(),
        port: String::from("22"),
        repo_path,
    })
}

fn parse_user(authority: &str) -> String {
    authority.rsplit_once('@').map_or_else(|| String::from("git"), |(user, _)| user.to_string())
}

fn has_git_extension(path: &str) -> bool {
    Path::new(path).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("git"))
}

#[cfg(test)]
mod tests {
    use shared::paths;

    use super::{parse_remote_url, parse_scp_style_url, parse_ssh_style_url};

    fn repo_path(name: &str) -> String {
        paths::default_repo_path_for(name)
    }

    #[test]
    fn parse_ssh_style_url_parses_host_port_and_repo_path() {
        let details = parse_ssh_style_url(&format!("ssh://git@example.com:2222{}", repo_path("myapp")));
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.user, "git");
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "2222");
            assert_eq!(details.repo_path, repo_path("myapp"));
        }
    }

    #[test]
    fn parse_ssh_style_url_defaults_port_to_22() {
        let details = parse_ssh_style_url(&format!("ssh://git@example.com{}", repo_path("myapp")));
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.user, "git");
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, repo_path("myapp"));
        }
    }

    #[test]
    fn parse_scp_style_url_parses_absolute_repo_path() {
        let details = parse_scp_style_url(&format!("git@example.com:{}", repo_path("myapp")));
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.user, "git");
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, repo_path("myapp"));
        }
    }

    #[test]
    fn parse_scp_style_url_trims_surrounding_whitespace() {
        let details = parse_scp_style_url("git@example.com : /home/git/myapp.git");
        assert!(details.is_some());
        if let Some(details) = details {
            assert_eq!(details.user, "git");
            assert_eq!(details.host, "example.com");
            assert_eq!(details.port, "22");
            assert_eq!(details.repo_path, "/home/git/myapp.git");
        }
    }

    #[test]
    fn parse_remote_url_rejects_non_git_paths() {
        let non_git_path = repo_path("myapp").trim_end_matches(".git").to_string();
        assert!(parse_remote_url(&format!("ssh://git@example.com:22{non_git_path}")).is_none());
        assert!(parse_remote_url(&format!("git@example.com:{non_git_path}")).is_none());
    }

    #[test]
    fn parse_remote_url_rejects_relative_scp_paths() {
        assert!(parse_remote_url("git@example.com:myapp.git").is_none());
    }

    #[test]
    fn parse_remote_url_rejects_non_ssh_urls() {
        assert!(parse_remote_url("https://example.com/org/repo.git").is_none());
    }
}

```

`crates/bonesdeploy/src/infra/mod.rs`:

```rs
pub mod bonesinfra;
pub mod bonesinfra_cli;
pub mod bootstrap_ssh;
pub mod embedded;
pub mod git;
pub mod ssh;

```

`crates/bonesdeploy/src/infra/rsync.rs`:

```rs
use std::process::{Command, ExitStatus, Output};

use anyhow::{Context, Result};

pub fn output(args: &[&str]) -> Result<Output> {
    Command::new("rsync").args(args).output().context("Failed to run rsync — is it installed?")
}

pub fn status(args: &[&str]) -> Result<ExitStatus> {
    Ok(output(args)?.status)
}

```

`crates/bonesdeploy/src/infra/ssh.rs`:

```rs
use std::process::Command;

use anyhow::{Context, Result, bail};
use openssh::{KnownHosts, Session, SessionBuilder, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::config::Bones;
use shared::config::{default_deploy_user, parse_port};

pub async fn connect(config: &Bones) -> Result<Session> {
    let host = &config.host;
    let port = parse_port(&config.port)?;
    let user = default_deploy_user();

    connect_as(&user, host, port).await
}

pub async fn connect_privileged(config: &Bones) -> Result<Session> {
    let host = &config.host;
    let port = parse_port(&config.port)?;

    connect_as(&config.ssh_user, host, port).await
}

pub async fn connect_as(user: &str, host: &str, port: u16) -> Result<Session> {
    SessionBuilder::default()
        .known_hosts_check(KnownHosts::Accept)
        .user(user.into())
        .port(port)
        .connect(host)
        .await
        .with_context(|| format!("Failed to connect to {user}@{host}:{port}"))
}

pub fn external_command(user: &str, host: &str, port: &str) -> Command {
    let mut command = Command::new("ssh");
    command
        .args(["-p", port, "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null"])
        .arg(format!("{user}@{host}"));
    command
}

pub async fn run_cmd(session: &Session, cmd: &str) -> Result<String> {
    let output = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Remote command failed: {cmd}\n{stderr}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn stream_cmd(session: &Session, cmd: &str) -> Result<()> {
    let mut child = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    let stdout = child.stdout().take().ok_or_else(|| anyhow::anyhow!("stdout was not piped"))?;
    let stderr = child.stderr().take().ok_or_else(|| anyhow::anyhow!("stderr was not piped"))?;

    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{line}");
        }
    });

    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("{line}");
        }
    });

    // Drain both streams concurrently before checking exit status
    let _ = tokio::join!(stdout_task, stderr_task);

    let status = child.wait().await.context("Failed to wait for remote command")?;

    if !status.success() {
        bail!("Remote command failed: {cmd}");
    }

    Ok(())
}

pub async fn run_cmd_with_stdin(session: &Session, cmd: &str, stdin_bytes: &[u8]) -> Result<()> {
    let mut child = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    let mut stdin = child.stdin().take().ok_or_else(|| anyhow::anyhow!("stdin was not piped"))?;
    stdin.write_all(stdin_bytes).await.context("Failed to write stdin to remote command")?;
    stdin.shutdown().await.context("Failed to close stdin for remote command")?;
    drop(stdin);

    let output = child.wait_with_output().await.context("Failed to wait for remote command")?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Remote command failed: {cmd}\n{stderr}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::external_command;

    #[test]
    fn external_command_uses_expected_ssh_options() {
        let command = external_command("root", "deploy.example.com", "2222");
        let args = command.get_args();
        let args = args.map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();

        assert_eq!(
            args,
            vec![
                String::from("-p"),
                String::from("2222"),
                String::from("-o"),
                String::from("StrictHostKeyChecking=no"),
                String::from("-o"),
                String::from("UserKnownHostsFile=/dev/null"),
                String::from("root@deploy.example.com"),
            ]
        );
    }
}

```

`crates/bonesdeploy/src/main.rs`:

```rs
mod cli;
mod commands;
mod config;

mod infra;

mod ui;

use anyhow::Result;
use clap::Parser;
use commands::Cli;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(&cli).await
}

```

`crates/bonesdeploy/src/ui/mod.rs`:

```rs
pub mod prompts;

```

`crates/bonesdeploy/src/ui/prompts.rs`:

```rs
use anyhow::{Result, anyhow, bail};
use console::style;
use inquire::{Confirm, Select, Text};

use crate::config::Bones;
use crate::infra::git;

fn config_default<'a>(
    existing_config: Option<&'a Bones>,
    accessor: impl Fn(&'a Bones) -> &'a str,
    fallback: &'a str,
) -> &'a str {
    existing_config
        .and_then(|cfg| {
            let value = accessor(cfg);
            (!value.is_empty()).then_some(value)
        })
        .unwrap_or(fallback)
}

pub fn prompt_runtime_questions(
    questions: &serde_json::Value,
    defaults: &serde_json::Value,
) -> Result<serde_json::Value> {
    let mut answers = defaults.clone();
    let questions = questions.as_array().cloned().unwrap_or_default();

    for question in questions {
        let key = question["key"].as_str().unwrap_or("");
        if key.is_empty() {
            continue;
        }
        let label = question["label"].as_str().unwrap_or(key);
        let question_type = question["type"].as_str().unwrap_or("text");
        let default = answers.get(key).or(question.get("default")).cloned();

        let answer: serde_json::Value = match question_type {
            "bool" => {
                let default_bool = default.as_ref().and_then(serde_json::Value::as_bool).unwrap_or(false);
                let choice = Confirm::new(label).with_default(default_bool).prompt().map_err(|err| anyhow!(err))?;
                serde_json::Value::Bool(choice)
            }
            "choice" => {
                let choices: Vec<String> = question["choices"]
                    .as_array()
                    .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                if choices.is_empty() {
                    default.unwrap_or(serde_json::Value::Null)
                } else {
                    let default_idx = default
                        .as_ref()
                        .and_then(|v| v.as_str())
                        .and_then(|d| choices.iter().position(|c| c == d))
                        .unwrap_or(0);
                    let choice = Select::new(label, choices.clone())
                        .with_starting_cursor(default_idx)
                        .prompt()
                        .map_err(|err| anyhow!(err))?;
                    serde_json::Value::String(choice)
                }
            }
            _ => {
                let default_str = default.as_ref().and_then(|v| v.as_str()).unwrap_or("");
                let input = Text::new(label).with_default(default_str).prompt().map_err(|err| anyhow!(err))?;
                serde_json::Value::String(input)
            }
        };
        answers[key] = answer;
    }

    Ok(answers)
}

pub fn choose_template(available_templates: &[String]) -> Result<Option<String>> {
    if available_templates.is_empty() {
        return Ok(None);
    }

    let choice =
        Select::new("Runtime template:", vec![String::from("Use a template"), String::from("Build from scratch")])
            .with_help_message("Choose the app runtime to configure")
            .prompt()?;

    if choice == "Build from scratch" {
        return Ok(None);
    }

    let template_name = Select::new("Template:", available_templates.to_vec()).prompt()?;

    Ok(Some(template_name))
}

pub fn prompt_project_name(project_name_hint: &str, existing_config: Option<&Bones>) -> Result<String> {
    let default_project_name = config_default(existing_config, |cfg| cfg.project_name.as_str(), project_name_hint);
    Text::new("Project name:")
        .with_default(default_project_name)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_branch(existing_config: Option<&Bones>) -> Result<String> {
    let default_branch = config_default(existing_config, |cfg| cfg.branch.as_str(), "main");
    Text::new("Branch:")
        .with_default(default_branch)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_remote_name(existing_config: Option<&Bones>) -> Result<String> {
    const CREATE_REMOTE_OPTION: &str = "Create new deployment remote";

    let remotes = git::list_remotes_with_urls()?;
    if remotes.is_empty() {
        return prompt_remote_name_text(existing_config);
    }

    let default_remote = existing_config.and_then(|cfg| {
        let value = cfg.remote_name.as_str();
        (!value.is_empty()).then(|| cfg.remote_name.clone())
    });

    let preferred = default_remote.or_else(|| {
        let has_production = remotes.iter().any(|r| r.name == "production");
        if has_production { Some(String::from("production")) } else { None }
    });

    let mut ordered_remotes = Vec::with_capacity(remotes.len());
    if let Some(ref pref) = preferred
        && let Some(pos) = remotes.iter().position(|r| r.name == *pref)
    {
        ordered_remotes.push(remotes[pos].clone());
        ordered_remotes.extend(remotes.iter().enumerate().filter(|(i, _)| *i != pos).map(|(_, r)| r.clone()));
    }
    if ordered_remotes.is_empty() {
        ordered_remotes = remotes;
    }

    let mut display_options: Vec<String> = ordered_remotes.iter().map(remote_display_label).collect();
    display_options.push(String::from(CREATE_REMOTE_OPTION));

    let choice = Select::new("Deployment remote:", display_options)
        .with_help_message("Choose the VPS remote, not your code host.")
        .raw_prompt()
        .map_err(|err| anyhow!(err))?;

    if choice.index == ordered_remotes.len() {
        return prompt_remote_name_text(existing_config);
    }

    let chosen = ordered_remotes[choice.index].name.clone();

    if chosen == "origin" {
        println!("{} origin usually points to your code host, not your VPS.", style("Warning:").yellow().bold());
        let proceed = Confirm::new("Use 'origin' anyway?")
            .with_default(false)
            .with_help_message("Choose No unless origin points to your VPS.")
            .prompt()
            .map_err(|err| anyhow!(err))?;
        if !proceed {
            bail!("Choose a deployment remote that points to your VPS.");
        }
    }

    Ok(chosen)
}

fn remote_display_label(remote: &git::RemoteInfo) -> String {
    if remote.name == "origin" {
        format!("{} ({}) — not a deployment remote", remote.name, remote.url)
    } else {
        format!("{} ({})", remote.name, remote.url)
    }
}

pub fn prompt_host(
    existing_config: Option<&Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.host.clone());
    }

    let default_host = config_default(existing_config, |cfg| cfg.host.as_str(), "");
    Text::new("Server host or IP:")
        .with_default(default_host)
        .with_help_message("e.g. deploy.example.com or 203.0.113.10")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_port(
    existing_config: Option<&Bones>,
    inferred_remote: Option<&git::RemoteConnectionDetails>,
) -> Result<String> {
    if let Some(details) = inferred_remote {
        return Ok(details.port.clone());
    }

    let default_port = config_default(existing_config, |cfg| cfg.port.as_str(), "22");
    Text::new("SSH port:")
        .with_default(default_port)
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn confirm_remote_setup() -> Result<bool> {
    println!();
    for line in remote_setup_prompt_lines() {
        println!("{line}");
    }
    println!();
    Confirm::new("Bootstrap remote server?").with_default(false).prompt().map_err(|err| anyhow!(err))
}

pub fn confirm_remote_runtime() -> Result<bool> {
    println!();
    for line in remote_runtime_prompt_lines() {
        println!("{line}");
    }
    println!();
    Confirm::new("Apply runtime setup?").with_default(false).prompt().map_err(|err| anyhow!(err))
}

#[cfg(test)]
fn is_affirmative(answer: &str) -> bool {
    matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes")
}

fn remote_setup_prompt_lines() -> [&'static str; 1] {
    ["Remote bootstrap prepares the VPS for this project."]
}

fn remote_runtime_prompt_lines() -> [&'static str; 1] {
    ["Runtime setup installs app services for this project."]
}

pub fn confirm_remote_ssl() -> Result<bool> {
    println!();
    for line in remote_ssl_prompt_lines() {
        println!("{line}");
    }
    println!();
    Confirm::new("Configure HTTPS?").with_default(false).prompt().map_err(|err| anyhow!(err))
}

fn remote_ssl_prompt_lines() -> [&'static str; 1] {
    ["HTTPS requires DNS to point at this server."]
}

fn prompt_remote_name_text(existing_config: Option<&Bones>) -> Result<String> {
    let default_remote =
        existing_config.map(|cfg| cfg.remote_name.as_str()).filter(|value| !value.is_empty()).unwrap_or("production");
    Text::new("Deployment remote name:")
        .with_default(default_remote)
        .with_help_message("Created if missing.")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_ssl_domain(existing_config: Option<&Bones>) -> Result<String> {
    let default_domain = config_default(existing_config, |cfg| cfg.domain.as_str(), "");
    Text::new("Domain:")
        .with_default(default_domain)
        .with_help_message("e.g. app.example.com")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

pub fn prompt_ssl_email(existing_config: Option<&Bones>) -> Result<String> {
    let default_email = config_default(existing_config, |cfg| cfg.email.as_str(), "");
    Text::new("Let's Encrypt email:")
        .with_default(default_email)
        .with_help_message("e.g. ops@example.com")
        .prompt()
        .map(|value| value.trim().to_string())
        .map_err(|err| anyhow!(err))
}

#[cfg(test)]
mod tests {
    use super::{is_affirmative, remote_runtime_prompt_lines, remote_setup_prompt_lines};

    /// Accepts common yes values like y, yes, and YES.
    #[test]
    fn confirmation_parser_accepts_common_yes_values() {
        assert!(is_affirmative("y"));
        assert!(is_affirmative(" yes "));
        assert!(is_affirmative("YES"));
    }

    /// Rejects non-affirmative values like empty string, n, and no.
    #[test]
    fn confirmation_parser_rejects_non_affirmative_values() {
        assert!(!is_affirmative(""));
        assert!(!is_affirmative("n"));
        assert!(!is_affirmative("no"));
    }

    /// Mentions VPS in the remote bootstrap prompt.
    #[test]
    fn remote_setup_prompt_lines_mention_vps() {
        let joined = remote_setup_prompt_lines().join("\n");

        assert!(joined.contains("VPS"), "remote bootstrap prompt should mention VPS\n{joined}");
    }

    /// Mentions app services in the remote runtime prompt.
    #[test]
    fn remote_runtime_prompt_lines_mention_app_services() {
        let joined = remote_runtime_prompt_lines().join("\n");

        assert!(joined.contains("app services"), "remote runtime prompt should mention app services\n{joined}");
    }
}

```

`crates/bonesremote/Cargo.toml`:

```toml
[package]
name = "bonesremote"
version = "0.5.1"
edition = "2024"


[dependencies]
anyhow = "1.0.102"
clap = { version = "4.6.1", features = ["derive"] }
console = "0.16.3"
nix = { version = "0.31.2", features = ["user"] }
serde = { version = "1.0.228", features = ["derive"] }
serde_json = "1.0"
toml = "0.8"
time = { version = "0.3.47", features = ["formatting", "macros"] }
walkdir = "2.5.0"
shared = { path = "../shared" }

[lints.clippy]
# broad groups (lower priority so individual lint overrides take effect)
correctness = { level = "deny", priority = -1 }
suspicious = { level = "deny", priority = -1 }
complexity = { level = "warn", priority = -1 }
style = { level = "allow", priority = -1 }
perf = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }

# readability / naming
similar_names = "warn"
many_single_char_names = "warn"
module_name_repetitions = "warn"
enum_variant_names = "warn"
struct_field_names = "warn"
disallowed_names = "deny"
wildcard_imports = "warn"
module_inception = "warn"

# complexity / API readability
cognitive_complexity = "warn"
too_many_arguments = "deny"
fn_params_excessive_bools = "warn"
large_types_passed_by_value = "warn"
trivially_copy_pass_by_ref = "warn"

# panic / debug leftovers
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
# This is a CLI binary: stdout/stderr output is the intended UX.
print_stdout = "allow"
print_stderr = "allow"

# docs / contracts
missing_panics_doc = "deny"
missing_errors_doc = "deny"
missing_safety_doc = "deny"
undocumented_unsafe_blocks = "deny"

# numeric safety
cast_possible_truncation = "warn"
cast_sign_loss = "warn"
cast_possible_wrap = "warn"
checked_conversions = "warn"

# anti-mess / consistency
absolute_paths = "warn"
allow_attributes = "deny"
redundant_clone = "warn"
implicit_clone = "warn"
semicolon_if_nothing_returned = "warn"
match_same_arms = "warn"
needless_pass_by_value = "warn"
cloned_instead_of_copied = "warn"
flat_map_option = "warn"
from_iter_instead_of_collect = "warn"
inefficient_to_string = "warn"
manual_let_else = "warn"
manual_ok_or = "warn"
map_unwrap_or = "warn"
unnecessary_wraps = "warn"

```

`crates/bonesremote/src/cli/args.rs`:

```rs
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bonesremote", about = "Remote release deployment tool")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub enum Command {
    /// Install sudoers drop-in for passwordless bonesremote
    Init,
    /// Check server environment health
    Doctor,
    /// Run the full remote deployment lifecycle
    Deploy {
        /// Site identifier (must match an imported site directory)
        #[arg(long)]
        site: String,
        /// Exact revision to deploy (defaults to the configured branch)
        #[arg(long)]
        revision: Option<String>,
    },
    /// Print remote deployment status as JSON
    Status {
        #[arg(long)]
        site: String,
    },
    /// Thin git-hook entrypoints
    Hook {
        #[command(subcommand)]
        command: HookCommand,
    },
    /// Import or export root-owned remote site state
    Site {
        #[command(subcommand)]
        command: SiteCommand,
    },
    /// Release lifecycle operations
    Release {
        #[command(subcommand)]
        command: ReleaseCommand,
    },
    /// Narrow privileged service operations (requires root)
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
    /// Print the version
    Version,
}

#[derive(Subcommand)]
pub enum HookCommand {
    /// Resolve a post-receive push and delegate deployment
    PostReceive {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum SiteCommand {
    /// Import a deployment dataset from stdin
    Import {
        #[arg(long)]
        site: String,
    },
    /// Export the current deployment dataset to stdout
    Export {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum ReleaseCommand {
    /// Repoint current to the previous release
    Rollback {
        #[arg(long)]
        site: String,
    },
    /// Drop the staged release and clean state
    DropFailed {
        #[arg(long)]
        site: String,
    },
    /// Prune old releases according to the registry keep count
    Prune {
        #[arg(long)]
        site: String,
    },
}

#[derive(Subcommand)]
pub enum ServiceCommand {
    /// Restart the per-site nginx service
    Restart {
        #[arg(long)]
        site: String,
    },
}

```

`crates/bonesremote/src/cli/dispatch.rs`:

```rs
use anyhow::Result;

use crate::cli::args::{Cli, Command, HookCommand, ReleaseCommand, ServiceCommand, SiteCommand};
use crate::commands::{deploy, doctor, drop_failed_release, hook, init, post_deploy, service, site, status, version};

pub fn run(cli: &Cli) -> Result<()> {
    match &cli.command {
        Command::Init => init::run(),
        Command::Doctor => doctor::run(),
        Command::Deploy { site, revision } => deploy::run_full(site, revision.as_deref()),
        Command::Status { site } => status::run(site),
        Command::Hook { command } => match command {
            HookCommand::PostReceive { site: site_name } => hook::post_receive(site_name),
        },
        Command::Site { command } => match command {
            SiteCommand::Import { site: site_name } => site::import(site_name),
            SiteCommand::Export { site: site_name } => site::export(site_name),
        },
        Command::Release { command } => match command {
            ReleaseCommand::Rollback { site: site_name } => deploy::rollback(site_name),
            ReleaseCommand::DropFailed { site: site_name } => drop_failed_release::run(site_name),
            ReleaseCommand::Prune { site: site_name } => post_deploy::run(site_name),
        },
        Command::Service { command } => match command {
            ServiceCommand::Restart { site: site_name } => service::run(site_name),
        },
        Command::Version => {
            version::run();
            Ok(())
        }
    }
}

```

`crates/bonesremote/src/cli/mod.rs`:

```rs
pub mod args;
pub mod dispatch;

```

`crates/bonesremote/src/commands/activate_release.rs`:

```rs
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use shared::paths;
use shared::registry;

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release activate")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg =
        registry::load(&registry_path).map_err(|error| anyhow::anyhow!("Failed to load remote site state: {error}"))?;

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    let current_link = PathBuf::from(&cfg.current_path);

    if !release_dir.exists() {
        anyhow::bail!("Promoted release directory does not exist: {}", release_dir.display());
    }

    if current_link.exists() && !current_link.is_symlink() {
        bail!("current exists and is not a symlink: {}", current_link.display());
    }

    release_state::point_symlink_atomically(&current_link, Path::new(&release_dir))?;
    release_state::clear_staged_release(site)?;

    println!("Activated release: {release_name}");
    Ok(())
}

```

`crates/bonesremote/src/commands/deploy.rs`:

```rs
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::paths;
use shared::registry;

use crate::commands::{
    activate_release, drop_failed_release, post_deploy, release_build, release_checkout, release_prepare, service,
    stage_release, wire_shared,
};
use crate::privileges;
use crate::release_state;

pub fn run_full(site: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_root("bonesremote deploy")?;
    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let target_revision = revision.map_or_else(|| cfg.branch.clone(), ToOwned::to_owned);

    stage_release::run(site)?;

    let context_dir = release_checkout::ensure_build_context(site)?;

    if let Err(error) = release_checkout::run(site, &target_revision, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = release_build::run(site, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = release_build::promote(site, &context_dir) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = wire_shared::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = release_prepare::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = activate_release::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = service::run(site) {
        cleanup(site, Some(&context_dir));
        drop_failed_release::run(site).ok();
        return Err(error);
    }

    if let Err(error) = post_deploy::run(site) {
        cleanup(site, Some(&context_dir));
        return Err(error);
    }

    cleanup(site, Some(&context_dir));
    Ok(())
}

fn cleanup(site: &str, context: Option<&Path>) {
    if let Some(context) = context {
        release_checkout::cleanup_build_context(site, context).ok();
    }
}

pub fn rollback(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release rollback")?;
    let cfg = registry::load(&paths::bonesremote_registry_path(site)).context(registry_load_error())?;

    let releases = release_state::list_releases_sorted(&cfg)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = release_state::current_release_name(&cfg)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;

    if current_idx == 0 {
        bail!("Current release is already the oldest release; cannot roll back");
    }

    let previous_name = releases[current_idx - 1].clone();
    let previous_dir = release_state::release_dir(&cfg, &previous_name);
    let current_link = PathBuf::from(&cfg.current_path);
    release_state::point_symlink_atomically(&current_link, &previous_dir)?;
    service::run(site)?;

    println!("Rollback complete: {current_name} -> {previous_name}");
    Ok(())
}

pub fn registry_load_error() -> String {
    String::from("Failed to load remote site state from registry")
}

```

`crates/bonesremote/src/commands/doctor.rs`:

```rs
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use console::style;
use shared::paths;

pub fn run() -> Result<()> {
    println!("{}", style(format!("{} doctor", paths::BONESREMOTE_BINARY)).bold());

    let mut issues: Vec<String> = Vec::new();

    check_supported_distribution(&mut issues);
    check_globally_available(&mut issues);
    check_passwordless_sudo(&mut issues);
    check_apparmor_support(&mut issues);

    if issues.is_empty() {
        println!("\n{} All checks passed.", style("OK").green().bold());
        Ok(())
    } else {
        println!();
        for issue in &issues {
            println!("  {} {issue}", style("!").red().bold());
        }
        anyhow::bail!("Doctor found {} issue{}", issues.len(), if issues.len() == 1 { "" } else { "s" });
    }
}

fn check_supported_distribution(issues: &mut Vec<String>) {
    let os_release = fs::read_to_string(paths::ETC_OS_RELEASE);
    let Ok(os_release) = os_release else {
        issues.push(format!("Failed to read {}; expected Debian or Ubuntu host", paths::ETC_OS_RELEASE));
        return;
    };

    let normalized = os_release.to_lowercase();
    if normalized.contains("id=debian") || normalized.contains("id=ubuntu") {
        return;
    }

    issues.push("Unsupported host OS; bonesremote currently supports Debian/Ubuntu only".to_string());
}

fn check_globally_available(issues: &mut Vec<String>) {
    let result = Command::new(paths::BONESREMOTE_BINARY).arg("version").output();

    match result {
        Ok(output) if output.status.success() => {}
        _ => issues.push(format!("{} is not globally available (not in PATH)", paths::BONESREMOTE_BINARY)),
    }
}

fn check_passwordless_sudo(issues: &mut Vec<String>) {
    let privileged_commands = [
        [paths::BONESREMOTE_BINARY, "hook", "post-receive", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "service", "restart", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "release", "rollback", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "release", "drop-failed", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "release", "prune", "--site", "nonexistent"],
    ];

    for command in privileged_commands {
        let result = Command::new("sudo").arg("-n").arg("-l").args(command).output();

        match result {
            Ok(output) if output.status.success() => {}
            _ => issues.push(format!(
                "{} is not allowed via passwordless sudo: {} (run 'sudo {} init')",
                paths::BONESREMOTE_BINARY,
                command.join(" "),
                paths::BONESREMOTE_BINARY
            )),
        }
    }
}

fn check_apparmor_support(issues: &mut Vec<String>) {
    check_apparmor_kernel_enabled(issues);
    check_apparmor_service(issues);

    let Some(profile_names) = check_apparmor_profiles_installed(issues) else {
        return;
    };

    check_apparmor_unit_wiring(&profile_names, issues);
}

fn check_apparmor_kernel_enabled(issues: &mut Vec<String>) {
    let enabled_file = fs::read_to_string(paths::APPARMOR_ENABLED_PARAM);
    let Ok(enabled_value) = enabled_file else {
        issues.push(format!("AppArmor check failed: could not read {}", paths::APPARMOR_ENABLED_PARAM));
        return;
    };

    if !apparmor_kernel_enabled(&enabled_value) {
        issues.push("AppArmor check failed: kernel module is not enabled".to_string());
    }
}

fn check_apparmor_service(issues: &mut Vec<String>) {
    let status = Command::new("systemctl").args(["is-active", "apparmor"]).output();
    match status {
        Ok(output) if output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "active" => {}
        Ok(output) => {
            let service_status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            issues.push(format!(
                "AppArmor check failed: apparmor service is not active (status: {})",
                if service_status.is_empty() { "unknown" } else { service_status.as_str() }
            ));
        }
        Err(error) => {
            issues.push(format!("AppArmor check failed: could not run systemctl is-active apparmor ({error})"));
        }
    }
}

fn check_apparmor_profiles_installed(issues: &mut Vec<String>) -> Option<Vec<String>> {
    let profiles = fs::read_dir(paths::ETC_APPARMOR_D);
    let Ok(profiles) = profiles else {
        issues.push(format!("AppArmor check failed: could not read {}", paths::ETC_APPARMOR_D));
        return None;
    };

    let profile_files: Vec<String> = profiles
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| apparmor_profile_filename(name))
        .collect();

    if profile_files.is_empty() {
        issues.push(format!(
            "AppArmor check failed: no bonesdeploy AppArmor profile found under {}",
            paths::ETC_APPARMOR_D
        ));
        return None;
    }

    Some(profile_files)
}

fn check_apparmor_unit_wiring(profile_names: &[String], issues: &mut Vec<String>) {
    let units = fs::read_dir(paths::ETC_SYSTEMD_SYSTEM);
    let Ok(units) = units else {
        issues.push(format!("AppArmor check failed: could not read {}", paths::ETC_SYSTEMD_SYSTEM));
        return;
    };

    let installed_profiles: HashSet<&str> = profile_names.iter().map(String::as_str).collect();
    let expected_unit_names: Vec<String> =
        profile_names.iter().filter_map(|profile_name| apparmor_unit_name_for_profile(profile_name)).collect();

    if expected_unit_names.is_empty() {
        issues.push("AppArmor check failed: no bonesdeploy AppArmor profile matched an expected unit name".to_string());
        return;
    }

    let unit_entries: HashMap<String, PathBuf> = units
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok().map(|name| (name, entry.path())))
        .collect();

    for expected_unit_name in expected_unit_names {
        let Some(path) = unit_entries.get(&expected_unit_name) else {
            issues.push(format!(
                "AppArmor check failed: expected {}/{} for installed bonesdeploy profile",
                paths::ETC_SYSTEMD_SYSTEM,
                expected_unit_name
            ));
            continue;
        };

        let contents = fs::read_to_string(path);

        match contents {
            Ok(contents) => match apparmor_unit_wiring_status(&contents, &installed_profiles) {
                AppArmorUnitWiringStatus::Ok => {}
                AppArmorUnitWiringStatus::MissingOrdering => {
                    issues.push(format!(
                        "AppArmor check failed: {} is missing apparmor.service ordering or dependency",
                        path.display()
                    ));
                }
                AppArmorUnitWiringStatus::MissingProfile => {
                    issues.push(format!("AppArmor check failed: {} is missing AppArmorProfile=", path.display()));
                }
                AppArmorUnitWiringStatus::UnknownProfile(profile_name) => {
                    issues.push(format!(
                        "AppArmor check failed: {} references unknown AppArmor profile {}",
                        path.display(),
                        profile_name
                    ));
                }
            },
            Err(error) => {
                issues.push(format!("AppArmor check failed: could not read {} ({error})", path.display()));
            }
        }
    }
}

fn apparmor_kernel_enabled(contents: &str) -> bool {
    matches!(contents.trim().to_ascii_lowercase().as_str(), "y" | "yes" | "1")
}

fn apparmor_profile_filename(name: &str) -> bool {
    name.starts_with("bonesdeploy-") && name.ends_with("-nginx")
}

fn apparmor_unit_name_for_profile(profile_name: &str) -> Option<String> {
    profile_name
        .strip_prefix("bonesdeploy-")
        .and_then(|name| name.strip_suffix("-nginx"))
        .map(|project_name| paths::nginx_service_name(&project_name))
}

fn systemd_directive_values<'a>(contents: &'a str, directive: &str) -> Vec<&'a str> {
    contents
        .lines()
        .filter_map(|line| line.strip_prefix(directive))
        .filter_map(|value| value.strip_prefix('='))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

fn systemd_directive_contains_token(contents: &str, directive: &str, token: &str) -> bool {
    systemd_directive_values(contents, directive)
        .into_iter()
        .flat_map(|value| value.split_ascii_whitespace())
        .any(|value| value == token)
}

fn apparmor_profile_binding(contents: &str) -> Option<&str> {
    systemd_directive_values(contents, "AppArmorProfile").into_iter().next()
}

enum AppArmorUnitWiringStatus {
    Ok,
    MissingOrdering,
    MissingProfile,
    UnknownProfile(String),
}

fn apparmor_unit_wiring_status(contents: &str, installed_profiles: &HashSet<&str>) -> AppArmorUnitWiringStatus {
    let has_apparmor_after = systemd_directive_contains_token(contents, "After", "apparmor.service");
    let has_apparmor_requires = systemd_directive_contains_token(contents, "Requires", "apparmor.service");

    if !has_apparmor_after || !has_apparmor_requires {
        return AppArmorUnitWiringStatus::MissingOrdering;
    }

    let Some(profile_name) = apparmor_profile_binding(contents) else {
        return AppArmorUnitWiringStatus::MissingProfile;
    };

    if installed_profiles.contains(profile_name) {
        AppArmorUnitWiringStatus::Ok
    } else {
        AppArmorUnitWiringStatus::UnknownProfile(profile_name.to_string())
    }
}

#[cfg(test)]
#[path = "tests/test_doctor.rs"]
mod test_doctor;

```

`crates/bonesremote/src/commands/drop_failed_release.rs`:

```rs
use std::fs;

use anyhow::{Context, Result};
use shared::paths;
use shared::registry;

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;

    let staged_path = release_state::staged_release_path(site);
    if !staged_path.exists() {
        println!("No staged release state found. Nothing to clean.");
        return Ok(());
    }

    let Ok(release_name) = release_state::read_staged_release(site) else {
        release_state::clear_staged_release(site).ok();
        println!("Cleared staged release state.");
        return Ok(());
    };

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let release_dir = release_state::release_dir(&cfg, &release_name);
    if release_dir.exists() {
        fs::remove_dir_all(&release_dir)
            .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
        println!("Removed failed release: {release_name}");
    }

    release_state::clear_staged_release(site)?;
    println!("Cleared staged release state.");
    Ok(())
}

```

`crates/bonesremote/src/commands/hook.rs`:

```rs
use anyhow::Result;

pub fn post_receive(site: &str) -> Result<()> {
    super::deploy::run_full(site, None)
}

```

`crates/bonesremote/src/commands/init.rs`:

```rs
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;

use crate::privileges;
use shared::paths;

pub fn run() -> Result<()> {
    privileges::ensure_root("bonesremote init")?;

    println!("{}", style(format!("{} init", paths::BONESREMOTE_BINARY)).bold());

    let sudoers_path = paths::SUDOERS_PATH;
    let bonesdeploy_path = which_bonesdeploy_remote()?;

    // Only the commands that need ownership or live-state changes run via sudo.
    let sudoers_content = format!(
        "# Installed by bonesremote init\n\
             {} ALL=(root) NOPASSWD: {} hook post-receive --site *, {} service restart --site *, {} release rollback --site *, {} release drop-failed --site *, {} release prune --site *\n",
        paths::DEPLOY_USER,
        bonesdeploy_path.display(),
        bonesdeploy_path.display(),
        bonesdeploy_path.display(),
        bonesdeploy_path.display(),
        bonesdeploy_path.display()
    );

    fs::write(sudoers_path, &sudoers_content).with_context(|| format!("Failed to write {sudoers_path}"))?;

    Command::new("chmod").args(["0440", sudoers_path]).status().context("Failed to chmod sudoers drop-in")?;

    let status = Command::new("visudo").args(["-c", "-f", sudoers_path]).status().context("Failed to run visudo")?;

    if !status.success() {
        fs::remove_file(sudoers_path).ok();
        bail!("visudo validation failed — sudoers drop-in removed for safety");
    }

    println!("{} Installed sudoers drop-in at {}", style("Done!").green().bold(), sudoers_path);

    Ok(())
}

fn which_bonesdeploy_remote() -> Result<PathBuf> {
    let output = Command::new("which")
        .arg(paths::BONESREMOTE_BINARY)
        .output()
        .context(format!("Failed to run 'which {}'", paths::BONESREMOTE_BINARY))?;

    if !output.status.success() {
        bail!(
            "{} is not in PATH. \
             Install it globally before running init.",
            paths::BONESREMOTE_BINARY
        );
    }

    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
    let path = fs::canonicalize(&path).with_context(|| format!("Failed to canonicalize {}", path.display()))?;
    if !approved_bonesdeploy_path(&path) {
        bail!("Refusing to write sudoers entry for {}: expected {} or /usr/bin", path.display(), paths::USR_LOCAL_BIN);
    }

    Ok(path)
}

fn approved_bonesdeploy_path(path: &Path) -> bool {
    path.starts_with(paths::USR_LOCAL_BIN) || path.starts_with("/usr/bin")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::approved_bonesdeploy_path;

    #[test]
    fn approved_bonesdeploy_path_accepts_standard_bin_dirs() {
        assert!(approved_bonesdeploy_path(Path::new("/usr/local/bin/bonesremote")));
        assert!(approved_bonesdeploy_path(Path::new("/usr/bin/bonesremote")));
    }

    #[test]
    fn approved_bonesdeploy_path_rejects_unexpected_dirs() {
        assert!(!approved_bonesdeploy_path(Path::new("/home/alex/.local/bin/bonesremote")));
    }
}

```

`crates/bonesremote/src/commands/mod.rs`:

```rs
pub(crate) mod activate_release;
pub(crate) mod deploy;
pub(crate) mod doctor;
pub(crate) mod drop_failed_release;
pub(crate) mod hook;
pub(crate) mod init;
pub(crate) mod post_deploy;
pub(crate) mod release_build;
pub(crate) mod release_checkout;
pub(crate) mod release_prepare;
pub(crate) mod service;
pub(crate) mod site;
pub(crate) mod stage_release;
pub(crate) mod status;
pub(crate) mod version;
pub(crate) mod wire_shared;

pub use crate::cli::args::Cli;
pub use crate::cli::dispatch::run;

```

`crates/bonesremote/src/commands/post_deploy.rs`:

```rs
use std::fs;

use anyhow::{Context, Result};
use shared::paths;
use shared::registry;

use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let pruned = prune_old_releases(&cfg)?;
    if !pruned.is_empty() {
        println!("Pruned releases: {}", pruned.join(", "));
    }

    Ok(())
}

fn prune_old_releases(cfg: &registry::Registry) -> Result<Vec<String>> {
    let active_release = release_state::current_release_name(cfg)?;
    let mut releases = release_state::list_releases_sorted(cfg)?;
    let keep = cfg.releases_keep.max(1);

    let mut pruned = Vec::new();
    while releases.len() > keep {
        let oldest = releases.remove(0);
        if oldest == active_release {
            releases.push(oldest);
            releases.sort();
            continue;
        }

        let path = release_state::release_dir(cfg, &oldest);
        if path.exists() {
            fs::remove_dir_all(&path).with_context(|| format!("Failed to prune old release {}", path.display()))?;
            pruned.push(oldest);
        }
    }

    Ok(pruned)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;
    use shared::registry::Registry;

    use super::prune_old_releases;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn config_for(temp_root: &Path, keep: usize) -> Registry {
        let site_root = temp_root.join("project_root");
        Registry {
            site: String::from("acme"),
            repo_path: temp_root.join("repo.git").to_string_lossy().to_string(),
            site_root: site_root.to_string_lossy().to_string(),
            shared_root: site_root.join(paths::SHARED_DIR).to_string_lossy().to_string(),
            releases_root: site_root.join(paths::RELEASES_DIR).to_string_lossy().to_string(),
            current_path: site_root.join(paths::CURRENT_LINK).to_string_lossy().to_string(),
            runtime_user: String::from("acme"),
            runtime_group: String::from("acme"),
            branch: String::from("main"),
            deploy_on_push: true,
            releases_keep: keep,
        }
    }

    fn make_release(root: &Path, name: &str) -> Result<()> {
        fs::create_dir_all(root.join("project_root").join(paths::RELEASES_DIR).join(name))?;
        Ok(())
    }

    fn set_current_release(root: &Path, name: &str) -> Result<()> {
        let project_root = root.join("project_root");
        let releases = project_root.join(paths::RELEASES_DIR);
        fs::create_dir_all(&releases)?;
        let target = releases.join(name);
        symlink(&target, project_root.join(paths::CURRENT_LINK))?;
        Ok(())
    }

    #[test]
    fn prune_old_releases_removes_oldest_inactive_releases_up_to_keep_limit() -> Result<()> {
        let root = temp_dir("bonesremote_post_deploy_prune")?;
        let cfg = config_for(&root, 2);

        make_release(&root, "20260101_000000")?;
        make_release(&root, "20260102_000000")?;
        make_release(&root, "20260103_000000")?;
        set_current_release(&root, "20260103_000000")?;

        let pruned = prune_old_releases(&cfg)?;

        assert_eq!(pruned, vec!["20260101_000000"]);
        assert!(!root.join("project_root").join(paths::RELEASES_DIR).join("20260101_000000").exists());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260102_000000").exists());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260103_000000").exists());

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn prune_old_releases_keeps_active_release_when_within_keep_limit() -> Result<()> {
        let root = temp_dir("bonesremote_post_deploy_prune_active")?;
        let cfg = config_for(&root, 2);

        make_release(&root, "20260101_000000")?;
        make_release(&root, "20260102_000000")?;
        set_current_release(&root, "20260101_000000")?;

        let pruned = prune_old_releases(&cfg)?;

        assert!(pruned.is_empty());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260101_000000").exists());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260102_000000").exists());

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}

```

`crates/bonesremote/src/commands/release_build.rs`:

```rs
use std::fs;
use std::os::unix::fs::chown;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Component, Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config::{Runtime, load_runtime};
use shared::paths;
use shared::paths::default_web_root;
use shared::registry;

use crate::privileges;
use crate::release::scripts as deploy_output;
use crate::release_state;

pub fn run(site: &str, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    if !context.is_dir() {
        bail!("Build context does not exist: {}", context.display());
    }

    let scripts_dir = paths::bonesremote_site_root(site).join(paths::DEPLOYMENT_DIR).join(paths::DEPLOYMENT_BUILD_DIR);
    if !scripts_dir.is_dir() {
        println!(
            "No deployment scripts at {}; running build steps directly on the exported source tree.",
            scripts_dir.display()
        );
        return Ok(());
    }

    let scripts = list_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No deployment scripts found at {}; skipping build.", scripts_dir.display());
        return Ok(());
    }

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running build script {script_name}...");

        let runtime = load_runtime(&paths::bonesremote_site_root(site)).unwrap_or_else(|_| Runtime {
            web_root: default_web_root(),
            build_image: String::new(),
            runtime_user: String::new(),
            runtime_group: String::new(),
            release_group: String::new(),
        });
        if runtime.build_image.is_empty() {
            bail!("Build scripts require build_image in runtime.toml");
        }

        let status = deploy_output::run_podman_build_script(
            &script,
            context,
            &context.join(format!("{script_name}.log")),
            &deploy_output::BuildScriptEnv {
                project_name: &cfg.site,
                web_root: &runtime.web_root,
                build_image: &runtime.build_image,
            },
        )
        .with_context(|| format!("Failed to execute build script {}", script.display()))?;

        if !status.success() {
            bail!("Build script {script_name} exited with status {status}");
        }
    }

    Ok(())
}

pub fn promote(site: &str, context: &Path) -> Result<PathBuf> {
    privileges::ensure_root("bonesremote release promote")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    harden_release_tree(context, &release_dir, &cfg)
        .with_context(|| format!("Failed to promote release {release_name}"))?;

    println!("Promoted release {release_name} into {}", release_dir.display());
    Ok(release_dir)
}

fn harden_release_tree(source: &Path, destination: &Path, cfg: &registry::Registry) -> Result<()> {
    if !source.is_dir() {
        bail!("Source tree is not a directory: {}", source.display());
    }

    fs::create_dir_all(destination)
        .with_context(|| format!("Failed to create release directory {}", destination.display()))?;
    clear_directory_children(destination)?;

    copy_hardened(source, destination, source)?;
    seal_release(destination, cfg)?;
    Ok(())
}

fn copy_hardened(source: &Path, destination: &Path, tree_root: &Path) -> Result<()> {
    for entry in fs::read_dir(source).with_context(|| format!("Failed to read source tree {}", source.display()))? {
        let entry = entry?;
        let source_path = entry.path();
        let dest_path = destination.join(entry.file_name());
        let metadata = fs::symlink_metadata(&source_path)
            .with_context(|| format!("Failed to inspect build artifact {}", source_path.display()))?;
        let file_type = metadata.file_type();

        if file_type.is_dir() {
            fs::create_dir_all(&dest_path)
                .with_context(|| format!("Failed to create release directory {}", dest_path.display()))?;
            copy_hardened(&source_path, &dest_path, tree_root)?;
            continue;
        }

        if file_type.is_file() {
            fs::copy(&source_path, &dest_path).with_context(|| {
                format!("Failed to copy build artifact {} into {}", source_path.display(), dest_path.display())
            })?;
            continue;
        }

        if file_type.is_symlink() {
            let target = fs::read_link(&source_path)
                .with_context(|| format!("Failed to read symlink {}", source_path.display()))?;
            validate_symlink_target(&source_path, &target, tree_root)?;
            symlink(&target, &dest_path)
                .with_context(|| format!("Failed to recreate symlink {}", dest_path.display()))?;
            continue;
        }

        bail!("Unsupported artifact type in promoted release: {}", source_path.display());
    }

    Ok(())
}

fn validate_symlink_target(link_path: &Path, target: &Path, tree_root: &Path) -> Result<()> {
    if target.is_absolute() {
        bail!("Absolute symlink is not allowed in release artifacts: {} -> {}", link_path.display(), target.display());
    }

    let link_parent = link_path.parent().unwrap_or(tree_root);
    let candidate = normalize_relative_path(&link_parent.join(target), tree_root)?;
    if !candidate.starts_with(tree_root) {
        bail!("Symlink escapes release tree: {} -> {}", link_path.display(), target.display());
    }

    Ok(())
}

fn normalize_relative_path(path: &Path, root: &Path) -> Result<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(_) | Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normalized == root || !normalized.pop() {
                    bail!("Path escapes release tree: {}", path.display());
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Ok(normalized)
}

fn seal_release(destination: &Path, cfg: &registry::Registry) -> Result<()> {
    use std::os::unix::fs::MetadataExt;

    let gid = site_group_gid(&cfg.runtime_group)?;
    let uid = root_uid()?;

    let metadata = fs::symlink_metadata(destination)
        .with_context(|| format!("Failed to inspect {} for sealing", destination.display()))?;
    if metadata.file_type().is_symlink() {
        return Ok(());
    }

    chown(destination, Some(uid), Some(gid))
        .with_context(|| format!("Failed to chown {} to root:{}", destination.display(), cfg.runtime_group))?;

    let mode = if metadata.file_type().is_dir() {
        0o750
    } else if metadata.mode() & 0o111 != 0 {
        0o750
    } else {
        0o640
    };
    fs::set_permissions(destination, fs::Permissions::from_mode(mode))
        .with_context(|| format!("Failed to set permissions on {}", destination.display()))?;

    if metadata.file_type().is_dir() {
        for entry in fs::read_dir(destination)
            .with_context(|| format!("Failed to read {} for sealing", destination.display()))?
        {
            let entry = entry?;
            seal_release(&entry.path(), cfg)?;
        }
    }

    Ok(())
}

fn root_uid() -> Result<u32> {
    let passwd = fs::read_to_string("/etc/passwd").context("Failed to read /etc/passwd while sealing release")?;
    let line = passwd.lines().find(|line| line.starts_with("root:")).context("root entry missing from /etc/passwd")?;
    let fields: Vec<&str> = line.split(':').collect();
    let uid = fields
        .get(2)
        .context("root passwd line missing uid field")?
        .parse::<u32>()
        .context("root uid is not a valid integer")?;
    Ok(uid)
}

fn site_group_gid(group: &str) -> Result<u32> {
    let groupfile = fs::read_to_string("/etc/group").context("Failed to read /etc/group while sealing release")?;
    let line = groupfile
        .lines()
        .find(|line| line.starts_with(&format!("{group}:")))
        .with_context(|| format!("Site group '{group}' is missing from /etc/group"))?;
    let fields: Vec<&str> = line.split(':').collect();
    let gid = fields
        .get(2)
        .with_context(|| format!("Group '{group}' missing gid field"))?
        .parse::<u32>()
        .with_context(|| format!("Group '{group}' gid is not a valid integer"))?;
    Ok(gid)
}

fn clear_directory_children(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read release directory {}", path.display()))? {
        let entry = entry?;
        let entry_type = entry.file_type()?;
        if entry_type.is_dir() {
            fs::remove_dir_all(entry.path())?;
        } else {
            fs::remove_file(entry.path())?;
        }
    }
    Ok(())
}

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use super::{clear_directory_children, normalize_relative_path, validate_symlink_target};
    use std::env;
    use std::fs;
    use std::path::Path;
    use std::process;

    use anyhow::Result;

    #[test]
    fn clear_directory_children_only_removes_entries() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-promote-clear-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("file.txt"), "x")?;
        fs::create_dir_all(root.join("nested"))?;

        clear_directory_children(&root)?;

        assert!(root.exists(), "clear must not remove the directory itself");
        assert!(fs::read_dir(&root)?.next().is_none());

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn normalize_relative_path_rejects_escape() {
        let root = Path::new("/tmp/release-root");
        let escaped = normalize_relative_path(Path::new("/tmp/release-root/app/../../etc/passwd"), root);
        assert!(escaped.is_err());
    }

    #[test]
    fn validate_symlink_target_rejects_absolute_and_escaping_targets() {
        let root = Path::new("/tmp/release-root");
        assert!(validate_symlink_target(Path::new("/tmp/release-root/x"), Path::new("/etc/passwd"), root).is_err());
        assert!(
            validate_symlink_target(Path::new("/tmp/release-root/public/x"), Path::new("../../evil"), root).is_err()
        );
    }
}

```

`crates/bonesremote/src/commands/release_checkout.rs`:

```rs
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, thread_local};

use anyhow::{Context, Result, bail};
use shared::paths;
use shared::registry;

use crate::privileges;

thread_local! {
    static SITES_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
struct ScopedRoot(Option<PathBuf>);

#[cfg(test)]
fn set_sites_root_for_tests(root: PathBuf) -> ScopedRoot {
    let prev = SITES_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root)));
    ScopedRoot(prev)
}

#[cfg(test)]
impl Drop for ScopedRoot {
    fn drop(&mut self) {
        let previous = self.0.take();
        SITES_ROOT_OVERRIDE.with(|slot| {
            slot.replace(previous);
        });
    }
}

fn resolved_sites_root() -> PathBuf {
    SITES_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_sites_root)
}

fn resolved_tmp_root(site: &str) -> PathBuf {
    resolved_sites_root().join(site).join(paths::TMP_BUILDS_DIR)
}

pub fn run(site: &str, revision: &str, context_dir: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release checkout")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let archive_output = Command::new("git")
        .args(["--git-dir", &cfg.repo_path, "archive", "--format=tar", revision])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run git archive for revision {revision} in {}", cfg.repo_path))?;

    let mut archive = archive_output.stdout.context("git archive stdout was not piped")?;
    let stderr = archive_output.stderr.context("git archive stderr was not piped")?;

    let mut tar = Command::new("tar")
        .args(["-xf", "-", "-C"])
        .arg(&context_dir)
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to start tar extraction into {}", context_dir.display()))?;

    let mut tar_stdin = tar.stdin.take().context("tar stdin was not piped")?;
    io::copy(&mut archive, &mut tar_stdin).context("Failed to stream git archive into tar")?;
    drop(tar_stdin);

    let tar_output = tar.wait_with_output().context("Failed to finish tar extraction")?;
    let git_stderr = String::from_utf8_lossy(&archive_stderr_handle(stderr)?).into_owned();

    if !tar_output.status.success() {
        bail!(
            "Failed to extract source archive into build context {}\n{}",
            context_dir.display(),
            String::from_utf8_lossy(&tar_output.stderr)
        );
    }

    if !git_stderr.is_empty() {
        println!("[bonesdeploy] git archive reported: {git_stderr}");
    }

    println!("Exported source for {revision} into {}", context_dir.display());
    Ok(())
}

fn archive_stderr_handle<R: Read>(mut reader: R) -> Result<Vec<u8>> {
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    Ok(buf)
}

pub(crate) fn ensure_build_context(site: &str) -> Result<PathBuf> {
    let root = resolved_tmp_root(site);
    fs::create_dir_all(&root).with_context(|| format!("Failed to create tmp builds root: {}", root.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let context = root.join(format!("build-{site}-{nanos}"));
    fs::create_dir_all(&context).with_context(|| format!("Failed to create build context {}", context.display()))?;
    Ok(context)
}

pub fn cleanup_build_context(site: &str, context: &Path) -> Result<()> {
    if context.exists() {
        fs::remove_dir_all(context).with_context(|| format!("Failed to remove build context {}", context.display()))?;
    }
    let root = resolved_tmp_root(site);
    if root.exists() && fs::read_dir(&root)?.next().is_none() {
        fs::remove_dir(&root).ok();
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;

    use super::ensure_build_context;
    use super::run;
    use super::set_sites_root_for_tests;

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_release_checkout_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    #[test]
    fn build_context_lives_under_bonesremote_tmp_root() -> Result<()> {
        let root = temp_dir_path("tmp_root");
        let _guard = set_sites_root_for_tests(root.clone());

        let context = ensure_build_context("unitapp")?;
        let expected_root = root.join("unitapp").join("tmp");
        assert!(context.starts_with(&expected_root));

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn run_reuses_supplied_context_path() -> Result<()> {
        let root = temp_dir_path("reuse_context");
        let _guard = set_sites_root_for_tests(root.clone());
        let context = ensure_build_context("unitapp")?;

        let site_root = root.join("unitapp");
        fs::create_dir_all(&site_root)?;
        fs::write(
            site_root.join(paths::REGISTRY_TOML),
            "site = \"unitapp\"\nrepo_path = \"/nope.git\"\nsite_root = \"/srv/sites/unitapp\"\nshared_root = \"/srv/sites/unitapp/shared\"\nreleases_root = \"/srv/sites/unitapp/releases\"\ncurrent_path = \"/srv/sites/unitapp/current\"\nruntime_user = \"unitapp\"\nruntime_group = \"unitapp\"\nbranch = \"main\"\ndeploy_on_push = true\nreleases_keep = 5\n",
        )?;

        let err = match run("unitapp", "main", &context) {
            Ok(()) => anyhow::bail!("missing repo should fail"),
            Err(error) => error,
        };
        let message = err.to_string();
        assert!(
            message.contains("git archive") || message.contains("/nope.git") || message.contains("must be run as root"),
            "unexpected error: {message}"
        );
        assert!(context.exists(), "caller-owned context should remain for cleanup");

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}

```

`crates/bonesremote/src/commands/release_prepare.rs`:

```rs
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use shared::config::{Runtime, load_runtime};
use shared::paths;
use shared::paths::default_web_root;
use shared::registry;

use crate::privileges;
use crate::release::scripts as deploy_output;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release prepare")?;
    registry::validate_site_name(site)?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let scripts_dir =
        paths::bonesremote_site_root(site).join(paths::DEPLOYMENT_DIR).join(paths::DEPLOYMENT_PREPARE_DIR);
    if !scripts_dir.is_dir() {
        println!("No prepare scripts at {}; skipping prepare.", scripts_dir.display());
        return Ok(());
    }

    let scripts = list_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No prepare scripts found at {}; skipping prepare.", scripts_dir.display());
        return Ok(());
    }

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let runtime = load_runtime(&paths::bonesremote_site_root(site)).unwrap_or_else(|_| Runtime {
        web_root: default_web_root(),
        build_image: String::new(),
        runtime_user: String::new(),
        runtime_group: String::new(),
        release_group: String::new(),
    });

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running prepare script {script_name}...");

        let status = deploy_output::run_prepare_script(
            &script,
            &release_dir,
            &release_dir.join(format!("{script_name}.log")),
            &deploy_output::PrepareScriptEnv {
                project_name: &cfg.site,
                project_root: &cfg.site_root,
                runtime_user: &cfg.runtime_user,
                web_root: &runtime.web_root,
            },
        )
        .with_context(|| format!("Failed to execute prepare script {}", script.display()))?;

        if !status.success() {
            bail!("Prepare script {script_name} exited with status {status}");
        }
    }

    Ok(())
}

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::list_scripts;

    #[test]
    fn list_scripts_sorts_prepare_scripts() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-prepare-list-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("02_second.sh"), "")?;
        fs::write(root.join("01_first.sh"), "")?;
        fs::create_dir_all(root.join("nested"))?;

        let scripts = list_scripts(&root)?;

        assert_eq!(scripts, vec![root.join("01_first.sh"), root.join("02_second.sh")]);

        fs::remove_dir_all(&root).ok();
        Ok(())
    }
}

```

`crates/bonesremote/src/commands/service.rs`:

```rs
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::{paths, registry};

use crate::privileges;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;
    registry::validate_site_name(site)?;

    let service_name = paths::nginx_service_name(site);

    let status = Command::new("systemctl")
        .args(["restart", &service_name])
        .status()
        .with_context(|| format!("Failed to restart {service_name} service"))?;

    if !status.success() {
        bail!("Failed to restart {service_name} service");
    }

    println!("Restarted {service_name} service");
    Ok(())
}

#[cfg(test)]
mod tests {}

```

`crates/bonesremote/src/commands/site.rs`:

```rs
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};
use shared::{config, paths, registry};
use walkdir::WalkDir;

use crate::privileges;

const ALLOWED_TOP_LEVEL_ENTRIES: &[&str] =
    &[paths::BONES_TOML, paths::RUNTIME_TOML, paths::DEPLOYMENT_DIR, paths::HOOKS_DIR];

/// # Errors
///
/// Returns an error if the dataset is invalid or the control-plane state cannot
/// be updated safely.
pub fn import(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote site import")?;
    registry::validate_site_name(site)?;

    let sites_root = paths::bonesremote_sites_root();
    fs::create_dir_all(&sites_root).with_context(|| format!("Failed to create {}", sites_root.display()))?;

    let staging_dir = unique_site_path(&sites_root, site, "incoming");
    fs::create_dir_all(&staging_dir).with_context(|| format!("Failed to create {}", staging_dir.display()))?;

    extract_stdin_archive(&staging_dir)?;
    validate_site_dataset(site, &staging_dir)?;
    write_registry(&staging_dir)?;
    replace_site_dir(site, &staging_dir)?;
    println!("Imported site state for {site}.");
    Ok(())
}

/// # Errors
///
/// Returns an error if the site state does not exist or cannot be archived.
pub fn export(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote site export")?;
    registry::validate_site_name(site)?;

    let site_root = paths::bonesremote_site_root(site);
    if !site_root.is_dir() {
        bail!("Remote site state does not exist: {}", site_root.display());
    }

    validate_site_dataset(site, &site_root)?;
    stream_site_archive(&site_root)
}

fn extract_stdin_archive(destination: &Path) -> Result<()> {
    let status = Command::new("tar")
        .args(["-xzf", "-", "-C"])
        .arg(destination)
        .status()
        .context("Failed to run tar for site import")?;

    if status.success() {
        return Ok(());
    }

    bail!("Failed to extract remote site dataset")
}

fn validate_site_dataset(site: &str, root: &Path) -> Result<()> {
    validate_top_level_entries(root)?;
    reject_symlinks(root)?;

    let bones_path = root.join(paths::BONES_TOML);
    if !bones_path.is_file() {
        bail!("Missing {} in imported site dataset", paths::BONES_TOML);
    }

    let bones = config::load(&bones_path)?;
    if bones.project_name != site {
        bail!("Imported site dataset is for '{}', expected '{}'", bones.project_name, site);
    }

    config::load_runtime(root)?;

    Ok(())
}

fn validate_top_level_entries(root: &Path) -> Result<()> {
    for entry in fs::read_dir(root).with_context(|| format!("Failed to read {}", root.display()))? {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else { bail!("Imported dataset contains a non-UTF-8 entry") };

        if ALLOWED_TOP_LEVEL_ENTRIES.contains(&name) {
            continue;
        }

        bail!("Imported dataset contains unsupported entry: {name}");
    }

    Ok(())
}

fn reject_symlinks(root: &Path) -> Result<()> {
    for entry in WalkDir::new(root).min_depth(1) {
        let entry = entry?;
        if entry.file_type().is_symlink() {
            bail!("Imported dataset cannot contain symlinks: {}", entry.path().display());
        }
    }

    Ok(())
}

fn write_registry(root: &Path) -> Result<()> {
    let bones = config::load(&root.join(paths::BONES_TOML))?;
    let registry = registry::Registry::derive(&bones);
    let registry_toml = toml::to_string_pretty(&registry).context("Failed to serialize site registry")?;
    fs::write(root.join(paths::REGISTRY_TOML), registry_toml).context("Failed to write site registry")?;
    Ok(())
}

fn replace_site_dir(site: &str, staging_dir: &Path) -> Result<()> {
    let site_root = paths::bonesremote_site_root(site);
    let backup_dir = unique_site_path(&paths::bonesremote_sites_root(), site, "backup");
    let had_existing = site_root.exists();

    if had_existing {
        fs::rename(&site_root, &backup_dir)
            .with_context(|| format!("Failed to move existing site state {} out of the way", site_root.display()))?;
    }

    if let Err(error) = fs::rename(staging_dir, &site_root) {
        if had_existing {
            fs::rename(&backup_dir, &site_root).ok();
        }
        return Err(error).with_context(|| format!("Failed to activate {}", site_root.display()));
    }

    if had_existing {
        fs::remove_dir_all(&backup_dir).with_context(|| format!("Failed to remove {}", backup_dir.display()))?;
    }

    Ok(())
}

fn stream_site_archive(site_root: &Path) -> Result<()> {
    let mut command = Command::new("tar");
    command.arg("-czf").arg("-").arg("-C").arg(site_root);

    for entry in ALLOWED_TOP_LEVEL_ENTRIES {
        let path = site_root.join(entry);
        if path.exists() {
            command.arg(entry);
        }
    }

    let status = command.stdout(Stdio::inherit()).status().context("Failed to run tar for site export")?;
    if status.success() {
        return Ok(());
    }

    bail!("Failed to export remote site dataset")
}

fn unique_site_path(parent: &Path, site: &str, suffix: &str) -> PathBuf {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    parent.join(format!(".{site}.{suffix}.{stamp}"))
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::{validate_site_dataset, validate_top_level_entries};

    #[test]
    fn validate_top_level_entries_rejects_unexpected_file() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-site-test-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(root.join("oops.txt"), "bad")?;

        let result = validate_top_level_entries(&root);

        fs::remove_dir_all(&root)?;
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn validate_site_dataset_rejects_bad_build_image() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-site-runtime-test-{}", process::id()));
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        fs::write(
            root.join("bones.toml"),
            r#"
remote_name = "production"
project_name = "unitapp"
ssh_user = "root"
host = "example.com"
port = "22"
repo_path = "/home/git/unitapp.git"
project_root = "/srv/sites/unitapp"
branch = "main"
preview_domain = ""
deploy_on_push = false
releases = 5
ssl_enabled = false
domain = ""
email = ""
"#,
        )?;
        fs::write(root.join("runtime.toml"), "build_image = \"node:22;rm -rf /\"\n")?;

        let result = validate_site_dataset("unitapp", &root);

        fs::remove_dir_all(&root)?;
        assert!(result.is_err());
        Ok(())
    }
}

```

`crates/bonesremote/src/commands/stage_release.rs`:

```rs
use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use shared::paths;
use shared::registry;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release stage")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let project_root = Path::new(&cfg.site_root);
    require_dir(project_root, "project_root directory")?;
    require_dir(&release_state::releases_dir(&cfg), "releases")?;
    require_dir(&release_state::shared_dir(&cfg), "shared")?;

    let release_name = create_release_name()?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    fs::create_dir_all(&release_dir)
        .with_context(|| format!("Failed to create release dir: {}", release_dir.display()))?;

    release_state::write_staged_release(site, &release_name)?;

    println!("Staged release: {release_name}");
    Ok(())
}

fn require_dir(path: &Path, label: &str) -> Result<()> {
    if !path.is_dir() {
        bail!(
            "Site not provisioned: {} does not exist ({label}). Run 'bonesdeploy remote setup' first.",
            path.display()
        );
    }
    Ok(())
}

fn create_release_name() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]_[hour][minute][second]");
    let now = OffsetDateTime::now_utc();
    now.format(TIMESTAMP_FORMAT).context("Failed to format release timestamp")
}

#[cfg(test)]
mod tests {}

```

`crates/bonesremote/src/commands/status.rs`:

```rs
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;
use shared::config;
use shared::paths;

#[derive(Debug, Serialize)]
struct Report {
    current_release: String,
    ssl: SslStatus,
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Serialize)]
struct SslStatus {
    enabled: bool,
    domain: String,
}

#[derive(Clone, Debug, Serialize)]
struct ServiceStatus {
    name: String,
    kind: String,
    state: String,
    enabled: String,
}

pub fn run(site: &str) -> Result<()> {
    let report = build_report(site)?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn build_report(site: &str) -> Result<Report> {
    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path).context("Failed to load site registry")?;
    let runtime = config::load_runtime(&paths::bonesremote_site_root(site))?;
    let deployment = cfg.deployment_paths(&runtime.web_root);

    Ok(Report {
        current_release: current_release(&deployment.current),
        ssl: ssl_status(&cfg, &deployment.nginx_site_available),
        services: services(&cfg.project_name),
    })
}

fn current_release(current_path: &str) -> String {
    fs::read_link(current_path).map_or_else(
        |_| String::from("unknown"),
        |path| path.file_name().map_or_else(|| String::from("unknown"), |name| name.to_string_lossy().to_string()),
    )
}

fn ssl_status(cfg: &config::Bones, nginx_config_path: &str) -> SslStatus {
    let enabled = !cfg.domain.is_empty()
        && fs::read_to_string(nginx_config_path).is_ok_and(|content| {
            content.contains(&format!("server_name {};", cfg.domain)) && content.contains("listen 443 ssl;")
        });

    SslStatus { enabled, domain: cfg.domain.clone() }
}

fn services(project_name: &str) -> Vec<ServiceStatus> {
    let expected = paths::nginx_service_name(project_name);
    let mut services = BTreeMap::from([(
        expected.clone(),
        ServiceStatus {
            name: expected,
            kind: String::from("site_nginx"),
            state: String::from("unknown"),
            enabled: String::from("unknown"),
        },
    )]);

    for name in discovered_service_names(project_name) {
        services.entry(name.clone()).or_insert_with(|| ServiceStatus {
            name,
            kind: String::from("discovered"),
            state: String::from("unknown"),
            enabled: String::from("unknown"),
        });
    }

    for service in services.values_mut() {
        service.state = systemctl_output(["is-active", service.name.as_str()]);
        service.enabled = systemctl_output(["is-enabled", service.name.as_str()]);
    }

    services.into_values().collect()
}

fn discovered_service_names(project_name: &str) -> Vec<String> {
    let pattern = format!("*{project_name}*.service");
    let mut names = BTreeSet::new();

    for args in [
        vec!["list-units", "--all", "--type=service", "--no-legend", "--no-pager", pattern.as_str()],
        vec!["list-unit-files", "--type=service", "--no-legend", "--no-pager", pattern.as_str()],
    ] {
        let output = Command::new("systemctl").args(args).output();
        let Ok(output) = output else { continue };
        if output.status.success() {
            names.extend(parse_systemctl_units(&String::from_utf8_lossy(&output.stdout)));
        }
    }

    names.into_iter().collect()
}

fn systemctl_output<const N: usize>(args: [&str; N]) -> String {
    Command::new("systemctl").args(args).output().map_or_else(
        |_| String::from("unknown"),
        |output| {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() { String::from("unknown") } else { value }
        },
    )
}

fn parse_systemctl_units(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| name.ends_with(".service"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_systemctl_units;

    #[test]
    fn parses_unit_names_from_systemctl_output() {
        let output = "atlas-nginx.service loaded active running Per-site nginx\natlas-worker.service loaded failed failed Worker\n";

        assert_eq!(parse_systemctl_units(output), vec!["atlas-nginx.service", "atlas-worker.service"]);
    }
}

```

`crates/bonesremote/src/commands/tests/test_doctor.rs`:

```rs
use std::collections::HashSet;

use super::{
    AppArmorUnitWiringStatus, apparmor_kernel_enabled, apparmor_profile_binding, apparmor_profile_filename,
    apparmor_unit_name_for_profile, apparmor_unit_wiring_status,
};

/// Accepts a yes value as indicating `AppArmor` is enabled in the kernel.
#[test]
fn apparmor_kernel_enabled_accepts_yes() {
    assert!(apparmor_kernel_enabled("Y\n"));
}

/// Rejects a no value as indicating `AppArmor` is not enabled in the kernel.
#[test]
fn apparmor_kernel_enabled_rejects_no() {
    assert!(!apparmor_kernel_enabled("N\n"));
}

/// Accepts a valid bonesdeploy `AppArmor` profile filename.
#[test]
fn apparmor_profile_filename_accepts_bonesdeploy_profile() {
    assert!(apparmor_profile_filename("bonesdeploy-demo-nginx"));
}

/// Rejects a filename that does not match the bonesdeploy profile naming convention.
#[test]
fn apparmor_profile_filename_rejects_unrelated_file() {
    assert!(!apparmor_profile_filename("default"));
}

/// Maps a bonesdeploy `AppArmor` profile name to its corresponding systemd unit name.
#[test]
fn apparmor_unit_name_for_profile_maps_project_unit() {
    assert_eq!(apparmor_unit_name_for_profile("bonesdeploy-demo-nginx"), Some("demo-nginx.service".to_string()));
}

/// Accepts a systemd unit with correctly wired `AppArmor` dependency and profile.
#[test]
fn apparmor_unit_wiring_accepts_expected_unit_with_reordered_after_tokens() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(matches!(
        apparmor_unit_wiring_status(
            "[Unit]\nAfter=apparmor.service network.target\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-nginx\n",
            &installed_profiles,
        ),
        AppArmorUnitWiringStatus::Ok
    ));
}

/// Rejects a systemd unit that is missing the `AppArmor` profile binding.
#[test]
fn apparmor_unit_wiring_rejects_missing_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(matches!(
        apparmor_unit_wiring_status(
            "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nType=simple\n",
            &installed_profiles,
        ),
        AppArmorUnitWiringStatus::MissingProfile
    ));
}

/// Rejects a systemd unit that binds an unknown `AppArmor` profile.
#[test]
fn apparmor_unit_wiring_rejects_unknown_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(matches!(
        apparmor_unit_wiring_status(
            "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-ngnix\n",
            &installed_profiles,
        ),
        AppArmorUnitWiringStatus::UnknownProfile(profile_name) if profile_name == "bonesdeploy-demo-ngnix"
    ));
}

/// Reads the first `AppArmor` profile assignment from a systemd unit file.
#[test]
fn apparmor_profile_binding_reads_first_profile_assignment() {
    assert_eq!(
        apparmor_profile_binding("[Service]\nAppArmorProfile=bonesdeploy-demo-nginx\n"),
        Some("bonesdeploy-demo-nginx")
    );
}

```

`crates/bonesremote/src/commands/version.rs`:

```rs
use shared::paths;

pub fn run() {
    println!("{} {}", paths::BONESREMOTE_BINARY, env!("CARGO_PKG_VERSION"));
}

```

`crates/bonesremote/src/commands/wire_shared.rs`:

```rs
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result, bail};
use shared::{paths, registry};

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release wire")?;
    registry::validate_site_name(site)?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path).context(super::deploy::registry_load_error())?;

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let shared_dir = release_state::shared_dir(&cfg);
    if !shared_dir.is_dir() {
        bail!(
            "Shared root is missing: {}. Run 'bonesdeploy remote setup' or runtime provisioning first.",
            shared_dir.display()
        );
    }

    for leaf in paths::SHARED_LEAVES {
        let target = shared_dir.join(leaf);
        ensure_shared_leaf(&target)?;
        link_relative(&release_dir, leaf, &target)?;
    }

    link_public_storage(&release_dir)?;

    Ok(())
}

fn ensure_shared_leaf(path: &Path) -> Result<()> {
    if path.exists() {
        return Ok(());
    }

    bail!("Required shared path is missing: {}. Provision the runtime shared paths before deploying.", path.display())
}

fn link_relative(release_dir: &Path, relative: &str, target: &Path) -> Result<()> {
    let link_path = release_dir.join(relative);
    remove_if_present(&link_path)?;
    symlink(target, &link_path)
        .with_context(|| format!("Failed to link {} -> {}", link_path.display(), target.display()))?;
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };

    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory {}", path.display()))?;
    }
    Ok(())
}

fn link_public_storage(release_dir: &Path) -> Result<()> {
    let public_dir = release_dir.join("public");
    if !public_dir.is_dir() {
        return Ok(());
    }

    let link_path = public_dir.join("storage");
    remove_if_present(&link_path)?;
    symlink(Path::new("../storage/app/public"), &link_path)
        .with_context(|| format!("Failed to link {} -> ../storage/app/public", link_path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process;

    use anyhow::Result;

    use super::{ensure_shared_leaf, link_public_storage, link_relative, remove_if_present};

    fn temp_dir(label: &str) -> Result<PathBuf> {
        let dir = env::temp_dir().join(format!("bonesremote-wire-{label}-{}", process::id()));
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        fs::create_dir_all(&dir)?;
        Ok(dir)
    }

    #[test]
    fn ensure_shared_leaf_requires_existing_path() -> Result<()> {
        let root = temp_dir("ensure_leaves")?;
        let missing = root.join("storage");
        assert!(ensure_shared_leaf(&missing).is_err());

        fs::create_dir_all(&missing)?;
        ensure_shared_leaf(&missing)?;

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn link_relative_creates_symlink_to_shared_target() -> Result<()> {
        let root = temp_dir("link_relative")?;
        let shared = root.join("shared/.env");
        let parent = shared.parent().ok_or_else(|| anyhow::anyhow!("shared test path should have a parent"))?;
        fs::create_dir_all(parent)?;
        fs::write(&shared, "FOO=bar\n")?;
        fs::set_permissions(&shared, PermissionsExt::from_mode(0o600))?;

        let release = root.join("releases/now");
        fs::create_dir_all(&release)?;
        link_relative(&release, ".env", &shared)?;

        let link = release.join(".env");
        assert!(link.is_symlink());
        let linked_target = fs::read_link(&link)?;
        assert_eq!(linked_target, shared);
        assert_eq!(fs::read_to_string(&link)?, "FOO=bar\n");

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn remove_if_present_handles_files_dirs_and_missing() -> Result<()> {
        let root = temp_dir("remove_if_present")?;
        let missing = root.join("missing");
        remove_if_present(&missing)?;

        let file = root.join("file.txt");
        fs::write(&file, "x")?;
        remove_if_present(&file)?;
        assert!(!file.exists());

        let dir = root.join("dir");
        fs::create_dir_all(&dir)?;
        remove_if_present(&dir)?;
        assert!(!dir.exists());

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn link_public_storage_links_into_release_storage_tree() -> Result<()> {
        let root = temp_dir("public_storage")?;
        let release = root.join("releases/now");
        fs::create_dir_all(release.join("public"))?;

        link_public_storage(&release)?;

        let link = release.join("public/storage");
        assert!(link.is_symlink());
        assert_eq!(fs::read_link(&link)?, PathBuf::from("../storage/app/public"));

        fs::remove_dir_all(&root).ok();
        Ok(())
    }
}

```

`crates/bonesremote/src/main.rs`:

```rs
mod cli;
mod commands;
mod privileges;
mod release;
mod release_state;

use anyhow::Result;
use clap::Parser;
use commands::Cli;

fn main() -> Result<()> {
    let cli = Cli::parse();
    commands::run(&cli)
}

```

`crates/bonesremote/src/privileges.rs`:

```rs
use anyhow::{Result, bail};
use nix::unistd::geteuid;

pub fn ensure_root(command_name: &str) -> Result<()> {
    if geteuid().is_root() {
        return Ok(());
    }

    bail!("{command_name} must be run as root (sudo)")
}

```

`crates/bonesremote/src/release/mod.rs`:

```rs
pub(crate) mod scripts;

```

`crates/bonesremote/src/release/scripts.rs`:

```rs
use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};

#[cfg(test)]
pub(super) struct HostScriptEnv<'a> {
    pub(super) project_name: &'a str,
    pub(super) project_root: &'a str,
    pub(super) repo_path: &'a str,
    pub(super) web_root: &'a str,
}

pub(crate) struct BuildScriptEnv<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) web_root: &'a str,
    pub(crate) build_image: &'a str,
}

pub(crate) struct PrepareScriptEnv<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) project_root: &'a str,
    pub(crate) runtime_user: &'a str,
    pub(crate) web_root: &'a str,
}

#[cfg(test)]
pub(super) fn run_deployment_script(
    script: &Path,
    build_root: &Path,
    log_path: &Path,
    env: &HostScriptEnv<'_>,
) -> Result<ExitStatus> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create log directory {}", parent.display()))?;
    }

    let mut child = Command::new("bash")
        .arg("-c")
        .arg("umask 0002\nexec bash \"$@\"")
        .arg("bonesdeploy-umask")
        .arg(script)
        .current_dir(build_root)
        .env("PROJECT_NAME", env.project_name)
        .env("PROJECT_ROOT", env.project_root)
        .env("REPO_PATH", env.repo_path)
        .env("WEB_ROOT", env.web_root)
        .env("SERVICE_USER", env.project_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to execute deployment script {}", script.display()))?;

    stream_child_output(&mut child, log_path, &format!("deployment script {}", script.display()))
}

pub(crate) fn run_podman_build_script(
    script: &Path,
    source_root: &Path,
    log_path: &Path,
    env: &BuildScriptEnv<'_>,
) -> Result<ExitStatus> {
    let script_file =
        fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;

    let mut command = Command::new("podman");
    configure_podman_build_command(&mut command, source_root, env);

    let mut child = command
        .stdin(Stdio::from(script_file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to execute build script {} in podman", script.display()))?;

    stream_child_output(&mut child, log_path, &format!("podman build script {}", script.display()))
}

pub(crate) fn run_prepare_script(
    script: &Path,
    release_root: &Path,
    log_path: &Path,
    env: &PrepareScriptEnv<'_>,
) -> Result<ExitStatus> {
    let script_file =
        fs::File::open(script).with_context(|| format!("Failed to open prepare script {}", script.display()))?;

    let mut command = Command::new("runuser");
    configure_prepare_command(&mut command, release_root, env);

    let mut child =
        command.stdin(Stdio::from(script_file)).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().with_context(
            || format!("Failed to execute prepare script {} as {}", script.display(), env.runtime_user),
        )?;

    stream_child_output(&mut child, log_path, &format!("prepare script {}", script.display()))
}

fn configure_podman_build_command(command: &mut Command, source_root: &Path, env: &BuildScriptEnv<'_>) {
    let mount = format!("{}:/workspace/source", source_root.display());
    command
        .args([
            "run",
            "--rm",
            "--pull=missing",
            "--security-opt=no-new-privileges",
            "--cap-drop=all",
            "--workdir=/workspace/source",
            "--volume",
        ])
        .arg(mount)
        .arg("--env")
        .arg(format!("PROJECT_NAME={}", env.project_name))
        .arg("--env")
        .arg("PROJECT_ROOT=/workspace")
        .arg("--env")
        .arg("REPO_PATH=")
        .arg("--env")
        .arg(format!("WEB_ROOT={}", env.web_root))
        .arg("--env")
        .arg(format!("SERVICE_USER={}", env.project_name))
        .arg(env.build_image)
        .args(["bash", "-c", "umask 0002; exec bash -s"]);
}

fn configure_prepare_command(command: &mut Command, release_root: &Path, env: &PrepareScriptEnv<'_>) {
    command
        .args(["-u", env.runtime_user, "--", "bash", "-c", "umask 0002; exec bash -s"])
        .current_dir(release_root)
        .env("PROJECT_NAME", env.project_name)
        .env("PROJECT_ROOT", env.project_root)
        .env("REPO_PATH", "")
        .env("WEB_ROOT", env.web_root)
        .env("SERVICE_USER", env.runtime_user);
}

fn stream_child_output(child: &mut Child, log_path: &Path, label: &str) -> Result<ExitStatus> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create log directory {}", parent.display()))?;
    }

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("Failed to open deployment log {}", log_path.display()))?;

    let stdout = child.stdout.take().context("Failed to capture deployment stdout")?;
    let stderr = child.stderr.take().context("Failed to capture deployment stderr")?;

    let stdout_handle =
        spawn_stream(stdout, io::stdout(), log_file.try_clone().context("Failed to clone deployment log")?);
    let stderr_handle = spawn_stream(stderr, io::stderr(), log_file);

    let status = child.wait().with_context(|| format!("Failed to wait for {label}"))?;

    join_stream(stdout_handle, "stdout")?;
    join_stream(stderr_handle, "stderr")?;

    Ok(status)
}

fn spawn_stream<R, W1, W2>(reader: R, primary: W1, secondary: W2) -> thread::JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
    W1: Write + Send + 'static,
    W2: Write + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = reader;
        let mut primary = primary;
        let mut secondary = secondary;
        let mut buffer = [0_u8; 8192];

        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            primary.write_all(&buffer[..read])?;
            secondary.write_all(&buffer[..read])?;
        }

        primary.flush()?;
        secondary.flush()?;
        Ok(())
    })
}

fn join_stream(handle: thread::JoinHandle<Result<()>>, stream_name: &str) -> Result<()> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => bail!("Deployment output thread for {stream_name} panicked"),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use std::os::unix::prelude::PermissionsExt;

    use super::{
        BuildScriptEnv, HostScriptEnv, PrepareScriptEnv, configure_podman_build_command, configure_prepare_command,
        run_deployment_script,
    };

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{nanos}"));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn write_file(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    #[test]
    fn run_deployment_script_streams_output_to_console_and_log() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_runner_stream")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let script = root.join("00_hello.sh");
        write_file(&script, "#!/usr/bin/env bash\necho 'hello-stdout'\necho 'hello-stderr' >&2\n")?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let log_path = logs.join("20260612_211412-00_hello.sh.log");
        let status = run_deployment_script(
            &script,
            &build_root,
            &log_path,
            &HostScriptEnv {
                project_name: "demo",
                project_root: "/srv/deployments/demo",
                repo_path: "/home/git/demo.git",
                web_root: "public",
            },
        )?;

        assert!(status.success(), "passing script should exit zero");

        let log = fs::read_to_string(&log_path)?;
        assert!(log.contains("hello-stdout"), "log should contain stdout\n{log}");
        assert!(log.contains("hello-stderr"), "log should contain stderr\n{log}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn run_deployment_script_preserves_failing_exit_status() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_runner_failing")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let script = root.join("01_install.sh");
        write_file(&script, "#!/usr/bin/env bash\necho 'about to fail' >&2\nexit 7\n")?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let log_path = logs.join("20260612_211412-01_install.sh.log");
        let status = run_deployment_script(
            &script,
            &build_root,
            &log_path,
            &HostScriptEnv {
                project_name: "demo",
                project_root: "/srv/deployments/demo",
                repo_path: "/home/git/demo.git",
                web_root: "public",
            },
        )?;

        assert!(!status.success(), "failing script should exit non-zero");
        assert_eq!(status.code(), Some(7), "failing script should preserve exit code 7");
        let log = fs::read_to_string(&log_path)?;
        assert!(log.contains("about to fail"), "log should still be written for failing script\n{log}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn run_deployment_script_applies_group_writable_umask() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_runner_umask")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let out_file = build_root.join("umask_probe.txt");
        let script = root.join("00_probe.sh");
        write_file(&script, &format!("#!/usr/bin/env bash\necho hi > \"{}\"\n", out_file.display()))?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let log_path = logs.join("20260612_211412-00_probe.sh.log");
        let status = run_deployment_script(
            &script,
            &build_root,
            &log_path,
            &HostScriptEnv {
                project_name: "demo",
                project_root: "/srv/deployments/demo",
                repo_path: "/home/git/demo.git",
                web_root: "public",
            },
        )?;

        assert!(status.success());
        let mode = fs::metadata(&out_file)?.permissions().mode() & 0o777;
        assert_eq!(mode, 0o664, "umask 0002 should make created files group-writable (0664), got {mode:o}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn podman_build_command_mounts_only_source_tree() {
        let mut command = Command::new("podman");
        configure_podman_build_command(
            &mut command,
            Path::new("/tmp/source"),
            &BuildScriptEnv {
                project_name: "demo",
                web_root: "public",
                build_image: "docker.io/library/node:22-bookworm",
            },
        );

        let args = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();
        assert!(args.contains(&String::from("--rm")));
        assert!(args.contains(&String::from("--security-opt=no-new-privileges")));
        assert!(args.contains(&String::from("--cap-drop=all")));
        assert!(args.contains(&String::from("/tmp/source:/workspace/source")));
        assert!(!args.iter().any(|arg| arg.contains("/srv/sites/demo/shared")));
        assert!(!args.iter().any(|arg| arg.contains("/root/.config/bonesremote")));
    }

    #[test]
    fn prepare_command_runs_as_runtime_user_in_release() {
        let mut command = Command::new("runuser");
        configure_prepare_command(
            &mut command,
            Path::new("/srv/sites/demo/releases/20260626_120000"),
            &PrepareScriptEnv {
                project_name: "demo",
                project_root: "/srv/sites/demo",
                runtime_user: "demo",
                web_root: "public",
            },
        );

        let args = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();
        assert_eq!(args[0], "-u");
        assert_eq!(args[1], "demo");
        assert_eq!(command.get_current_dir(), Some(Path::new("/srv/sites/demo/releases/20260626_120000")));
        assert!(!args.iter().any(|arg| arg.contains("/root/.config/bonesremote")));
        let service_user = command.get_envs().find_map(|(key, value)| {
            (key.to_string_lossy() == "SERVICE_USER")
                .then(|| value.map(|value| value.to_string_lossy().into_owned()))
                .flatten()
        });
        assert_eq!(service_user, Some(String::from("demo")));
        assert!(!args.iter().any(|arg| arg.contains("podman")));
    }
}

```

`crates/bonesremote/src/release_state.rs`:

```rs
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, thread_local};

use anyhow::{Context, Result, bail};

use shared::paths;
use shared::registry::Registry;

thread_local! {
    static SITES_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_sites_root_for_tests(root: PathBuf) -> ScopedRoot {
    let prev = SITES_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root)));
    ScopedRoot(prev)
}

#[cfg(test)]
pub(crate) struct ScopedRoot(Option<PathBuf>);

#[cfg(test)]
impl Drop for ScopedRoot {
    fn drop(&mut self) {
        let previous = self.0.take();
        SITES_ROOT_OVERRIDE.with(|slot| {
            slot.replace(previous);
        });
    }
}

fn resolved_sites_root() -> PathBuf {
    SITES_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_sites_root)
}

fn resolved_site_root(site: &str) -> PathBuf {
    resolved_sites_root().join(site)
}

pub fn staged_release_path(site: &str) -> PathBuf {
    resolved_site_root(site).join(paths::STAGED_RELEASE_FILE)
}

pub fn read_staged_release(site: &str) -> Result<String> {
    let path = staged_release_path(site);
    let value = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read staged release state at {}", path.display()))?;
    let release = value.trim().to_string();

    if release.is_empty() {
        bail!("Staged release state file is empty: {}", path.display());
    }

    Ok(release)
}

pub fn write_staged_release(site: &str, release: &str) -> Result<()> {
    let path = staged_release_path(site);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create staged release state dir: {}", parent.display()))?;
    }

    fs::write(&path, format!("{release}\n"))
        .with_context(|| format!("Failed to write staged release state: {}", path.display()))
}

pub fn clear_staged_release(site: &str) -> Result<()> {
    let path = staged_release_path(site);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Failed to remove staged release state: {}", path.display()))?;
    }
    Ok(())
}

pub fn release_dir(cfg: &Registry, release: &str) -> PathBuf {
    releases_dir(cfg).join(release)
}

pub fn releases_dir(cfg: &Registry) -> PathBuf {
    PathBuf::from(&cfg.releases_root)
}

pub fn shared_dir(cfg: &Registry) -> PathBuf {
    PathBuf::from(&cfg.shared_root)
}

pub fn current_release_dir(cfg: &Registry) -> Result<PathBuf> {
    let current_link = PathBuf::from(&cfg.current_path);
    let active_target =
        fs::read_link(&current_link).with_context(|| format!("Failed to read {}", current_link.display()))?;

    Ok(if active_target.is_absolute() {
        active_target
    } else {
        current_link.parent().unwrap_or_else(|| Path::new("/")).join(active_target)
    })
}

pub fn current_release_name(cfg: &Registry) -> Result<String> {
    let current_release = current_release_dir(cfg)?;
    current_release
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve current release name from {}", current_release.display()))
}

pub fn list_releases_sorted(cfg: &Registry) -> Result<Vec<String>> {
    let releases_dir = releases_dir(cfg);
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&releases_dir)
        .with_context(|| format!("Failed to read releases dir: {}", releases_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != paths::PLACEHOLDER_RELEASE_NAME {
                names.push(name);
            }
        }
    }

    names.sort();
    Ok(names)
}

pub fn point_symlink_atomically(link_path: &Path, target_path: &Path) -> Result<()> {
    let Some(parent) = link_path.parent() else {
        bail!("Invalid symlink path: {}", link_path.display());
    };

    fs::create_dir_all(parent).with_context(|| format!("Failed to create symlink parent: {}", parent.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    let temp_name = format!(".tmp_current_{}_{}", process::id(), nanos);
    let temp_link = parent.join(temp_name);

    if fs::symlink_metadata(&temp_link).is_ok() {
        fs::remove_file(&temp_link)
            .with_context(|| format!("Failed to cleanup stale temp link: {}", temp_link.display()))?;
    }

    symlink(target_path, &temp_link).with_context(|| {
        format!("Failed to create temporary symlink {} -> {}", temp_link.display(), target_path.display())
    })?;

    fs::rename(&temp_link, link_path).with_context(|| {
        format!("Failed to atomically switch symlink {} -> {}", link_path.display(), target_path.display())
    })
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;
    use shared::registry::Registry;

    use super::{
        ScopedRoot, clear_staged_release, current_release_name, list_releases_sorted, point_symlink_atomically,
        read_staged_release, releases_dir, set_sites_root_for_tests, staged_release_path, write_staged_release,
    };

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_release_state_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    fn temp_root(test_name: &str) -> Result<(ScopedRoot, PathBuf)> {
        let path = temp_dir_path(test_name);
        fs::create_dir_all(&path)?;
        Ok((set_sites_root_for_tests(path.clone()), path))
    }

    fn sample_config(root: &Path, site: &str) -> Registry {
        let site_root = root.join("deploy");
        Registry {
            site: String::from(site),
            repo_path: root.join("repo.git").to_string_lossy().to_string(),
            site_root: site_root.to_string_lossy().to_string(),
            shared_root: site_root.join(paths::SHARED_DIR).to_string_lossy().to_string(),
            releases_root: site_root.join(paths::RELEASES_DIR).to_string_lossy().to_string(),
            current_path: site_root.join(paths::CURRENT_LINK).to_string_lossy().to_string(),
            runtime_user: String::from(site),
            runtime_group: String::from(site),
            branch: String::from("master"),
            deploy_on_push: true,
            releases_keep: 5,
        }
    }

    #[test]
    fn write_then_read_staged_release_round_trips() -> Result<()> {
        let (_guard, _root) = temp_root("round_trip")?;

        write_staged_release("unitapp", "20260507_151500")?;
        let release_name = read_staged_release("unitapp")?;
        assert_eq!(release_name, "20260507_151500");

        Ok(())
    }

    #[test]
    fn read_staged_release_rejects_empty_file() -> Result<()> {
        let (_guard, root) = temp_root("empty_state")?;
        let state_path = root.join("emptyapp").join(paths::STAGED_RELEASE_FILE);
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&state_path, " \n")?;

        let result = read_staged_release("emptyapp");
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    fn clear_staged_release_removes_state_file() -> Result<()> {
        let (_guard, _root) = temp_root("clear_state")?;

        write_staged_release("clearapp", "20260507_151501")?;
        clear_staged_release("clearapp")?;
        assert!(!staged_release_path("clearapp").exists());

        Ok(())
    }

    #[test]
    fn point_symlink_atomically_creates_parent_dirs_and_points_to_target() -> Result<()> {
        let root = temp_dir_path("point_symlink_parent");
        fs::create_dir_all(&root)?;

        let target = root.join("target_dir");
        fs::create_dir_all(&target)?;

        let link_path = root.join("nested/path/current");
        point_symlink_atomically(&link_path, &target)?;

        assert!(link_path.exists());
        let linked = fs::read_link(&link_path)?;
        assert_eq!(linked, target);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn point_symlink_atomically_repoints_existing_link() -> Result<()> {
        let root = temp_dir_path("point_symlink_repoint");
        fs::create_dir_all(&root)?;

        let target_a = root.join("target_a");
        let target_b = root.join("target_b");
        fs::create_dir_all(&target_a)?;
        fs::create_dir_all(&target_b)?;

        let link_path = root.join(paths::CURRENT_LINK);
        point_symlink_atomically(&link_path, &target_a)?;
        point_symlink_atomically(&link_path, &target_b)?;

        let linked = fs::read_link(&link_path)?;
        assert_eq!(linked, target_b);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn list_releases_sorted_returns_only_directories_in_order() -> Result<()> {
        let root = temp_dir_path("list_releases");
        let cfg = sample_config(&root, "listapp");
        fs::create_dir_all(&root)?;

        let releases = releases_dir(&cfg);
        fs::create_dir_all(&releases)?;
        fs::create_dir_all(releases.join("20260507_120000"))?;
        fs::create_dir_all(releases.join("20260507_110000"))?;
        fs::create_dir_all(releases.join(paths::PLACEHOLDER_RELEASE_NAME))?;
        fs::write(releases.join("notes.txt"), "not a release")?;

        let items = list_releases_sorted(&cfg)?;
        assert_eq!(items, vec![String::from("20260507_110000"), String::from("20260507_120000")]);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn current_release_name_resolves_from_current_symlink() -> Result<()> {
        let root = temp_dir_path("current_release_name");
        fs::create_dir_all(&root)?;
        let cfg = sample_config(&root, "currentapp");

        let releases_dir = releases_dir(&cfg);
        let release = releases_dir.join("20260507_170000");
        fs::create_dir_all(&release)?;

        let current = PathBuf::from(cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).current);
        point_symlink_atomically(&current, &release)?;

        let name = current_release_name(&cfg)?;
        assert_eq!(name, "20260507_170000");

        fs::remove_dir_all(root)?;
        Ok(())
    }
}

```

`crates/shared/Cargo.toml`:

```toml
[package]
name = "shared"
version = "0.1.0"
edition = "2021"

[dependencies]
anyhow = "1.0.102"
serde = { version = "1", features = ["derive"] }
toml = "0.8"

[lints.clippy]
# broad groups (lower priority so individual lint overrides take effect)
correctness = { level = "deny", priority = -1 }
suspicious = { level = "deny", priority = -1 }
complexity = { level = "warn", priority = -1 }
style = { level = "allow", priority = -1 }
perf = { level = "warn", priority = -1 }
pedantic = { level = "warn", priority = -1 }

# readability / naming
similar_names = "warn"
many_single_char_names = "warn"
module_name_repetitions = "warn"
enum_variant_names = "warn"
struct_field_names = "warn"
disallowed_names = "deny"
wildcard_imports = "warn"
module_inception = "warn"

# complexity / API readability
cognitive_complexity = "warn"
too_many_arguments = "deny"
fn_params_excessive_bools = "warn"
large_types_passed_by_value = "warn"
trivially_copy_pass_by_ref = "warn"

# panic / debug leftovers
unwrap_used = "deny"
expect_used = "deny"
panic = "deny"
todo = "deny"
unimplemented = "deny"
dbg_macro = "deny"
# This is a CLI binary: stdout/stderr output is the intended UX.
print_stdout = "allow"
print_stderr = "allow"

# docs / contracts
missing_panics_doc = "deny"
missing_errors_doc = "deny"
missing_safety_doc = "deny"
undocumented_unsafe_blocks = "deny"

# numeric safety
cast_possible_truncation = "warn"
cast_sign_loss = "warn"
cast_possible_wrap = "warn"
checked_conversions = "warn"

# anti-mess / consistency
absolute_paths = "warn"
allow_attributes = "deny"
redundant_clone = "warn"
implicit_clone = "warn"
semicolon_if_nothing_returned = "warn"
match_same_arms = "warn"
needless_pass_by_value = "warn"
cloned_instead_of_copied = "warn"
flat_map_option = "warn"
from_iter_instead_of_collect = "warn"
inefficient_to_string = "warn"
manual_let_else = "warn"
manual_ok_or = "warn"
map_unwrap_or = "warn"
unnecessary_wraps = "warn"


```

`crates/shared/src/config.rs`:

```rs
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::paths::{self, Deployment};

/// Keys in the JSON object that bonesdeploy sends to bonesinfra.
pub mod bonesinfra_input {
    pub const SSH_PORT: &str = "ssh_port";
    pub const SSH_USER: &str = "ssh_user";
    pub const DEPLOY_USER: &str = "deploy_user";
    pub const PROJECT_ROOT: &str = "project_root";
    pub const RUNTIME_USER: &str = "runtime_user";
    pub const RUNTIME_GROUP: &str = "runtime_group";
    pub const RELEASE_GROUP: &str = "release_group";
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Bones {
    pub remote_name: String,
    pub project_name: String,
    pub ssh_user: String,
    pub host: String,
    pub port: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub repo_path: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub project_root: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub branch: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub preview_domain: String,
    pub deploy_on_push: bool,
    #[serde(rename = "releases")]
    pub releases_keep: usize,
    pub ssl_enabled: bool,
    pub domain: String,
    pub email: String,
}

impl Default for Bones {
    fn default() -> Self {
        Self {
            remote_name: String::new(),
            project_name: String::new(),
            ssh_user: String::from("root"),
            host: String::new(),
            port: "22".into(),
            repo_path: String::new(),
            project_root: String::new(),
            branch: "master".into(),
            preview_domain: String::new(),
            deploy_on_push: false,
            releases_keep: 5,
            ssl_enabled: false,
            domain: String::new(),
            email: String::new(),
        }
    }
}

impl Bones {
    #[must_use]
    pub fn deployment_paths(&self, web_root: &str) -> Deployment {
        Deployment::new(&self.project_name, &self.repo_path, &self.project_root, web_root)
    }
}

#[must_use]
pub fn default_deploy_user() -> String {
    paths::DEPLOY_USER.to_string()
}

/// # Errors
///
/// Returns an error if the string cannot be parsed as a `u16` port number.
pub fn parse_port(port: &str) -> Result<u16> {
    port.parse().with_context(|| format!("Invalid port: {port}"))
}

/// # Errors
///
/// Returns an error if the host contains characters that are not ASCII
/// alphanumeric, dots, or dashes.
pub fn validate_host(host: &str) -> Result<()> {
    let host = host.trim();
    if host.is_empty() {
        return Ok(());
    }

    if host.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-')) {
        return Ok(());
    }

    bail!("Invalid host: {host}")
}

#[must_use]
pub fn runtime_user_for(project_name: &str) -> String {
    project_name.to_string()
}

#[must_use]
pub fn runtime_group_for(project_name: &str) -> String {
    project_name.to_string()
}

#[must_use]
pub fn release_group_for(project_name: &str) -> String {
    format!("{project_name}-release")
}

#[must_use]
pub fn default_repo_path_for(project_name: &str) -> String {
    paths::default_repo_path_for(project_name)
}

#[must_use]
pub fn default_preview_domain_for(project_name: &str, host: &str) -> String {
    let project = sanitize_domain_label(project_name);
    let host = sanitize_domain_label(host);

    if project.is_empty() || host.is_empty() {
        return String::new();
    }

    format!("{project}-{host}.nip.io")
}

fn sanitize_domain_label(value: &str) -> String {
    value
        .trim()
        .chars()
        .map(|ch| if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' })
        .collect::<String>()
        .trim_matches('-')
        .to_string()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Runtime {
    #[serde(default = "paths::default_web_root")]
    pub web_root: String,
    #[serde(default)]
    pub build_image: String,
    #[serde(default)]
    pub runtime_user: String,
    #[serde(default)]
    pub runtime_group: String,
    #[serde(default)]
    pub release_group: String,
}

/// Loads the runtime configuration from a TOML file, falling back to defaults.
///
/// # Errors
///
/// Returns an error if the file exists but cannot be read or parsed.
pub fn load_runtime(config_dir: &Path) -> Result<Runtime> {
    let path = config_dir.join(paths::RUNTIME_TOML);
    if path.exists() {
        let content = fs::read_to_string(&path).with_context(|| format!("Failed to read {}", path.display()))?;
        let runtime: Runtime =
            toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
        validate_build_image(&runtime.build_image)?;
        Ok(runtime)
    } else {
        Ok(Runtime {
            web_root: paths::default_web_root(),
            build_image: String::new(),
            runtime_user: String::new(),
            runtime_group: String::new(),
            release_group: String::new(),
        })
    }
}

/// # Errors
///
/// Returns an error when the image reference contains shell/control characters.
pub fn validate_build_image(image: &str) -> Result<()> {
    if image.is_empty() {
        return Ok(());
    }

    if image.chars().all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-' | '/' | ':' | '@')) {
        return Ok(());
    }

    bail!("Invalid build image: {image}")
}

pub fn apply_derived_defaults(config: &mut Bones) {
    let project_name = &config.project_name;

    if config.ssh_user.is_empty() {
        config.ssh_user = String::from("root");
    }
    if config.repo_path.is_empty() {
        config.repo_path = default_repo_path_for(project_name);
    }
    if config.project_root.is_empty() {
        config.project_root = paths::default_project_root_for(project_name);
    }
    if config.preview_domain.is_empty() {
        config.preview_domain = default_preview_domain_for(project_name, &config.host);
    }
}

/// Loads and parses a `bones.toml` configuration file, applying derived defaults.
///
/// # Errors
///
/// Returns an error if the file cannot be read or the TOML is invalid.
pub fn load(path: &Path) -> Result<Bones> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let mut config: Bones = toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    apply_derived_defaults(&mut config);
    validate_host(&config.host)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_host_accepts_hostnames_and_ips() {
        assert!(validate_host("deploy.example.com").is_ok());
        assert!(validate_host("192.0.2.10").is_ok());
        assert!(validate_host("").is_ok());
    }

    #[test]
    fn validate_host_rejects_shell_metacharacters() {
        assert!(validate_host("deploy.example.com;rm -rf /").is_err());
    }

    #[test]
    fn validate_build_image_allows_common_references() {
        assert!(validate_build_image("docker.io/library/node:22-bookworm").is_ok());
        assert!(validate_build_image("ghcr.io/acme/app@sha256:abc123").is_ok());
    }

    #[test]
    fn validate_build_image_rejects_shell_metacharacters() {
        assert!(validate_build_image("node:22;rm -rf /").is_err());
        assert!(validate_build_image("node:22 $(whoami)").is_err());
    }
}

```

`crates/shared/src/lib.rs`:

```rs
pub mod config;
pub mod paths;
pub mod registry;

```

`crates/shared/src/paths.rs`:

```rs
use std::env;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

pub const DEFAULT_REPO_PARENT: &str = "/home/git";
pub const DEFAULT_PROJECT_ROOT_PARENT: &str = "/srv/sites";
pub const DEFAULT_CONF_ROOT_PARENT: &str = "/srv/conf";
pub const DEFAULT_WEB_ROOT: &str = "public";

pub const DEPLOY_USER: &str = "git";
pub const DEFAULT_GROUP: &str = "www-data";

pub const ETC_NGINX: &str = "/etc/nginx";
pub const ETC_NGINX_SITES_AVAILABLE: &str = "/etc/nginx/sites-available";
pub const ETC_NGINX_SITES_ENABLED: &str = "/etc/nginx/sites-enabled";
pub const ETC_SYSTEMD_SYSTEM: &str = "/etc/systemd/system";
pub const ETC_APPARMOR_D: &str = "/etc/apparmor.d";
pub const ETC_LETSENCRYPT_LIVE: &str = "/etc/letsencrypt/live";
pub const ETC_SUDOERS_D: &str = "/etc/sudoers.d";
pub const ETC_OS_RELEASE: &str = "/etc/os-release";
pub const APPARMOR_ENABLED_PARAM: &str = "/sys/module/apparmor/parameters/enabled";
pub const APPARMOR_PROFILES: &str = "/sys/kernel/security/apparmor/profiles";
pub const PROC_MODULES: &str = "/proc/modules";
pub const ETC_MODPROBE_D: &str = "/etc/modprobe.d";
pub const USR_LOCAL_BIN: &str = "/usr/local/bin";
pub const OPT_BONESDEPLOY: &str = "/opt/bonesdeploy";
pub const TMP_ROOT: &str = "/tmp";

pub const LOCAL_BONES_DIR: &str = ".bones";
pub const LOCAL_BONES_TOML: &str = ".bones/bones.toml";
pub const LOCAL_BONES_HOOKS_DIR: &str = ".bones/hooks";
pub const LOCAL_BONES_DEPLOYMENT_DIR: &str = ".bones/deployment";
pub const LOCAL_BONES_RUNTIME_TOML: &str = ".bones/runtime.toml";
pub const LOCAL_BONES_SECRETS_DIR: &str = ".bones/secrets";
pub const DOT_ENV: &str = ".env";
pub const RUNTIME_TOML: &str = "runtime.toml";
pub const REGISTRY_TOML: &str = "registry.toml";

pub const BONES_DIR: &str = "bones";
pub const BONES_TOML: &str = "bones.toml";
pub const NGINX_CONF: &str = "nginx.conf";
pub const INDEX_HTML: &str = "index.html";
pub const GIT_HEAD: &str = "HEAD";
pub const DEPLOYMENT_DIR: &str = "deployment";
pub const DEPLOYMENT_BUILD_DIR: &str = "build";
pub const DEPLOYMENT_PREPARE_DIR: &str = "prepare";
pub const RELEASES_DIR: &str = "releases";
pub const SHARED_DIR: &str = "shared";
pub const BUILD_DIR: &str = "build";
pub const WORKSPACE_DIR: &str = "workspace";
pub const LOGS_DIR: &str = "logs";
pub const CURRENT_LINK: &str = "current";
pub const STAGED_RELEASE_FILE: &str = "staged-release";
pub const TMP_BUILDS_DIR: &str = "tmp";
pub const INSTALL_VERSIONS_DIR: &str = "versions";
pub const INSTALL_CURRENT_LINK: &str = "current";
pub const BONESDEPLOY_SWAP_LINK: &str = ".bonesdeploy_swap";
pub const BONESREMOTE_SWAP_LINK_PREFIX: &str = ".bonesremote_swap_";
pub const PLACEHOLDER_RELEASE_NAME: &str = "19700101_000000";
pub const SUDOERS_FILE: &str = "bonesdeploy";
pub const SUDOERS_PATH: &str = "/etc/sudoers.d/bonesdeploy";
pub const BONESDEPLOY_BINARY: &str = "bonesdeploy";
pub const BONESREMOTE_BINARY: &str = "bonesremote";
pub const BONESREMOTE_CONFIG_DIR: &str = "/root/.config/bonesremote";
pub const BONESREMOTE_SITES_DIR: &str = "sites";
pub const NGINX_SOCKET: &str = "nginx.sock";
pub const NGINX_PID: &str = "nginx.pid";
pub const PHP_FPM_SOCKET: &str = "php-fpm.sock";
pub const DEFAULT_NGINX_SITE: &str = "default";

pub const GIT_HOOKS_DIR: &str = ".git/hooks";
pub const GIT_PRE_PUSH_HOOK: &str = ".git/hooks/pre-push";
pub const PRE_PUSH_HOOK_NAME: &str = "pre-push";
pub const PRE_PUSH_HOOK_TARGET: &str = "../../.bones/hooks/pre-push";
pub const HOOKS_DIR: &str = "hooks";
pub const KIT_HOOKS_DIR: &str = "hooks/";
pub const KIT_DEPLOYMENT_DIR: &str = "deployment/";
pub const KIT_SECRETS_DIR: &str = "secrets/";

const RUNTIME_SOCKET_PARENT: &str = "/run";

#[must_use]
pub fn default_repo_path_for(project_name: &str) -> String {
    Path::new(DEFAULT_REPO_PARENT).join(format!("{project_name}.git")).display().to_string()
}

#[must_use]
pub fn default_project_root_for(project_name: &str) -> String {
    Path::new(DEFAULT_PROJECT_ROOT_PARENT).join(project_name).display().to_string()
}

#[must_use]
pub fn default_web_root() -> String {
    DEFAULT_WEB_ROOT.to_string()
}

#[must_use]
pub fn ssl_certificate_path(domain: &str) -> String {
    Path::new(ETC_LETSENCRYPT_LIVE).join(domain).join("fullchain.pem").display().to_string()
}

#[must_use]
pub fn ssl_certificate_key_path(domain: &str) -> String {
    Path::new(ETC_LETSENCRYPT_LIVE).join(domain).join("privkey.pem").display().to_string()
}

#[must_use]
pub fn nginx_service_name(project_name: &str) -> String {
    format!("{project_name}-nginx.service")
}

#[must_use]
pub fn bonesremote_staging_path(version: &str) -> String {
    Path::new(TMP_ROOT).join(format!("{BONESREMOTE_BINARY}-{version}")).display().to_string()
}

#[must_use]
pub fn install_root() -> PathBuf {
    PathBuf::from(OPT_BONESDEPLOY)
}

#[must_use]
pub fn bonesremote_config_root() -> PathBuf {
    PathBuf::from(BONESREMOTE_CONFIG_DIR)
}

#[must_use]
pub fn bonesremote_sites_root() -> PathBuf {
    bonesremote_config_root().join(BONESREMOTE_SITES_DIR)
}

#[must_use]
pub fn bonesremote_site_root(site: &str) -> PathBuf {
    bonesremote_sites_root().join(site)
}

#[must_use]
pub fn bonesremote_registry_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(REGISTRY_TOML)
}

#[must_use]
pub fn bonesremote_bones_toml_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(BONES_TOML)
}

#[must_use]
pub fn bonesremote_staged_release_path(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(STAGED_RELEASE_FILE)
}

#[must_use]
pub fn bonesremote_tmp_builds_root(site: &str) -> PathBuf {
    bonesremote_site_root(site).join(TMP_BUILDS_DIR)
}

/// Canonical list of release-tree paths that should be linked from `shared/`.
pub const SHARED_LEAVES: &[&str] = &[DOT_ENV, "storage", "bootstrap/cache", "database/database.sqlite"];

#[must_use]
pub fn bonesremote_sites_root_resolved() -> PathBuf {
    if let Some(root) = env::var_os("BONESREMOTE_SITES_ROOT") {
        let raw = root.to_string_lossy().to_string();
        if !raw.trim().is_empty() {
            return PathBuf::from(raw);
        }
    }
    bonesremote_sites_root()
}

#[must_use]
pub fn install_versions_dir() -> PathBuf {
    install_root().join(INSTALL_VERSIONS_DIR)
}

#[must_use]
pub fn install_current_dir() -> PathBuf {
    install_root().join(INSTALL_CURRENT_LINK)
}

#[must_use]
pub fn bonesdeploy_global_link() -> PathBuf {
    Path::new(USR_LOCAL_BIN).join(BONESDEPLOY_BINARY)
}

#[must_use]
pub fn bonesremote_global_link() -> PathBuf {
    Path::new(USR_LOCAL_BIN).join(BONESREMOTE_BINARY)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Deployment {
    pub repo: String,
    pub repo_parent: String,
    pub repo_head: String,
    pub repo_bones: String,
    pub repo_bones_toml: String,
    pub repo_deployment: String,
    pub site_nginx_config: String,
    pub conf_root: String,
    pub project_root: String,
    pub project_root_parent: String,
    pub releases: String,
    pub shared: String,
    pub build_root: String,
    pub build_logs: String,
    pub current: String,
    pub current_web_root: String,
    pub placeholder_release: String,
    pub placeholder_web_root: String,
    pub placeholder_index: String,
    pub nginx_site_available: String,
    pub nginx_site_enabled: String,
    pub nginx_default_site_enabled: String,
    pub systemd_site_nginx_service: String,
    pub apparmor_profile_path: String,
    pub runtime_socket_dir: String,
    pub runtime_nginx_socket: String,
    pub runtime_nginx_pid: String,
    pub runtime_php_fpm_socket: String,
    pub sudoers_path: String,
    pub usr_local_bin: String,
    pub bonesremote_global_link: String,
    pub apparmor_enabled_param: String,
    pub apparmor_profiles: String,
}

impl Deployment {
    #[must_use]
    pub fn new(project_name: &str, repo_path: &str, project_root: &str, web_root: &str) -> Self {
        let repo = repo_path.to_string();
        let project_root = project_root.to_string();
        let placeholder_release = Path::new(&project_root).join(RELEASES_DIR).join(PLACEHOLDER_RELEASE_NAME);
        let current = Path::new(&project_root).join(CURRENT_LINK);
        let runtime_socket_dir = Path::new(RUNTIME_SOCKET_PARENT).join(project_name);
        let repo_bones = Path::new(&repo).join(BONES_DIR);
        let conf_root = Path::new(DEFAULT_CONF_ROOT_PARENT).join(project_name);

        Self {
            repo: repo.clone(),
            repo_parent: parent_or_default(&repo, DEFAULT_REPO_PARENT),
            repo_head: Path::new(&repo).join(GIT_HEAD).display().to_string(),
            repo_bones: repo_bones.display().to_string(),
            repo_bones_toml: repo_bones.join(BONES_TOML).display().to_string(),
            site_nginx_config: conf_root.join(NGINX_CONF).display().to_string(),
            repo_deployment: repo_bones.join(DEPLOYMENT_DIR).display().to_string(),
            conf_root: conf_root.display().to_string(),
            project_root: project_root.clone(),
            project_root_parent: parent_or_default(&project_root, DEFAULT_PROJECT_ROOT_PARENT),
            releases: Path::new(&project_root).join(RELEASES_DIR).display().to_string(),
            shared: Path::new(&project_root).join(SHARED_DIR).display().to_string(),
            build_root: Path::new(&project_root).join(BUILD_DIR).join(WORKSPACE_DIR).display().to_string(),
            build_logs: Path::new(&project_root).join(BUILD_DIR).join(LOGS_DIR).display().to_string(),
            current: current.display().to_string(),
            current_web_root: current.join(web_root).display().to_string(),
            placeholder_release: placeholder_release.display().to_string(),
            placeholder_web_root: placeholder_release.join(web_root).display().to_string(),
            placeholder_index: placeholder_release.join(web_root).join(INDEX_HTML).display().to_string(),
            nginx_site_available: Path::new(ETC_NGINX_SITES_AVAILABLE)
                .join(format!("{project_name}.conf"))
                .display()
                .to_string(),
            nginx_site_enabled: Path::new(ETC_NGINX_SITES_ENABLED)
                .join(format!("{project_name}.conf"))
                .display()
                .to_string(),
            nginx_default_site_enabled: Path::new(ETC_NGINX_SITES_ENABLED)
                .join(DEFAULT_NGINX_SITE)
                .display()
                .to_string(),
            systemd_site_nginx_service: Path::new(ETC_SYSTEMD_SYSTEM)
                .join(nginx_service_name(project_name))
                .display()
                .to_string(),
            apparmor_profile_path: Path::new(ETC_APPARMOR_D)
                .join(format!("bonesdeploy-{project_name}-nginx"))
                .display()
                .to_string(),
            runtime_socket_dir: runtime_socket_dir.display().to_string(),
            runtime_nginx_socket: runtime_socket_dir.join(NGINX_SOCKET).display().to_string(),
            runtime_nginx_pid: runtime_socket_dir.join(NGINX_PID).display().to_string(),
            runtime_php_fpm_socket: runtime_socket_dir.join(PHP_FPM_SOCKET).display().to_string(),
            sudoers_path: Path::new(ETC_SUDOERS_D).join(SUDOERS_FILE).display().to_string(),
            usr_local_bin: USR_LOCAL_BIN.to_string(),
            bonesremote_global_link: Path::new(USR_LOCAL_BIN).join(BONESREMOTE_BINARY).display().to_string(),
            apparmor_enabled_param: APPARMOR_ENABLED_PARAM.to_string(),
            apparmor_profiles: APPARMOR_PROFILES.to_string(),
        }
    }
}

fn parent_or_default(path: &str, fallback: &str) -> String {
    Path::new(path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map_or_else(|| fallback.to_string(), |parent| parent.display().to_string())
}

fn home_dir() -> PathBuf {
    env::var("HOME").map_or_else(|_| PathBuf::from("/root"), PathBuf::from)
}

#[must_use]
pub fn bones_config_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_CONFIG_HOME").ok().filter(|v| !v.is_empty()) {
        Path::new(&dir).join("bonesdeploy")
    } else {
        home_dir().join(".config/bonesdeploy")
    }
}

#[must_use]
pub fn bones_state_root() -> PathBuf {
    if let Some(dir) = env::var("XDG_STATE_HOME").ok().filter(|v| !v.is_empty()) {
        Path::new(&dir).join("bonesdeploy")
    } else {
        home_dir().join(".local/state/bonesdeploy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bones_config_root_uses_xdg_config_home() {
        let home = home_dir();
        let expected = home.join(".config/bonesdeploy");
        assert_eq!(bones_config_root(), expected);
    }

    #[test]
    fn bones_state_root_uses_xdg_state_home() {
        let home = home_dir();
        let expected = home.join(".local/state/bonesdeploy");
        assert_eq!(bones_state_root(), expected);
    }
}

```

`crates/shared/src/registry.rs`:

```rs
use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};

use crate::{config, paths};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Registry {
    pub site: String,
    pub repo_path: String,
    pub site_root: String,
    pub shared_root: String,
    pub releases_root: String,
    pub current_path: String,
    pub runtime_user: String,
    pub runtime_group: String,
    pub branch: String,
    pub deploy_on_push: bool,
    pub releases_keep: usize,
}

impl Registry {
    #[must_use]
    pub fn derive(bones: &config::Bones) -> Self {
        let site_root = bones.project_root.clone();
        Self {
            site: bones.project_name.clone(),
            repo_path: bones.repo_path.clone(),
            shared_root: format!("{site_root}/{}", paths::SHARED_DIR),
            releases_root: format!("{site_root}/{}", paths::RELEASES_DIR),
            current_path: format!("{site_root}/{}", paths::CURRENT_LINK),
            site_root,
            runtime_user: config::runtime_user_for(&bones.project_name),
            runtime_group: config::runtime_group_for(&bones.project_name),
            branch: bones.branch.clone(),
            deploy_on_push: bones.deploy_on_push,
            releases_keep: bones.releases_keep,
        }
    }

    #[must_use]
    pub fn deployment_paths(&self, web_root: &str) -> paths::Deployment {
        paths::Deployment::new(&self.site, &self.repo_path, &self.site_root, web_root)
    }
}

/// # Errors
///
/// Returns an error if the registry file cannot be read or parsed.
pub fn load(path: &Path) -> Result<Registry> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
    let registry: Registry = toml::from_str(&content).with_context(|| format!("Failed to parse {}", path.display()))?;
    validate_site_name(&registry.site)?;
    Ok(registry)
}

/// # Errors
///
/// Returns an error when the site name is empty or contains characters outside
/// ASCII lowercase letters, digits, and dashes.
pub fn validate_site_name(site: &str) -> Result<()> {
    if site.is_empty() {
        bail!("Site name cannot be empty");
    }

    if site.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-') {
        return Ok(());
    }

    bail!("Invalid site name: {site}")
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::{load, validate_site_name, Registry};
    use crate::config::Bones;

    #[test]
    fn validate_site_name_rejects_path_escapes() {
        assert!(validate_site_name("../evil").is_err());
        assert!(validate_site_name("evil/site").is_err());
        assert!(validate_site_name("good-site").is_ok());
    }

    #[test]
    fn derive_uses_conventional_remote_paths() {
        let bones = Bones {
            project_name: String::from("acme"),
            repo_path: String::from("/home/git/acme.git"),
            project_root: String::from("/srv/sites/acme"),
            branch: String::from("main"),
            deploy_on_push: true,
            releases_keep: 7,
            ..Default::default()
        };

        let registry = Registry::derive(&bones);
        assert_eq!(registry.shared_root, "/srv/sites/acme/shared");
        assert_eq!(registry.releases_root, "/srv/sites/acme/releases");
        assert_eq!(registry.current_path, "/srv/sites/acme/current");
        assert_eq!(registry.runtime_user, "acme");
        assert_eq!(registry.branch, "main");
        assert!(registry.deploy_on_push);
    }

    #[test]
    fn load_reads_registry_toml() -> Result<()> {
        let path = env::temp_dir().join(format!("bonesremote-registry-{}.toml", process::id()));
        fs::write(
            &path,
            r#"
site = "acme"
repo_path = "/home/git/acme.git"
site_root = "/srv/sites/acme"
shared_root = "/srv/sites/acme/shared"
releases_root = "/srv/sites/acme/releases"
current_path = "/srv/sites/acme/current"
runtime_user = "acme"
runtime_group = "acme"
branch = "main"
deploy_on_push = true
releases_keep = 5
"#,
        )?;

        let registry = load(&path)?;
        fs::remove_file(&path).ok();

        assert_eq!(registry.site, "acme");
        assert_eq!(registry.releases_root, "/srv/sites/acme/releases");
        Ok(())
    }
}

```

`docker/Dockerfile`:

```
FROM debian:stable

RUN apt-get update \
    && DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends \
        bash \
        git \
        rsync \
        sudo \
        openssh-server \
    && rm -rf /var/lib/apt/lists/*

```

`docker/docker-compose.yml`:

```yml
services:
  bonesdeploy-test-server:
    privileged: true
    build:
      context: .
    container_name: bonesdeploy-test-server
    ports:
      - "2222:22"
    working_dir: /workspace
    volumes:
      - ./:/workspace
      - ${HOME}/.ssh/id_rsa.pub:/tmp/host_key.pub:ro
    tmpfs:
      - /var/lib/bonesdeploy-test-server
    command: >
      /bin/bash -lc "
      mkdir -p /run/sshd /root/.ssh &&
      cat /tmp/host_key.pub > /root/.ssh/authorized_keys &&
      chmod 700 /root/.ssh &&
      chmod 600 /root/.ssh/authorized_keys &&
      /usr/sbin/sshd -D -e
      "

```

`docs/PROJECT.md`:

```md
# BonesDeploy v3

A remote release deployment tool for simple Linux servers. It produces two executables: `bonesdeploy` (local CLI for setup, provisioning, deployment, and management) and `bonesremote` (server-side release lifecycle executor, installed on the deployment host). Git remains supported as an optional trigger, but it does not own the deployment model. **We only handle Debian/Ubuntu machines.**

We keep detailed documentation of each command at: `docs/commands/*.md:`

## Deployment Methodology
We have an SSH deployment user (normally `git`) that handles deployment concerns. This user has a home folder, restricted sudo ability, but no password login. We also have a per-project service user named after the project. This is not a shared `applications` user; it must be a dedicated user per project so isolation works on a shared server. This user has no home folder, no login, and no sudo ability. This is ultimately who we want to own our project files to limit attack scope.

### Just-in-Time Concerns
This project should prefer just-in-time mutations.

A concern should only be handled at the last responsible moment: immediately before the system would fail if that mutation did not occur. We should not widen permissions, rewrite symlinks, mutate shared state, or otherwise touch live project state early just because a later step might need it. The idea here is to limit the surface of attack time, so that potential vulnerabilities are not created by "jumping the gun" to solve a problem too early, long before it arises.

This principle exists to keep deployment behavior coherent and safe:

- the pre-deploy steps (doctor, stage, checkout, wire) should validate and prepare isolated state, not mutate live state.
- build steps should operate on isolated workspace state whenever possible.
- activation concerns should happen at activation time.
- permission hardening should happen after a successful activation, not before.
- if a deploy fails, it should not leave behind broadened access or half-applied live mutations.

In practice, this means we should prefer:
- isolated staging over speculative live-state mutation
- narrow, local changes over recursive ownership changes
- exact, just-before-use fixes over broad upfront rewrites
- failure-safe sequencing over convenience

If a mutation can be delayed safely, it should be delayed.
If a mutation affects live state, it must be justified by an immediate need.

### Common Problems
- Shared groups have too many logic traps. My apps should not have 660 or 770 permissions on all files so that a `git` user can have read/write.
- I don't like ACLs; they're far too opaque.
- Setting up inotify systems are cumbersome.

### Permission Model

Permissions are a **provisioning-time contract**, not a deployment-time repair. The ownership layout is established once during `bonesdeploy remote setup` and never rewritten by deploy commands.

**Three identity classes:**

| Identity | Owner of | Scope |
|----------|----------|-------|
| `git` (deploy user) | Bare repo | Ingress only |
| `<site>` (runtime user) | Shared files, `/run/<site>`, writable paths | Mutates runtime state |
| `root` | System units, config dirs, users/groups, sealed releases | Provisions, deploys, and restarts services |

**Key mechanics:**

- `releases/` contains root-promoted artifacts sealed as `root:<site>`.
- `shared/` is owned by the runtime user (`<site>:<site>`) — only the app writes here.
- Build input is temporary and disposable; build scripts run in Podman with the source mounted at `/workspace/source`.
- Prepare scripts run as the runtime user after shared paths are wired and before `current` is repointed.
- Git hooks only trigger `bonesremote`; they do not check out source, run builds, write releases, or restart services.
- `bonesremote` is the privileged mediator for promotion, activation, and service restart.

## Bones Scaffolding
```

.bones
├── bones.toml
├── runtime.toml
├── hooks
│ ├── hooks.sh # (legacy; pre-push only used to source from here)
│ ├── post-receive
│ └── pre-push
├── deployment
│ ├── 01_install_build_deps.sh
│ └── 02_run_build.sh

````

Python infra scripts and templates are managed separately by the hidden `bonesinfra` checkout; see `crates/bonesdeploy/src/infra/bonesinfra.rs`.

### Bones TOML
This stores crucial data we will need and is collected on running `bonesdeploy init` via user prompts.
Collects the following project information from the user:
- `project_name`: str
- `branch`: str
- `remote_name`: existing remote selection when available, otherwise prompted; defaults to `production`. Must point to a fresh VPS, not a code host like GitHub.
- `host`: prompted when not inferable from selected remote
- `port`: defaults to `22`, prompt shown when remote inference is unavailable
- `repo_path`: inferred from selected remote URL when possible, else defaults to `/home/git/{project_name}.git`

Everything else is defaulted or derived for Debian/Ubuntu-first usability:
- `project_root`: defaults to `/srv/sites/{project_name}`
- `ssh_user`: defaults to `root`
- `deploy_on_push`: defaults to `false`
- `releases`: defaults to `5`

`web_root`, `runtime_user`, `runtime_group`, and `release_group` live in `.bones/runtime.toml`. Those identity values default to `{project_name}`, `{project_name}`, and `{project_name}-release` respectively.

Users can override any default by editing `.bones/bones.toml` after init.

Example `bones.toml`:
```toml
remote_name = "production"
project_name = "lawsnipe"
ssh_user = "root"
host = "deploy.example.com"
port = "22"
repo_path = "/home/git/lawsnipe.git"
project_root = "/srv/sites/lawsnipe"
branch = "master"
preview_domain = "lawsnipe-deploy-example-com.nip.io"
domain = "app.example.com"
email = "ops@example.com"
deploy_on_push = false
ssl_enabled = true
releases = 5
````

### Hooks

Hooks are static shell scripts embedded in the `bonesdeploy` binary. They are written to `.bones/hooks/` once during `bonesdeploy init`: a local `pre-push` guard and a remote `post-receive` thin trigger. The previous shared `hooks.sh` library is gone; `pre-push` is now self-contained and `post-receive` delegates directly to `sudo bonesremote hook post-receive --site <project>`. After that, they belong to the user and can be edited freely. They are published into `bonesremote`'s root-owned remote site state via `bonesdeploy push` and can be restored locally with `bonesdeploy pull`.

- `pre-push` => Local hook, symlinked to `.git/hooks/pre-push`. This checks to see if we are pushing to our bonesdeploy designated remote. If so, then we run `bonesdeploy doctor --local` and we fail if the doctor command expresses any warning or errors.
- `post-receive` => Thin trigger that derives `<site>` from `GIT_DIR` and runs `sudo bonesremote hook post-receive --site <site>`. `bonesremote` then reads branch policy and config from `/root/.config/bonesremote/sites/<site>/` instead of the bare repo.

### Deployment Folder

This folder stores build and prepare scripts that are published into bonesremote site state. Build scripts live in `.bones/deployment/build/`, must be ordered sequentially like `01_install_deps.sh`, `02_run_build.sh`, and run inside the `build_image` from `.bones/runtime.toml` with `cwd=/workspace/source`. The build container receives the exported source tree only; it does not receive `.env`, `shared/`, `current`, `releases/`, the bare repo, or bonesremote control-plane files. Prepare scripts live in `.bones/deployment/prepare/`, run in lexical order as the site runtime user with `cwd` set to the sealed release, and are the right place for migrations, cache warmups, and other runtime-state work.

## Crate Structure

This Cargo workspace has three crates under `crates/`:

- `bonesdeploy` for the local CLI binary
- `bonesremote` for the server-side binary
- `shared` for code that must be common to both binaries

### Path Centralization

All product-owned paths must live in `crates/shared/src/paths.rs`.

Other modules may derive subpaths by joining values from `shared::paths`, but they must not introduce their own independent path roots, filenames, or install locations.

This applies to Rust code, bonesinfra's internal operations/templates, and docs examples that describe the system layout.

```
bonesdeploy/
├── Cargo.toml                  # workspace root
├── crates/
│   ├── bonesdeploy/
│   │   ├── kit/                # embedded scaffolding templates and hooks
│   │   └── src/
│   │       ├── cli/            # clap args + dispatch
│   │       ├── commands/       # CLI command implementations
│   │       ├── infra/          # ssh, git, embedded assets, bonesinfra wrapper
│   │       ├── ui/             # prompt helpers
│   │       ├── config.rs
│   │       └── main.rs
│   ├── bonesremote/
│   │   └── src/
│   │       ├── cli/            # clap args + dispatch
│   │       ├── commands/       # remote release lifecycle steps
│   │       ├── config.rs
│   │       ├── privileges.rs   # sudoers drop-in install + privilege checks
│   │       ├── release/
│   │       ├── release_state.rs
│   │       └── main.rs
│   └── shared/                 # config schema + central paths
└── docs/
```

### Per-Framework Templates

Runtime templates ship starter overlays that `bonesdeploy remote runtime` uses when scaffolding infrastructure for a matching framework. Each template lives in the `bonesinfra` repo (`https://github.com/AlextheYounga/bonesinfra.git`) — framework runtime assets (`operations.py`, Jinja2 templates) stay together:

- `runtimes/laravel/` → Laravel (PHP + PHP-FPM)
- `runtimes/django/` → Django (Python + Gunicorn)
- `runtimes/next/` → Next.js (Node)
- `runtimes/nuxt/` → Nuxt (Node)
- `runtimes/sveltekit/` → SvelteKit (Node)
- `runtimes/vue/` → Vue (Node)
- `runtimes/rails/` → Rails (Ruby)

Templates inherit the same `bones.toml` schema and customize permissions paths, deployment scripts, and the runtime operations captured in the `bonesinfra` repo.

### BonesDeploy CLI Commands

- **init**:

  - Gets or creates the `.bones` folder with our default scaffolding.
  - Updates `.gitignore` to add .bones folder.
  - Loads existing config from `.bones/bones.toml` or collects user input via prompts.
  - Creates local deployment remote if missing using `{deploy_user}@{host}:{repo_path}`, constructed from the production VPS target configured during prompts.
  - Prints next-step guidance to run `bonesdeploy remote setup` and `bonesdeploy remote runtime` before first deploy.
  - Saves config to `.bones/bones.toml`.

- **doctor**

  - This command checks all concerns in your local environment.
  - Loads config from `.bones/bones.toml`
  - Runs local checks:
    - `.bones` folder exists and is a symlink (warns if it is not a symlink to `~/.config/bonesdeploy/<project>.bones/`). Deployment scripts are named appropriately.
    - Required files exist: `bones.toml`, `hooks/pre-push`, `hooks/post-receive`, `deployment/` dir.
    - Local `pre-push` hook is symlinked properly when `deploy_on_push = true`.
  - Runs remote checks (skipped with `--local`):
    - `bonesremote` executable exists on remote and can be run globally.
    - `{project_name}.git/bones` folder exists on remote (needs `bonesdeploy push` warning)
    - `{project_name}.git/bones/hooks` matches with `{project_name}.git/hooks` inside the remote bare repo when `deploy_on_push = true`.
  - The `--local` flag skips all remote checks. The `pre-push` hook uses this flag because it is only a local guard before optional git-triggered deploys.

- **push**

  - Archives the local `.bones/` dataset, excluding local secrets, and streams it to `bonesremote site import --site <project>` over SSH.
  - `bonesremote` validates the dataset, derives a root-owned `registry.toml`, and atomically replaces the current remote site state under `/root/.config/bonesremote/sites/<project>/`.
  - The bare repo is no longer the control-plane storage target for `push`.

- **pull**

  - Streams the current remote site dataset back from `bonesremote site export --site <project>` and extracts it into local `.bones/`.
  - Recreates the local `.git/hooks/pre-push` symlink so the repository regains its pre-push check after recovery.

- **deploy**

  - Publishes the local `.bones/` dataset into remote bonesremote site state first, then SSHes into the configured host and runs `bonesremote deploy --site <project>` directly.
  - Omits the `--revision` flag, so `bonesremote deploy` uses the configured branch from `bones.toml`.

- ****remote setup****

  - Delegates to the hidden `bonesinfra` checkout by running `python -m bonesinfra setup apply --config <path>` against the configured host as root (or `BONES_BOOTSTRAP_SSH_USER`).
  - Passes `bones.toml` deployment values plus computed paths and variables as JSON on stdin.
  - Initializes bare git repository at `repo_path`.
  - Creates initial placeholder release with default page.
  - Installs `bonesremote` from source and installs the validated `/etc/sudoers.d/bonesdeploy` drop-in.
  - Provisions machine-level dependencies (users, groups, firewall, system packages).

- **remote runtime**:

  - Prompts for a framework template, refreshes `.bones/runtime/`, and writes `.bones/runtime.toml`.
  - Reapplies template-specific defaults into `.bones/bones.toml` only when they still match generic or previous-template values.
  - After a `y/N` confirmation, delegates to the hidden `bonesinfra` checkout by running `python -m bonesinfra runtime apply --config <path> --runtime-config <path>` against the configured host as the configured `ssh_user`.
  - Loads the template's `operations.py` at runtime to install framework-specific packages and services.
  - Configures per-site runtime assets: AppArmor profile, nginx router + per-site config + systemd service, and runs `bonesremote doctor`.
  - Does not handle SSL; use `remote ssl` for TLS configuration.

- **remote ssl**

  - Delegates to the hidden `bonesinfra` checkout by running `python -m bonesinfra ssl apply --config <path>` against the configured host as root.
  - Uses certbot with a webroot challenge to obtain/renew certificates for the configured domain.
  - Re-renders the per-site runtime nginx router with TLS enabled, listening on 443 and redirecting HTTP to HTTPS.
  - Separate from `remote runtime` to keep certificate management decoupled from app runtime concerns.

- **rollback**

  - SSHes into the configured host and runs `bonesremote release rollback --site <project>`, which repoints `current` to the previous release without rebuilding and restarts `<project>-nginx.service`.

- **secrets**

  - Subcommands: `init`, `edit`, `push`.
  - Manages GPG-encrypted environment secrets under `.bones/secrets/`.
  - `secrets init` bootstraps the `.bones/secrets/` directory and GPG recipients.
  - `secrets edit` decrypts `.bones/secrets/.env.gpg` for editing and re-encrypts on save.
  - `secrets push` uploads the decrypted `.env` to the remote `shared/.env` over SSH.

- **config**

  - Reads or prints values from `.bones/bones.toml`.
  - `--file <path>` overrides the config file location.
  - `<key>` prints a single value when supplied; otherwise dumps the whole file.

- **manage**

  - Opens an interactive SSH session to the remote and runs `bonesremote manage --config <path>`. Requires `bonesremote manage` to be implemented on the server.

- **version**:

  - Echoes the installed `bonesdeploy` version.

### BonesRemote CLI Commands

- **Release commands** live under `bonesremote release ...`
- **Service commands** live under `bonesremote service ...`
- **deploy**:
  - Runs the full deployment lifecycle as a single command (the primary entrypoint used by both `post-receive` hook and `bonesdeploy deploy`).
  - Orchestrates: stage release → source export from the bare repo into a temp build context → build scripts → release promotion/hardening → shared wiring → activate → service restart → post-deploy pruning.
  - On failure, automatically drops the staged release.
  - `--site <name>`: imported site identifier used to load root-owned registry state
  - `--revision <rev>`: optional exact commit to check out; defaults to configured branch
- **init**:
  - Must be run as sudo.
  - Installs a drop-in file at `/etc/sudoers.d/bonesdeploy` to allow only `sudo bonesremote service restart --config *` without requiring password.
- **doctor**:
  - Checks to see if the server is set up properly:
    - `bonesremote` is globally available.
    - AppArmor support is available on the host.
    - Sudoers drop-in is correctly configured.
- **release stage**
  - Creates a staged release tree under `releases/`, ensures `build/workspace` and `shared/`, then writes staged release state before checkout.
- **release wire**
  - Wires shared paths into `build/workspace` after checkout, replacing any existing build workspace paths with symlinks to the shared directory.
- **release activate**
  - Atomically switches `current` to the staged release and clears staged release state.
- **release drop-failed**
  - Deletes a failed staged release and clears staged release state.
- **release rollback**
  - Repoints `current` to the previous release.
- **service restart**
  - Restarts the per-site nginx systemd service (`<project>-nginx.service`). This is the only `bonesremote` command that requires root privileges.
- **version**:
  - Echoes the installed `bonesremote` version.

## Security Notes

- Sudo access for the deployment user is strictly limited to passwordless execution of the narrow allowed `bonesremote` subcommands via the `/etc/sudoers.d/bonesdeploy` drop-in installed by `bonesinfra`.
- No broader sudo privileges are granted — the deploy user cannot run arbitrary commands as root, read root-owned files, or write outside their owned directories.
- All release artifacts are created with the setgid bit on `releases/` so the runtime group inherits read access without needing a post-deploy chown.
- The build workspace (`build/`) is private to the deploy user (`0700`), invisible to other processes.
- Runtime processes are sandboxed via systemd `ProtectSystem=strict`, `NoNewPrivileges=yes`, `PrivateTmp=yes`, and AppArmor profiles — limiting blast radius even if a service is compromised.
- Per-project systemd services run as the dedicated runtime user, not a shared `www-data` — so service isolation is enforced at the OS level, not just the application level.

## Flow

- User runs `bonesdeploy init`, and the procedures outlined above are executed.
- User can make any changes to their deployment scripts or hooks in `.bones/` (e.g., customizing `deployment/build/` files or adding project-specific logic).
- User runs `bonesdeploy push` to publish the `.bones/` dataset to bonesremote site state under `/root/.config/bonesremote/sites/<site>/`.
- User runs `bonesdeploy deploy` to perform the actual remote release deployment.

### Primary Deploy Flow

1. `bonesdeploy deploy` publishes local `.bones/` state, then SSHes into the configured host.
2. It runs `bonesremote deploy --site <site>`.
3. `bonesremote deploy` orchestrates the full pipeline:
   - **stage_release** — Create timestamped release state
   - **release_checkout** — Export the approved revision with `git archive` into a temporary source tree
   - **release_build** — Run `deployment/build/*.sh` in disposable Podman containers at `/workspace/source`
   - **release_promote** — Copy safe artifacts into a sealed `root:<site>` release
   - **wire_shared** — Symlink declared shared paths into the sealed release
   - **release_prepare** — Run `deployment/prepare/*.sh` as the site runtime user in the sealed release
   - **activate_release** — Atomically repoint `current`
   - **restart_services** — Restart `<site>-nginx.service`
   - **post_deploy** — Prune old releases beyond `releases`
   - On failure: **drop_failed_release** — Clean up staged release

### Hook Event Order on `git push`

`pre-push -> post-receive`

1. **pre-push** (local): Runs `bonesdeploy doctor --local` if pushing to the configured bones remote and `deploy_on_push = true`. Aborts on warnings or errors.
2. Git updates refs in the bare repository.
3. **post-receive** (remote): Resolves the configured deployment ref from stdin:
   - If `deploy_on_push = false`, exits early without deploying.
   - If the configured branch wasn't pushed, or the push deleted it, exits without deploying.
   - Otherwise runs a single unified command:
     ```
     bonesremote deploy --site <site> --revision <newrev>
     ```
   - This command orchestrates the full pipeline:
     - **stage_release** — Create timestamped release state
     - **release_checkout** — Export source from the bare repo into temporary context
     - **release_build** — Run `deployment/build/*.sh` in Podman at `/workspace/source`
     - **release_promote** — Seal safe artifacts into `releases/<release>`
     - **wire_shared** — Link shared runtime paths
     - **release_prepare** — Run `deployment/prepare/*.sh` as the site runtime user
     - **activate_release** — Repoint `current`
     - **restart_services** — Restart `<site>-nginx.service`
     - **post_deploy** — Prune old releases beyond `releases`
     - On failure: **drop_failed_release** — Clean up staged release

`bonesdeploy deploy` performs the same remote pipeline by SSHing into the host and running `bonesremote deploy --site <site>` directly (without `--revision`, so it uses the configured branch). Git-triggered deploy is optional plumbing, not the primary model.

````

`docs/thinking/01_security_architecture_problems.md`:

```md
Compression is rejecting ID order. I'll proceed directly to the synthesis — context is still manageable.

---

# Synthesis: BonesDeploy Ownership Model Redesign

I audited both repos end-to-end. Here's the grounded picture, the contradictions, and a proposed target model with explicit tradeoffs. **No final decisions baked in** — I flag every fork where I need your call.

---

## 1. What the code actually does today (the one-paragraph version)

There is **no `foo-build` user anywhere**. The global `git` user is the single deploy-side identity for every site: it owns the bare repo, owns `/srv/sites/<project>` and `releases/` and `build/`, runs `git checkout` into `build/workspace`, runs composer/npm/vite as itself with its own HOME as the cache root, runs `cp -a` into `releases/<id>` (preserving `git` ownership), flips `current`, prunes old releases, and drops failed ones. The runtime user (`<project>`, a system user with `nologin`) is enrolled in both `runtime_group` and `release_group`; `git` is **also** enrolled in `runtime_group` (`users.py:47`). `shared/` is `runtime_user:runtime_group` mode **2775** (group-writable + world-readable). The only sudoers grant is `git ALL=(root) NOPASSWD: <bonesremote> service restart --config *` (`init.rs:20-29`), and that command trusts git-supplied `project_name` with only a regex check. There is **no root-owned site registry** — `bones.toml`/`runtime.toml` live inside the git-owned bare repo and are trusted for privileged decisions (AppArmor grants read on `repo_bones_toml` at `project-nginx-profile.j2:32`; `service.rs` trusts `project_name` for `systemctl restart <x>-nginx`).

---

## 2. Violations against your desired model (ranked by severity)

### Severity: Critical (trust boundary / cross-site)

**V1. No `foo-build` user exists.** Build/release ownership is collapsed into the global `git`. One compromised `git` = every site's release code across the host.
- `crates/shared/src/paths.rs:11` `DEPLOY_USER = "git"`; `crates/shared/src/config.rs:91-103` has `runtime_user_for`/`runtime_group_for`/`release_group_for` but **no `build_user_for`**.
- `bonesinfra/.../setup/users.py:26-33` creates only `git` + runtime user.
- `bonesinfra/.../setup/directories.py:43-65` provisions `project_root`, `releases/`, `build/` all as `git:`.

**V2. `git` is in every site's `runtime_group`.** `users.py:47`. Combined with `shared/` mode 2775 (`directories.py:67-73`), `git` can read/write runtime mutable state (`.env`, sqlite, uploads) on every site. Tested-as-intended at `tests/test_deploy_structure.py:91-93` and `tests/test_shared_paths.py:33-35`.

**V3. No root-owned site registry; privileged code trusts git-owned config.**
- `bonesinfra/.../apparmor/project-nginx-profile.j2:32` grants runtime nginx read on `/home/git/<project>.git/bones/bones.toml`.
- `bonesinfra/.../domain/context.py:22-66` `DeployContext.from_files()` reads `project_name`/`repo_path`/`project_root`/`runtime_user`/`runtime_group`/`release_group` from git-owned TOML and those flow into root-owned systemd units, AppArmor profiles, nginx configs.
- `crates/bonesremote/src/commands/service.rs:12-26` trusts `project_name` from `--config *` (regex-only validation) → compromised `git` can restart any `<x>-nginx.service` on the host.
- `crates/bonesdeploy/src/commands/secrets.rs:120-143` trusts `runtime_group` from git-owned `runtime.toml` for `chown root:<group>` on secret files.

**V4. `runtime_group` from git-owned `runtime.toml` is trusted for `chown root:<group>`** in secrets push. A compromised `git` could set `runtime_group = "sudo"` and make secret files readable by a privileged group. (`secrets.rs:141-143`, `config.rs:151-164`.)

### Severity: High (release code ownership)

**V5. Entire deploy pipeline runs as `git`.** No `sudo -u foo-build` anywhere. `kit/hooks/hooks.sh:81,86` invokes `bonesremote deploy` directly; `deploy.rs:23-45` runs doctor→stage→post_receive→wire→publish→activate→restart→post_deploy all as `git`.
- `stage_release.rs:32-33` creates `releases/<id>` as `git`.
- `post_receive.rs:19-30` runs `git checkout` as `git` into `git:git 0700` build/workspace.
- `deploy.rs:74-85` runs build scripts (composer/npm/vite) as `git` with `git`'s HOME.
- `deploy.rs:116-134` `publish_release_tree` uses `cp -a`, preserving `git` ownership into the release tree.

**V6. Runtime user writes into release tree (not shared).**
- Rails (`bonesinfra/.../runtimes/rails/rails.py:38-42`): `runtime_write_paths = [current/tmp, current/log, current/storage]` → systemd `ReadWritePaths` + AppArmor `rwk` grant inside the release tree.
- Django (`django.py:45-47`): `current/staticfiles`, `current/media`.
- Laravel `runtime.toml`: `bootstrap/cache`, `database/database.sqlite` mode 770/660 inside the release tree (only `storage` and `.env` are shared via `[shared]`).
This contradicts your invariant "foo-run should not own release code... runtime compromise not to persist by rewriting code."

### Severity: Medium (docs/functional)

**V7. `wire_release` wires ONLY `.env`.** `wire_release.rs:18-48`. The test `storage_is_not_symlinked` (`wire_release.rs:240-263`) explicitly asserts `storage` is NOT symlinked. The `[shared] paths` block in Laravel's `runtime.toml` declares `storage` as shared, but the wire code doesn't implement it. → Laravel apps cannot write to `storage/` at runtime; either they crash or an operator hand-fixes permissions (exactly the recursive chown you want to eliminate).

**V8. `wire_release` `.env.example` fallback creates `shared/.env` as `git:git 0600`** → runtime_user cannot read it. (`wire_release.rs:27-34`.) Only the `secrets push` path does the correct `chown root:<runtime_group> 0640`.

**V9. Docs contradict code on modes and "activation hardening".**
- `docs/security/02-filesystem-layout.md:70` claims releases become "service-owned after activation hardening" — **no such step exists** (`activate_release.rs:9-30` only flips the symlink).
- `:84-90` claims modes `0750/0711`; actual modes are `2751/0700/2750/2775`.
- `:129-132` claims `storage` is wired as a shared symlink — it is not.
- `docs/PROJECT.md:52` claims `shared/` is `0711`; actual is `2775`.
- `docs/security/03-identity-and-user-policy.md:6` and `22-desired-end-state-summary.md:10` describe a 2-identity model (deploy + service), not your desired 3-identity (git + build + run).

**V10. `git` has `/bin/bash` + HOME + SSH authorized_keys** (`users.py:30-31, 74-90`). Desired: ingress-only (git-shell or nologin, no interactive shell).

**V11. Bare repo lives in `git`'s HOME** (`paths.py:4` `DEFAULT_REPO_PARENT = "/home/git"`). `git`'s HOME is simultaneously: bare-repo parent, build-tool cache root (`$HOME/.nvm`, `$HOME/.npm`), and SSH authorized_keys location.

**V12. Build tool caches are global, not per-site.** `01_install_build_deps.sh`/`02_run_build.sh` (quoted at `docs/references/bonesdeploy.md:793-853`) use `NVM_DIR="${NVM_DIR:-$HOME/.nvm}"`, `npm install -g`, `corepack enable` — all writing into `/home/git/.nvm`, `/home/git/.npm`, shared across every site.

**V13. `www-data` is a de facto global cross-site read key.** Every `/run/<project>/` and `/run/<project>/nginx/` is `0711` so system nginx can traverse; per-site nginx socket is `0666`-ish (`site-nginx.conf.j2:21-24` ponytail comment acknowledges this). Accepted for single-tenant hosts but violates multi-site isolation.

---

## 3. Proposed target ownership model

### Roles (per site `foo`)

| Role | Identity | Created by | Shell | HOME | sudo | Groups |
|---|---|---|---|---|---|---|
| Ingress | `git` (global, one per host) | bonesinfra | `git-shell` | `/home/git` (or `/srv/git` — see Q5) | none | none per-site |
| Build | `foo-build` (per-site) | bonesinfra | `/usr/sbin/nologin` | `/var/lib/bonesdeploy/build/foo` | none | `foo-release` |
| Runtime | `foo-run` (per-site, system user) | bonesinfra | `/usr/sbin/nologin` | `/nonexistent` | none | `foo-run` (primary), `foo-release` (supplementary, read) |
| Mediator | `root` / `bonesremote` | — | — | — | — | — |

### Directory ownership target

| Path | owner:group:mode | Notes |
|---|---|---|
| `/home/git` (or `/srv/git`) | `git:git 0750` | Ingress home; no build caches here |
| `/home/git/foo.git` (bare repo) | `git:git` | Ingress repo; git-shell push only |
| `/srv/sites` | `root:root 0711` | Traverse-only (unchanged) |
| `/srv/sites/foo` | `foo-build:foo-release 2751` | Setgid so children inherit `foo-release` |
| `/srv/sites/foo/releases` | `foo-build:foo-release 2750` | Setgid; `foo-build` owns entries, `foo-run` reads via group |
| `/srv/sites/foo/build` | `foo-build:foo-build 0700` | Private build workspace |
| `/srv/sites/foo/shared` | `foo-run:foo-run 0750` | **Runtime-exclusive; `git` NOT in group** |
| `/srv/sites/foo/current` | `root:root` (symlink) | Flipped only by root/bonesremote |
| `/run/foo` | `foo-run:foo-run 0711` | Unchanged (www-data traverse) |
| `/var/lib/bonesdeploy/build/foo` | `foo-build:foo-build 0750` | HOME + composer/npm/cargo caches |
| `/etc/bonesdeploy/sites/foo.toml` | `root:root 0640` | **NEW: root-owned site registry** |
| `/srv/conf/foo` | `root:foo-run 0750` | nginx config, runtime.env |

### Trust boundary

- **Git-owned** (untrusted for privileged decisions): bare repo, `bones/bones.toml`, `runtime.toml` inside the repo. These drive **non-privileged deploy orchestration only** (stage, checkout, build scripts, wire, publish, prune).
- **Root-owned** (trusted for privileged decisions): `/etc/bonesdeploy/sites/foo.toml` — canonical `project_name`/`repo_path`/`project_root`/`build_user`/`runtime_user`/`runtime_group`/`release_group`/`service_name`. Written once by `bonesremote init` (root); `bonesinfra` reads it and refuses to provision if the git-owned `bones.toml` values conflict.
- **Privileged operations** (`service restart`, `current` flip if we move it, any future `release publish` if we go that route) read identity from the registry, not from `--config`.

### Runtime-writable paths (the shared-symlink model)

Every runtime-mutable path lives under `shared/` and is symlinked into the release at wire time. `runtime_write_paths` in systemd/AppArmor reference **`shared/...` only, never `current/...`**.

| Framework | shared paths | release symlink targets |
|---|---|---|
| Laravel | `.env`, `storage/`, `bootstrap/cache/`, `database/database.sqlite` | `release/.env`, `release/storage`, `release/bootstrap/cache`, `release/database/database.sqlite` |
| Rails | `.env`, `storage/`, `tmp/`, `log/` | `release/.env`, `release/storage`, `release/tmp`, `release/log` |
| Django | `.env`, `media/`, `staticfiles/` (if runtime-generated) | `release/.env`, `release/media`, `release/staticfiles` |
| Next/Nuxt/SvelteKit/Vue | `.env` only | `release/.env` |

`wire_release` creates these symlinks as `foo-build` (so the symlink entries are build-owned, pointing at foo-run-owned shared leaves). The shared leaves themselves are created as `foo-run:foo-run 0750` — either by `bonesinfra` at provisioning or by a root-mediated `bonesremote shared init` step.

---

## 4. Open design questions (need your call)

### Q1. Keep a single global `git` user as ingress?

**My recommendation: yes, but downgrade to `git-shell`.** One global ingress user is operationally simpler and the blast radius is contained if (a) `git` is removed from all runtime groups, (b) `git`'s shell is `git-shell` (no interactive commands, only `git-receive-pack`/`git-upload-pack`), (c) the post-receive hook invokes `bonesremote` via a narrow sudoers rule rather than `git`'s own shell.

**Tradeoff:** if you want per-site ingress isolation (separate SSH keys per site, separate ingress users), that's `foo-ingress` per site — more users to manage, but a compromised key for site A cannot push to site B. For a single-admin-host model, global `git` + `git-shell` is the lazy-correct choice.

### Q2. Per-site `foo-build` user?

**My recommendation: yes, this is the core of the redesign.** Without it, V1/V5/V12 cannot be fixed. `foo-build` owns `build/`, `releases/`, runs checkout + build scripts, has its own HOME for tool caches.

**Tradeoff:** +1 system user per site. For N sites that's N+3 users (git + foo-build + foo-run + foo-release group) vs the current N+2 (git + foo + foo-release). The isolation gain (per-site build identity, per-site cache isolation, no global `git` owning all release code) is worth it.

### Q3. Per-site `foo-run` user?

**Already the case** (runtime_user defaults to project_name). Keep. Just rename conceptually to `foo-run` in docs and make `runtime_group` distinct from `runtime_user` (currently both default to `project_name`, so the runtime user's primary group == runtime_group, and adding `git` to runtime_group bleeds everywhere).

**My recommendation:** `runtime_user = "foo-run"`, `runtime_group = "foo-run"` (same name, but explicitly a group), `release_group = "foo-release"`. Drop `git` from `runtime_group` (V2 fix).

### Q4. `foo-release` group composition?

**My recommendation:** `foo-release` contains `foo-build` (primary or supplementary) and `foo-run` (supplementary, read-only intent). `git` is **not** a member. `foo-build` is the owner of release dirs/files; `foo-run` reads via group. Mode `2750` on `releases/` (setgid) ensures new release subdirs inherit `foo-release` as group.

### Q5. Bare repo location: `/home/git` vs `/srv/git` vs `/var/lib/bonesdeploy/repos`?

**My recommendation: `/srv/git/<project>.git`, owned `git:git 0750`.** Rationale: keeps `git`'s HOME (`/home/git`) clean for SSH/authorized_keys only, separates bare repos from HOME clutter, and `/srv` is the conventional location for site-served data. `git` still owns the repos (ingress). `foo-build` does **not** have direct repo access — see Q6.

**Tradeoff:** migrating existing repos from `/home/git/<project>.git` to `/srv/git/<project>.git` is a one-time `mv` + `bones.toml` `repo_path` update + remote URL update on dev machines. Low cost.

**Alternative (lazy):** keep `/home/git/<project>.git` but set `git`'s shell to `git-shell` and move build caches to `foo-build`'s HOME. Cheaper migration, but `git`'s HOME still mixes SSH identity with repo storage.

### Q6. Does `foo-build` get direct read access to the bare repo, or does `bonesremote` export a source archive?

**My recommendation: archive export via root mediator (your option A).** The post-receive hook (running as `git`) calls `sudo bonesremote release checkout --config <registry>` which as root:
1. Validates the repo/revision against the root-owned registry.
2. `git --git-dir=/srv/git/foo.git archive <rev> | sudo -u foo-build tar -x -C /srv/sites/foo/build/workspace`

This gives `foo-build` no direct access to `/srv/git/foo.git`, no traversal into `git`'s space, and the privileged step validates the rev before exporting.

**Tradeoff (option B):** give `foo-build` read-only group access to exactly one bare repo via a `foo-repo-read` group. Simpler (no archive step) but `foo-build` can traverse into `/srv/git` and read the full repo history (including `.git` metadata). Option A is tighter.

**Tradeoff (option C):** move repos under per-site paths like `/srv/sites/foo/repo.git` owned `foo-build:foo-build`. Then `foo-build` owns ingress too — but this collapses ingress and build identities for that site, weakening the isolation you want between push-access and build-access.

### Q7. Should `bonesremote` run as root and drop privileges to `foo-build` for checkout/build?

**My recommendation: yes, for the privileged boundary steps only.** `bonesremote` runs as root (via sudo from `git`'s post-receive hook), reads identity from `/etc/bonesdeploy/sites/foo.toml`, validates, then `setuid`/`sudo -u foo-build` for the actual checkout + build script execution. Build scripts run with `foo-build`'s HOME, no sudo, AppArmor-confined (future).

**Tradeoff:** this adds a privileged `release checkout` and `release build` command to `bonesremote`. Simpler alternative: keep the post-receive hook as `git`, but have it invoke `sudo -u foo-build bonesremote deploy --config <registry>` for the build steps. The hook itself stays unprivileged; only the `sudo -u` crosses the boundary. Narrower sudoers: `git ALL=(foo-build) NOPASSWD: /usr/local/bin/bonesremote deploy --config /etc/bonesdeploy/sites/*`.

### Q8. Build scripts run as `foo-build`?

**Yes, non-negotiable for the model.** `deploy.rs:74-85` currently runs `build_workspace/deployment/*.sh` as `git`. Change to run as `foo-build` (via the sudoers rule in Q7 or the setuid in Q7). HOME must be `/var/lib/bonesdeploy/build/foo` so composer/npm/cargo caches are per-site.

### Q9. How do build tools get HOME/cache without login?

**My recommendation:** `foo-build` is a system user with `home=/var/lib/bonesdeploy/build/foo`, `shell=/usr/sbin/nologin`, `create_home=True`. `bonesinfra` provisions `$HOME/.composer`, `$HOME/.npm`, `$HOME/.cargo`, `$HOME/.cache` as `foo-build:foo-build 0750`. Build scripts invoked via `sudo -u foo-build` set `HOME=/var/lib/bonesdeploy/build/foo`.

### Q10. Which paths are code artifacts vs runtime mutable?

See the table in §3. **Code artifacts** (build-owned, read-only at runtime): everything except the shared-symlink targets. **Runtime mutable** (foo-run-owned, under `shared/`): `.env`, `storage/`, `bootstrap/cache/`, `database/database.sqlite`, `tmp/`, `log/`, `media/`, `uploads/`, `staticfiles/` (if runtime-generated).

### Q11. Should `.env` be readable by `foo-build` during build?

**My recommendation: no, with a carve-out for Laravel.** `.env` lives at `shared/.env` owned `root:foo-run 0640` (via secrets push). `foo-build` cannot read it. For Laravel `php artisan config:cache` which needs `.env` at build time, run that specific command as `foo-run` via a narrow `bonesremote release warm --config <registry>` step, or accept that `config:cache` runs at first-request runtime (slower first boot, no secret leak to build).

**Tradeoff:** allowing `foo-build` to read `.env` simplifies Laravel builds but leaks secrets to the build identity (which has no login but could be compromised via the build scripts). The lazy-secure default: don't let build read secrets; run secret-dependent warm-up as `foo-run`.

### Q12. `bootstrap/cache` shared or per-release?

**My recommendation: shared.** Laravel's `bootstrap/cache` (compiled config, routes, views, events) should persist across releases for fast boot. Symlink `shared/bootstrap/cache` → `release/bootstrap/cache`, owned `foo-run:foo-run 0750`. Rebuilt by `php artisan config:cache` etc. running as `foo-run` (per Q11) on each deploy.

### Q13. How does `current` symlink activation work?

**My recommendation: root/bonesremote flips it.** `current` is `root:root` (already the case post-provisioning, `placeholder.py:18-23`). A new `bonesremote release activate --config <registry>` command (root-only, sudoers-allowed) atomically flips the symlink via `rename(2)`. `foo-build` stages the release but does not flip `current`.

**Tradeoff (lazy alternative):** let `foo-build` flip `current` (it owns `project_root` parent at `2751`). Simpler (no new privileged command) but gives the build identity control over which release is live — a compromised `foo-build` could point `current` at a malicious release. The root-mediated flip is one extra sudoers line and ~15 lines of Rust; worth it for the blast-radius reduction.

### Q14. Rollback?

**My recommendation: `bonesremote release rollback --config <registry>` (root-only).** Reads the registry, finds the previous release under `releases/`, flips `current` to it. No rebuild, no re-checkout. Since `current` is root-owned (Q13), rollback must be privileged.

### Q15. Service restart?

**Keep the current narrow sudoers, but tie it to the registry.** `git ALL=(root) NOPASSWD: <bonesremote> service restart --config /etc/bonesdeploy/sites/*` (note: registry path, not arbitrary `--config *`). `service::run` reads `project_name` from the registry, validates it matches an allowed pattern, and runs `systemctl restart <project>-nginx`. The `--config *` wildcard is replaced with a fixed registry path glob so `git` cannot point at an arbitrary file.

### Q16. AppArmor/systemd hardening?

**My recommendation: keep the existing per-site nginx + per-runtime app profiles, but:**
- Remove the `repo_bones_toml r,` grant from `project-nginx-profile.j2:32` (V3 fix — nginx config is at `/srv/conf/<project>/nginx.conf`, doesn't need the repo's `bones.toml`).
- Change `runtime_write_paths` for Rails/Django to point at `shared/...` symlinks, not `current/...` (V6 fix).
- Add an AppArmor profile for `foo-build`'s build scripts (future hardening; not v1).

### Q17. What does nginx need access to?

**System nginx (`www-data`):** only needs to traverse `/run/<project>/` (`0711`) and connect to the per-site nginx socket. Does **not** need to read release code or `bones.toml`. The per-site nginx (running as `foo-run`) reads `current/public/**` via AppArmor grant `r`.

**Per-site nginx (`foo-run`):** reads `/srv/conf/<project>/nginx.conf` (root:foo-run 0750), `current/public/**` (foo-build:foo-release, foo-run reads via group), `/run/<project>/*.sock` (rw). No access to `shared/` secrets, no access to `bones.toml`.

### Q18. How to avoid `www-data` as universal cross-site read key?

**My recommendation for v1: accept the documented ceiling** (`site-nginx.conf.j2:21-24` ponytail). The upgrade path (per the ponytail comment): set per-site nginx group to `www-data` and use `0660` socket mode, so only `www-data` can connect — but `www-data` is still global. The real fix is per-site nginx master processes running as distinct users, which is a larger refactor. For v1, `0711` on `/run/<project>/` + `0660` sockets (group `www-data`) is the lazy-correct ceiling.

### Q19. Minimum viable safe v1?

**My recommendation for the smallest change that fixes the critical violations:**

1. **Add `foo-build` user** in `bonesinfra/users.py` (per-site, system, nologin, HOME at `/var/lib/bonesdeploy/build/foo`).
2. **Re-provision `project_root`, `releases/`, `build/` as `foo-build:foo-release`** in `bonesinfra/directories.py` (replace `ctx.config.deploy_user` with `ctx.config.build_user`).
3. **Remove `git` from `runtime_group`** (`users.py:47` delete).
4. **Tighten `shared/` to `0750`** (`directories.py:72`), drop setgid (only `foo-run` writes).
5. **Add `/etc/bonesdeploy/sites/<project>.toml`** root-owned registry, written by `bonesremote init`, read by `bonesinfra` and privileged `bonesremote` commands.
6. **Sudoers:** replace `--config *` with `--config /etc/bonesdeploy/sites/*` for `service restart`; add `git ALL=(foo-build) NOPASSWD: <bonesremote> deploy --config /etc/bonesdeploy/sites/*` so the build runs as `foo-build`.
7. **`deploy.rs`:** run checkout + build scripts + publish as `foo-build` (via the sudoers rule). `cp -a` now preserves `foo-build` ownership into the release tree — correct.
8. **`wire_release`:** implement the `[shared] paths` symlinks (storage, bootstrap/cache, etc.), created as `foo-build`-owned symlinks pointing at `foo-run`-owned shared leaves.
9. **Move Rails/Django `runtime_write_paths` to `shared/...`** symlinks (V6 fix).
10. **Reconcile docs** (`02-filesystem-layout.md`, `PROJECT.md`, `03-identity-and-user-policy.md`, `22-desired-end-state-summary.md`) to the 3-identity model and the actual modes.

**Explicitly deferred to v2:** `git-shell` for `git` (Q1 deeper hardening), per-site AppArmor for build scripts, `www-data` cross-site fix (Q18), root-mediated `current` flip (Q13 — lazy v1 lets `foo-build` flip since it owns the parent), rollback command (Q14).

### Q20. What code/docs need to change?

**bonesinfra (Python):**
- `domain/context.py`: add `build_user` field, default `f"{project_name}-build"`.
- `deploys/setup/users.py`: create `foo-build`, remove `git` from `runtime_group` (line 47).
- `deploys/setup/directories.py`: owner `foo-build` for `project_root`, `releases/`, `build/`; `shared/` mode `0750`.
- `runtimes/rails/rails.py`, `runtimes/django/django.py`: `runtime_write_paths` → `shared/...`.
- `assets/apparmor/project-nginx-profile.j2:32`: remove `repo_bones_toml r,`.

**bonesdeploy (Rust):**
- `crates/shared/src/config.rs`: add `build_user_for()`, add registry-loading helpers.
- `crates/shared/src/paths.rs`: add `/etc/bonesdeploy/sites/` registry path constants.
- `crates/bonesremote/src/commands/init.rs`: write `/etc/bonesdeploy/sites/<project>.toml`; tighten sudoers to registry paths + `foo-build` deploy rule.
- `crates/bonesremote/src/commands/deploy.rs`: invoke checkout/build/publish as `foo-build` via `sudo -u`.
- `crates/bonesremote/src/commands/wire_release.rs`: implement `[shared] paths` symlinks; fix `.env.example` fallback (chown to `root:foo-run 0640` or fail loudly if secrets not pushed).
- `crates/bonesremote/src/commands/service.rs`: load `project_name` from registry, not `--config`.
- `crates/bonesdeploy/src/commands/secrets.rs`: validate `runtime_group` against registry.
- `crates/bonesdeploy/kit/hooks/hooks.sh:81,86`: invoke `bonesremote deploy` via `sudo -u foo-build`.

**Docs:**
- `docs/security/02-filesystem-layout.md`: fix modes table, delete "activation hardening" myth, document shared-symlink model.
- `docs/security/03-identity-and-user-policy.md`: 3-identity model.
- `docs/security/22-desired-end-state-summary.md`: 3-identity model.
- `docs/PROJECT.md`: fix `shared/` mode (0750 not 0711/2775), ownership table (foo-build not git for releases), wire_release coverage.

---

## 5. The one-sentence version

The redesign's spine is: **introduce `foo-build` per site, move every deploy-side ownership from `git` to `foo-build`, remove `git` from `runtime_group`, tighten `shared/` to `foo-run`-exclusive, add a root-owned site registry that privileged commands read instead of git-owned config, and wire shared symlinks for every runtime-mutable path so the runtime user never writes inside the release tree.**

---

## What I need from you

Before I draft an implementation plan, I need your calls on:

1. **Q1** — global `git` (git-shell) vs per-site `foo-ingress`?
2. **Q5** — move bare repos to `/srv/git` or keep `/home/git`?
3. **Q6** — archive-export (option A) vs `foo-build` read access to bare repo (option B)?
4. **Q7** — `bonesremote` runs as root and drops to `foo-build`, or `git` invokes `sudo -u foo-build bonesremote deploy`?
5. **Q11** — does `foo-build` read `.env` during build, or do secret-dependent warm-up commands run as `foo-run`?
6. **Q13** — root-mediated `current` flip (tighter) or `foo-build` flips (lazier)?
7. **Q19** — is the v1 scope right, or do you want `git-shell`/root-flip/rollback in v1 too?

Once I have those, I'll write the migration path as a sequenced checklist with exact file:line edits for both repos.
````

`docs/thinking/02_new_architecture_approach.md`:

````md
Yes. The v1 target should be:

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
````

The point is not perfect theoretical isolation. The point is a sane and explainable boundary:

```text
A build can produce an artifact.
A build cannot touch runtime state.
Runtime can touch runtime state.
Runtime cannot rewrite sealed release code.
git cannot directly touch site state.
Only bonesremote performs privileged host operations.
```

## Phase 1: simplify the identity model

Move away from `foo-build`, `foo-run`, `foo-release`, `foo-shared` for v1.

Use one per-site Unix identity:

```text
user:  foo
group: foo
shell: /usr/sbin/nologin
home:  /nonexistent or controlled non-login path
sudo:  none
```

`git` remains global ingress, but it should not be in `foo`. It should not own releases, shared files, build output, runtime state, or framework caches.

Conceptual rule:

```text
git owns repos.
foo owns mutable site state.
root owns host control and sealed releases.
```

## Phase 2: establish the filesystem model

Target layout:

```text
/home/git/foo.git
  git:git
  bare repo, ingress only

/srv/sites/foo/
  root:root or root:foo
  site boundary

/srv/sites/foo/shared/
  foo:foo
  persistent runtime state

/srv/sites/foo/releases/
  root:foo
  promoted release artifacts

/srv/sites/foo/releases/<release_id>/
  root:foo
  readable by foo, not writable by foo

/srv/sites/foo/current
  root:root symlink to active release
```

Laravel shared paths should be explicit:

```text
shared/.env
shared/storage/
shared/bootstrap/cache/
shared/database/database.sqlite
```

Release symlinks:

```text
release/.env                    -> shared/.env
release/storage                 -> shared/storage
release/bootstrap/cache         -> shared/bootstrap/cache
release/database/database.sqlite -> shared/database/database.sqlite
release/public/storage          -> ../storage/app/public
```

Do not put a permanent `build/` directory under `/srv/sites/foo` for v1. Builds should be throwaway.

## Phase 3: make builds disposable

The deploy flow should become:

```text
1. git receives push.
2. git invokes bonesremote through a narrow sudo rule.
3. bonesremote reads root-owned site registry.
4. bonesremote exports source into a temporary build context.
5. podman runs the build.
6. podman receives no shared/, no .env, no SQLite, no current release.
7. build scripts mutate the source tree into deployable shape.
8. container is destroyed.
9. temporary build context is deleted after promotion/failure.
```

The build container should see only something like:

```text
/workspace/source
/cache/composer
/cache/npm
/tmp
```

It should not see:

```text
/srv/sites/foo/shared
/srv/sites/foo/current
/srv/sites/foo/releases
/home/git/foo.git
/etc/bonesdeploy
/root
```

For Laravel, sandbox build commands are probably:

```sh
composer install --no-dev --prefer-dist --no-interaction --optimize-autoloader

if [ -f package.json ]; then
  npm ci
  npm run build
fi
```

Do not run migrations, config caching, storage linking, queue restarts, or service restarts inside the build container.

## Phase 4: promote artifacts through bonesremote

After the Podman build succeeds, BonesRemote should promote the mutated source tree.

Conceptually:

```text
/workspace/source -> /srv/sites/foo/releases/<release_id>
```

Build scripts run with `cwd=/workspace/source` and write everything needed for runtime into that tree. They must not depend on `.git`, because source export should not expose Git history to the build container.

But BonesRemote must treat the built source tree as untrusted. It should normalize and seal it:

```text
owner: root
group: foo
dirs:  0750
files: 0640
executables/scripts: 0750
```

It should reject or carefully handle dangerous artifact contents:

```text
absolute symlinks
symlinks escaping the release/shared boundary
setuid/setgid bits
device files
FIFOs/sockets
unexpected ownership/modes
```

This matters because root is promoting files produced by project-controlled build code.

## Phase 5: wire shared paths before runtime prepare

Before Laravel runtime commands run, BonesRemote wires the release:

```text
.env
storage/
bootstrap/cache/
database/database.sqlite
public/storage
```

Those writable paths point into `shared/`, which is owned by `foo:foo`.

The sealed release itself remains `root:foo`, so `foo` can read the app but cannot rewrite normal code.

## Phase 6: run runtime prepare as foo

This is the phase where Laravel touches runtime state.

Run as `foo`, not root:

```sh
cd "$BONES_RELEASE_PATH"

php artisan migrate --force
php artisan optimize
```

Potentially:

```sh
php artisan package:discover --ansi
php artisan queue:restart
```

But keep the conceptual split clear:

```text
build phase:
  no secrets, no shared state

runtime prepare phase:
  sees .env
  sees SQLite
  sees storage
  writes framework caches
  runs migrations

root phase:
  flips current
  restarts system services
```

This is the key Laravel baseline. Other frameworks can use the same phase model even if some phases are noops.

## Phase 7: activate only after prepare succeeds

After runtime prepare succeeds:

```text
current -> releases/<release_id>
```

Activation should be done by BonesRemote/root.

Then restart/reload the relevant service.

Conceptual order for Laravel v1:

```text
build artifact
promote release
wire shared paths
runtime prepare as foo
activate current
restart service
```

The important tradeoff: migrations may mutate the DB before activation. That is normal in many deployments, but it means rollback is code rollback, not database rollback. Document this clearly.

## Phase 8: add a root-owned site registry

Privileged operations should not trust git-owned config.

Add:

```text
/etc/bonesdeploy/sites/foo.toml
```

This should be the trusted source for:

```text
site name
repo path
site root
runtime user/group
shared path
release path
service names
framework
allowed deploy paths
```

Repo-owned config can still describe app preferences, but privileged decisions should come from the root-owned registry.

## Phase 9: update the hook model

Current problem: `git` currently does too much.

Target:

```text
git post-receive hook:
  identify repo + revision
  call narrow sudo bonesremote deploy for that registered site
```

Not:

```text
git checks out code
git runs composer/npm
git writes releases
git writes shared
git restarts services
```

`git` becomes a trigger, not a deploy user.

## Phase 10: keep v1 intentionally scoped

For v1, do not solve everything.

Include:

```text
one per-site foo user/group
git removed from site groups
root-owned site registry
ephemeral Podman builds
no shared/.env mounted into build
source-tree promotion
root:foo sealed releases
foo-owned shared state
Laravel shared symlinks
runtime prepare phase as foo
root activation/restart
```

Defer:

```text
perfect rollback of database migrations
per-site build users
per-site ingress users
full systemd sandboxing
per-framework advanced queue/octane/reverb handling
www-data global socket isolation
complex ACL/group designs
```

## The one-sentence implementation path

First make `git` ingress-only, then make `foo` the only site identity, then move builds into temporary Podman containers with no runtime mounts, then promote the mutated source tree into root-owned sealed releases, then run Laravel runtime commands as `foo` against shared state, then activate through BonesRemote.

That is the cleanest v1 we have arrived at.

````

`docs/thinking/03_bonesdeploy_bonesremote_concerns.md`:

```md
# BonesDeploy and BonesRemote Concerns

This document defines the concerns that belong in the Rust tools:

- `bonesdeploy`: the local CLI used by the operator.
- `bonesremote`: the remote release orchestrator and privileged mediator.

It intentionally excludes concerns handled by `bonesinfra`, such as OS package installation, user creation, directory provisioning, systemd unit rendering, nginx configuration, AppArmor profile installation, firewall setup, Podman installation, and other host bootstrap/runtime infrastructure work.

The target security model is:

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
````

## Shared Principles

`bonesdeploy` and `bonesremote` should enforce the deployment lifecycle, not provision the machine.

Their responsibilities are:

- trust only root-owned registry data for privileged decisions.
- treat repo-owned config and build output as untrusted input.
- keep `git` as a trigger, not a deploy identity.
- keep builds disposable and isolated from runtime state.
- harden build artifacts before promoting them into releases.
- run runtime preparation as the per-site user.
- activate releases only after preparation succeeds.
- avoid deployment-time ownership repair except for artifact sealing performed by `bonesremote`.

Their responsibilities are not:

- creating Unix users or groups.
- installing Podman or language runtimes.
- creating base directories under `/srv`, `/run`, `/etc`, or `/var/lib`.
- rendering or installing nginx, systemd, or AppArmor files.
- installing sudoers policy; `bonesinfra` installs the single narrow sudoers drop-in.
- changing firewall rules.
- deciding host-level package policy.

## `bonesdeploy` Concerns

`bonesdeploy` is the local operator interface. It should prepare project-owned configuration, invoke remote commands, and synchronize project deployment assets.

### Remote Control-Plane Sync

`bonesdeploy push` should no longer copy `.bones/` into each bare Git repository. Bare repos should stay boring: they receive source code and trigger deployments, but they should not store the deployment control plane.

Instead, `bonesdeploy push` should send a deployment dataset to `bonesremote`, and `bonesremote` should import that dataset into its own central remote state, for example:

```text
/root/.config/bonesremote/
  sites/
    foo/
      registry.toml
      runtime.toml
      deployment/
        build/
          01_install_deps.sh
          02_build_assets.sh
        prepare/
          01_migrate.sh
          02_optimize.sh
```

The important split is authority:

- `registry.toml` is canonical remote state accepted and written by `bonesremote` after validation.
- deployment scripts are user-provided data, stored by `bonesremote`, but they do not authorize privileged paths, users, groups, or service names.
- the bare repo remains under `/home/git/<project>.git` and is only source ingress.

This keeps the existing operator-facing command name, but changes its meaning from "sync `.bones/` into Git internals" to "publish the deployment dataset to the remote control plane."

`bonesremote` should validate and normalize the incoming dataset before making it current. Privileged values should be derived or constrained server-side. For v1, convention over configurability is preferred: paths like `/home/git/<project>.git`, `/srv/sites/<project>`, `/srv/sites/<project>/shared`, and service names like `<project>-nginx.service` should not be freely supplied by repo-owned config.

Imports should be transactional enough to avoid half-updated remote state, but v1 does not need a generation/history model. Write a temporary sibling directory, validate it, then atomically replace the current site directory.

```text
/root/.config/bonesremote/sites/foo/
  registry.toml
  runtime.toml
  deployment/
```

The drift model is intentional: editing local `.bones/` does nothing remotely until `bonesdeploy push` publishes a new deployment dataset.

### Local Project Configuration

`bonesdeploy` owns local project setup concerns:

- initialize `.bones/` project scaffolding.
- write and update `.bones/bones.toml`.
- write and update `.bones/runtime.toml` when selecting a runtime template.
- install or repair local Git hooks that call back into `bonesdeploy` or remote hooks.
- keep local config values coherent enough to call remote commands.

`bonesdeploy` should not treat local config as authority for privileged remote operations. Local config is operator input; the remote root-owned site registry is authority.

### Operator Commands

`bonesdeploy` owns user-facing command orchestration:

- `init` for local project scaffolding.
- `doctor` for local checks and remote reachability checks.
- `push` for publishing the local `.bones/` deployment dataset into `bonesremote`'s remote control-plane state.
- `pull` for recovering the currently published deployment dataset from `bonesremote` when needed.
- `deploy` for explicitly asking the remote host to deploy the configured site.
- `rollback` for explicitly asking the remote host to roll back the configured site.
- `secrets` commands for local encrypted secret editing and remote secret delivery.
- `config`, `manage`, and `version` commands.

When a command needs privileged remote behavior, `bonesdeploy` should call `bonesremote` through SSH and let `bonesremote` validate against the root-owned registry.

### Remote Invocation

`bonesdeploy` may initiate remote actions, but it should not implement their privileged logic locally.

It may:

- SSH into the remote host.
- pass the intended site identifier or registry path.
- pass the intended revision when deploying a specific commit.
- stream or upload project-owned files where required.
- report remote command failures clearly.

It should not:

- compute trusted remote ownership decisions from local `.bones/*.toml`.
- directly `chown`, `chmod`, or mutate privileged paths over SSH.
- restart services directly.
- flip `current` directly.
- write release artifacts directly.

### Secrets Delivery

`bonesdeploy` owns local encrypted secret management and transport.

It may:

- manage encrypted secret files under `.bones/secrets/`.
- decrypt secrets locally for editing.
- upload secret material to a remote command or staging location.

It should not decide final ownership, group, mode, or destination from repo-owned config. Final placement must be mediated by `bonesremote` using the root-owned registry.

## `bonesremote` Concerns

`bonesremote` is the remote deployment state machine. It owns the secure transition from a Git revision to an activated release.

### Registry-Based Authority

Privileged `bonesremote` commands must read authority from the root-owned site registry, for example:

```text
/etc/bonesdeploy/sites/foo.toml
```

The registry is the trusted source for:

- site name.
- repository path.
- site root.
- shared path.
- releases path.
- current symlink path.
- runtime user and group.
- service names.
- framework/runtime identifier.
- allowed deploy paths.

`bonesremote` may read repo-owned `.bones/bones.toml` and `.bones/runtime.toml` only as untrusted deployment preferences. Those files must not authorize privileged paths, users, groups, or service names.

### Deploy Orchestration

`bonesremote deploy` owns the remote deployment sequence:

```text
validate registry
resolve revision
export source
run disposable build
promotion hardening
wire shared paths
run runtime prepare as foo
activate current as root
restart service as root
prune old releases
clean temporary state
```

On failure, `bonesremote` should clean temporary build state and incomplete release state without weakening permissions or mutating the active release.

### Source Export

`bonesremote` owns converting a registered Git revision into a build input.

It should:

- validate the requested repo and revision against the root-owned registry.
- export source into a temporary build context.
- avoid giving the build environment direct access to the bare repo.
- avoid exposing `.git` history unless explicitly required later.

The build context should be disposable and deleted after success or failure.

### Disposable Build Execution

`bonesremote` owns invoking the build, but not provisioning the container runtime.

It should:

- run builds in a temporary Podman container.
- mount only source, caches, and temporary directories required for the build.
- avoid mounting `shared/`, `.env`, SQLite databases, `current`, `releases/`, `/etc/bonesdeploy`, `/root`, or the bare repo.
- treat build scripts as project-controlled and untrusted.
- run build scripts with `cwd=/workspace/source` and promote the mutated source tree.

It should not install Podman, configure registries, install language packages on the host, or create cache directories. Those are `bonesinfra` concerns.

### Promotion Hardening

`bonesremote` owns promotion hardening: turning untrusted build output into a sealed release.

Podman output is not a release. It is raw build output produced by project-controlled code. Before activation, `bonesremote` must validate it, reject unsafe filesystem entries, and normalize the accepted files into a release tree.

`bonesremote` should:

- copy accepted built source into a new release directory.
- set release ownership to `root:foo`.
- set directory modes to `0750`.
- set regular file modes to `0640`.
- preserve executable bits only where appropriate, using `0750`.
- clear setuid and setgid bits.
- reject device files, FIFOs, sockets, and unsafe special files.
- reject or safely rewrite symlinks that are absolute or escape the release/shared boundary.

This is a core `bonesremote` concern because root is turning output generated by project-controlled build code into host-owned release files.

### Shared Path Wiring

`bonesremote` owns wiring runtime-mutable paths into a sealed release before runtime preparation.

For Laravel v1, this includes:

```text
release/.env                     -> shared/.env
release/storage                  -> shared/storage
release/bootstrap/cache          -> shared/bootstrap/cache
release/database/database.sqlite -> shared/database/database.sqlite
release/public/storage           -> ../storage/app/public
```

The release remains sealed as `root:foo`. The symlink targets under `shared/` remain mutable state owned by `foo:foo`.

By default, `bonesinfra` should create the shared files and directories a runtime needs, such as `shared/storage/` or `shared/database/database.sqlite`.

`bonesremote` should normally only link those existing shared paths into a release. If a deploy-time command ever creates a missing shared path, it must use the trusted registry path for `shared/`, not a path supplied by repo-owned config.

### Runtime Prepare

`bonesremote` owns running post-build runtime preparation commands as the per-site user.

For Laravel, this phase may include:

```sh
php artisan migrate --force
php artisan optimize
php artisan package:discover --ansi
php artisan queue:restart
```

This phase runs as `foo`, not root and not `git`.

This phase may see `.env`, SQLite, storage, and framework caches through `shared/` symlinks.

This phase must complete before activation. If it fails, the release must not become current.

Migrations can mutate the database before activation. `bonesremote` should make that ordering explicit in logs and documentation: rollback is code rollback, not database rollback.

### Activation and Service Control

`bonesremote` owns privileged activation and service control.

It should:

- atomically update `current` to the prepared release.
- restart or reload only registry-approved services.
- support rollback by repointing `current` to a previous sealed release.
- refuse service names or paths supplied only by repo-owned config.

`git` should reach this behavior only through a narrow sudo rule that permits the intended `bonesremote` command for registered sites.

### Release State and Cleanup

`bonesremote` owns deployment state tracking on the remote host.

It should:

- track the staged release being prepared.
- distinguish temporary build contexts from promoted releases.
- remove failed temporary build contexts.
- remove incomplete releases that never activated.
- prune old sealed releases according to registry or project policy.
- leave the active release untouched on failure.

## Hook Model Concerns

Git hooks should be thin triggers.

The remote `post-receive` hook should:

- identify the pushed repo and revision.
- exit early when the configured deployment ref was not updated.
- call a narrow `sudo bonesremote deploy ...` command for the registered site.

It should not:

- check out code.
- run build commands.
- write releases.
- write shared state.
- restart services.
- make ownership decisions.

## Explicitly Out of Scope for These Tools

The following concerns belong to `bonesinfra`, not `bonesdeploy` or `bonesremote`:

- creating the `git` user.
- creating the per-site `foo` user and group.
- removing `git` from site groups.
- creating `/home/git`, `/srv/sites/foo`, `/srv/sites/foo/shared`, `/srv/sites/foo/releases`, `/run/foo`, or cache roots.
- setting base ownership and modes for provisioned directories.
- installing `bonesremote` onto the host.
- installing Podman.
- configuring Podman policy, storage, or registries.
- installing PHP, Composer, Node, npm, database clients, or framework dependencies on the host.
- rendering and installing systemd units.
- rendering and installing nginx config.
- rendering and installing AppArmor profiles.
- installing sudoers policy. `bonesinfra` owns the narrow sudoers drop-in.
- obtaining TLS certificates.
- configuring firewall rules.
- applying OS hardening.

`bonesdeploy` and `bonesremote` may validate that these prerequisites exist and report actionable errors, but they should not silently provision or repair them during deployment.

## Minimal v1 Boundary

The minimum useful v1 split is:

- `bonesinfra` provisions the host, users, directories, services, Podman, and the registry parent directory.
- `bonesremote` writes and reads the site registry file used for privileged deployment decisions.
- `bonesdeploy` prepares local project config and asks the remote to deploy.
- `git` receives pushes and triggers `bonesremote` only.
- `bonesremote` validates registry data, builds in disposable Podman, hardens artifacts into sealed releases, wires shared paths, runs runtime prepare as `foo`, activates as root, restarts services, and cleans up.

The simplest rule is:

```text
bonesinfra creates the stage.
bonesdeploy asks for a deployment.
bonesremote performs the deployment safely.
```

````

`docs/thinking/04_scout_migration_impact.md`:

```md
# Scout Report: Migration Impact Analysis

**Target model** (from `02_new_architecture_approach.md` + `03_bonesdeploy_bonesremote_concerns.md`):

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary, disposable build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
````

______________________________________________________________________

## Files to DELETE

These are obsolete under the new model:

| File | Reason |
|------|--------|
| `crates/bonesremote/src/commands/post_receive.rs` | Replaced: `git checkout -f` into permanent `build/workspace` becomes disposable `git archive` export into temporary podman build context. |
| `crates/bonesremote/src/commands/stage_release.rs` | Replaced: creates release dir + ensures permanent `build/workspace`. New model: release dir creation is part of promotion step; no permanent build dir exists. |
| `crates/bonesremote/src/commands/wire_release.rs` | Replaced: only wires `.env`. New model must wire all shared paths (`storage/`, `bootstrap/cache/`, `database/database.sqlite`, etc.) into the **promoted release**, not the build workspace. Different target, different phase. |
| `crates/bonesremote/src/release/scripts.rs` | Replaced: runs deployment scripts via `bash` in build workspace as `git` user. New model: build runs in podman container; this module becomes podman invocation instead. |
| `crates/bonesdeploy/kit/deployment/01_install_build_deps.sh` | Replaced: installs nvm/npm into `$HOME` of deploy user. New model: build deps are in container image, not installed per-deploy. |
| `crates/bonesdeploy/kit/deployment/02_run_build.sh` | Replaced: runs `composer install`/`npm run build` in workspace as deploy user. New model: these run inside podman container. |

______________________________________________________________________

## Files requiring MAJOR REWRITE (>50% changed)

| File | Current behavior | Required changes |
|------|-----------------|------------------|
| **`crates/bonesremote/src/commands/deploy.rs`** (260 lines) | Orchestrates: doctor → stage → post_receive → wire → deploy_scripts → publish → activate → restart → prune. Everything runs as `git`. | New pipeline: validate registry → export source → podman build → promotion hardening → wire shared paths → runtime prepare (as `foo`) → activate (as root) → restart service → prune → cleanup. Must load root-owned registry. Must invoke podman. Must `sudo -u foo` for runtime prepare. See §Pipeline below. |
| **`crates/bonesremote/src/commands/init.rs`** (86 lines) | Installs sudoers drop-in: `git ALL=(root) NOPASSWD: bonesremote service restart --config *`. | Must write `/etc/bonesdeploy/sites/<project>.toml` registry. Must install multi-rule sudoers: `git ALL=(root) NOPASSWD: bonesremote deploy --registry /etc/bonesdeploy/sites/*`, `bonesremote service restart --registry ...`, etc. |
| **`crates/shared/src/paths.rs`** (285 lines) | `DEPLOY_USER = "git"`, `DEFAULT_REPO_PARENT = "/home/git"`, permanent `BUILD_DIR`/`WORKSPACE_DIR`/`LOGS_DIR` in `Deployment` struct. | Remove `DEPLOY_USER` as a deploy identity, but keep `/home/git` as the bare repo parent. Remove `build_root`/`build_logs` from `Deployment` struct (build is disposable). Add registry/control-plane path constants. |
| **`crates/shared/src/config.rs`** (220 lines) | `Bones` struct with 13 fields. `Runtime` struct with 4 fields. `deployment_paths()` method. | Add `SiteRegistry` struct (root-owned, canonical). Remove `default_deploy_user()`. Add `registry_path_for(project_name)`. `deployment_paths()` must drop build workspace fields. |
| **`crates/bonesremote/src/privileges.rs`** (10 lines) | Only `ensure_root()`. | Add `ensure_root_or_die()`, `drop_privileges_to(user)`, maybe `setuid` wrapper. The mediator pattern needs both root operations and `sudo -u foo` sub-operations. |
| **`crates/bonesremote/src/release_state.rs`** (288 lines) | Staged release tracking via `.staged_release` file in `repo/bones/`. Lists releases, flips `current` symlink. | Remove staged-release tracking via git-owned file. Release state tracking moves to registry or temp file. Keep `point_symlink_atomically()`, `list_releases_sorted()`, `current_release_name()`/`current_release_dir()`. |
| **`crates/bonesdeploy/kit/hooks/hooks.sh`** (123 lines) | Calls `bonesremote deploy --config "$BONES_TOML"` directly as `git`. | Must call `sudo bonesremote deploy --registry /etc/bonesdeploy/sites/$PROJECT.toml`. Becomes thin trigger only. No config parsing, no branch resolution — all that moves to bonesremote. |
| **`crates/bonesdeploy/src/commands/secrets.rs`** (327 lines) | Pushes `.env` to `shared/.env` with `chown root:<runtime_group>` where `runtime_group` comes from git-owned `runtime.toml`. | Must validate `runtime_group` against root-owned registry. The `chown root:<group>` and destination path must use registry values, not repo-owned config. |
| **`crates/bonesremote/src/commands/service.rs`** (46 lines) | Loads `project_name` from `--config` (git-owned bones.toml). Trusts it after regex validation. | Must load `project_name` and `service_name` from root-owned registry. `--config` replaced with `--registry`. |

______________________________________________________________________

## Files requiring MODERATE EDIT (new structs/fields, removed dependencies)

| File | Changes |
|------|---------|
| **`crates/bonesremote/src/commands/activate_release.rs`** (30 lines) | Currently runs as `git` (no root check). Must run as root to flip `current` symlink. Add `privileges::ensure_root()`. Otherwise similar — reads staged release name, calls `point_symlink_atomically()`. |
| **`crates/bonesremote/src/commands/drop_failed_release.rs`** (30 lines) | Currently reads staged release from `repo/bones/.staged_release`. Must track failed release via temp state or registry. Otherwise similar cleanup logic. |
| **`crates/bonesremote/src/commands/post_deploy.rs`** (137 lines) | Pruning logic is largely correct — reads `current` symlink, lists releases, prunes oldest non-active. No identity change needed, but `releases_keep` should come from registry not repo config. |
| **`crates/bonesremote/src/commands/rollback.rs`** (33 lines) | Currently runs as `git` (no root check). Must run as root. `releases_keep` / config from registry. |
| **`crates/bonesremote/src/commands/doctor.rs`** | Adds checks for: podman available, registry exists, AppArmor profiles. |
| **`crates/bonesremote/src/config.rs`** | Must add `load_registry()` alongside existing `load()`. Registry loading from `/etc/bonesdeploy/sites/<project>.toml`. |
| **`crates/bonesremote/src/cli/args.rs`** | Add `--registry` arg alongside `--config`. Deploy subcommand takes `--registry`, `--revision`. Service restart takes `--registry`. |
| **`crates/bonesdeploy/src/commands/init_project.rs`** | Keep `/home/git/<name>.git` repo defaults. Template defaults change for the new deployment/control-plane model. |
| **`crates/bonesdeploy/src/commands/push_state.rs`** | Stop pushing `.bones/` to `bones/` in the bare repo. Publish the deployment dataset to `bonesremote` control-plane state instead. |
| **`crates/bonesdeploy/src/commands/pull_state.rs`** | Same path update. |
| **`crates/bonesdeploy/src/commands/doctor.rs`** | Remote checks change: verify `bonesremote` + podman available, registry exists. No longer checks `build/workspace`. |
| **`crates/bonesdeploy/src/commands/remote_setup.rs`** | Passes JSON to `bonesinfra setup apply`. JSON must include new fields: registry path, site user (`foo`), no `DEPLOY_USER` as deploy identity. |
| **`crates/bonesdeploy/src/commands/remote_runtime.rs`** | Passes JSON to `bonesinfra runtime apply`. Must reflect new identity model. |
| **`crates/bonesdeploy/src/infra/bonesinfra.rs`** | JSON payload construction for `setup apply` and `runtime apply` must use registry-aware fields. |
| **`crates/bonesdeploy/src/config.rs`** | Local config loading. Defaults stay on `/home/git`; privileged remote values are constrained by bonesremote. |
| **`crates/bonesdeploy/kit/bones.toml`** | Default repo_path template. |
| **`crates/bonesdeploy/kit/hooks/post-receive`** | Must become thin trigger: resolve ref, call `sudo bonesremote deploy --registry ...`. |
| **`crates/bonesdeploy/kit/hooks/pre-push`** | Local pre-push hook. Unchanged (still calls `bonesdeploy doctor --local`). |

______________________________________________________________________

## Files requiring MINOR EDIT (path constants, field removals)

| File | Changes |
|------|---------|
| `crates/bonesdeploy/src/infra/git.rs` | `RemoteConnectionDetails` parsing should continue to handle `/home/git/...` URLs. |
| `crates/bonesdeploy/src/infra/ssh.rs` | SSH user changes: `bonesdeploy deploy` may SSH as root (to run `bonesremote deploy --registry ...`) rather than as `git`. |
| `crates/bonesdeploy/src/infra/bootstrap_ssh.rs` | Bootstrap SSH user selection. |
| `crates/bonesdeploy/src/main.rs` | Version bump, no structural changes. |
| `crates/bonesremote/src/main.rs` | Version bump, no structural changes. |
| `crates/bonesremote/src/cli/dispatch.rs` | Wire new subcommands (promotion, source-export, runtime-prepare). |
| `crates/bonesdeploy/src/cli/dispatch.rs` | Update SSH target user for `deploy`/`rollback` commands. |
| `crates/bonesdeploy/src/ui/prompts.rs` | Prompt text updates for new default paths. |
| `crates/bonesdeploy/kit/runtime.toml` | Shared paths section becomes more explicit / validated. |

______________________________________________________________________

## NEW files needed

| File | Purpose |
|------|---------|
| `crates/bonesremote/src/commands/promote.rs` | **Promotion hardening**: copies podman build output into release dir. Sets `root:foo` ownership, `0750` dirs, `0640` files. Rejects setuid/setgid, device files, FIFOs, sockets, unsafe symlinks. |
| `crates/bonesremote/src/commands/source_export.rs` | **Source export**: `git --git-dir=<repo> archive <rev> \| tar -x -C <tmp>` into disposable temp dir. Validates repo/rev against registry. |
| `crates/bonesremote/src/commands/runtime_prepare.rs` | **Runtime prepare**: runs as `foo` via `sudo -u foo`. Executes framework commands (e.g. `php artisan migrate --force`, `php artisan optimize`). Sees `.env`/SQLite/storage via shared symlinks. |
| `crates/bonesremote/src/commands/build.rs` | **Podman build**: creates/disposes container, mounts source+cache, runs build scripts in `/workspace/source`, returns the mutated source tree for promotion. |
| `crates/bonesremote/src/commands/wire_shared.rs` | **Shared path wiring** (replaces old `wire_release.rs`): symlinks all shared paths (`.env`, `storage/`, `bootstrap/cache/`, `database/database.sqlite`, etc.) from `shared/` into the promoted release directory. |
| `crates/shared/src/registry.rs` | **Site registry schema** (`SiteRegistry` struct): canonical `project_name`, `repo_path`, `project_root`, `runtime_user`, `runtime_group`, `shared_root`, `releases_root`, `service_name`, `framework`. Load from `/etc/bonesdeploy/sites/<project>.toml`. |
| `crates/shared/src/hardening.rs` | **Artifact hardening logic**: validate symlinks, reject dangerous file types, normalize modes. Shared between promote and any future artifact import. |

______________________________________________________________________

## The new deploy pipeline (replaces `deploy.rs:23-44`)

```text
validate_registry
  └─ resolve_revision
       └─ source_export          (git archive → temp dir)
            └─ podman_build      (container: source → artifact)
                 └─ promote       (artifact → releases/<id>, root:foo, hardened)
                      └─ wire_shared  (symlink shared/ paths into release)
                           └─ runtime_prepare  (sudo -u foo: migrate, optimize)
                                └─ activate     (root: flip current symlink)
                                     └─ restart_service  (root: systemctl restart)
                                          └─ post_deploy  (prune old releases)
                                               └─ cleanup_temp (rm temp build dir)
```

On failure: delete temp dir, delete staged release dir (if promoted), clear any state files.

______________________________________________________________________

## Key identity shifts by file

| Concern | Old identity | New identity |
|---------|-------------|--------------|
| `git checkout` into workspace | `git` user | `root` exports archive |
| Run build scripts | `git` user | Podman container |
| Copy build → release | `git` user (`cp -a`, preserves git ownership) | `root` promotes (hardens, sets `root:foo`) |
| Wire `.env` / shared paths | `git` user | `root` creates symlinks (targets `foo:foo` shared) |
| Runtime prepare (migrate) | N/A (didn't exist) | `foo` user |
| Flip `current` symlink | `git` user | `root` |
| Restart service | `root` via sudo (same) | `root` via sudo (same, but registry-backed) |
| Prune old releases | `git` user | `root` or `foo` (TBD) |
| Push secrets to `shared/.env` | `git` user over SSH | `root` over SSH, validated against registry |

______________________________________________________________________

## Summary counts

| Category | Count |
|----------|-------|
| Files to DELETE | 6 |
| Files to MAJOR REWRITE | 10 |
| Files to MODERATE EDIT | 17 |
| Files to MINOR EDIT | 13 |
| NEW files | 7 |
| **Total files touched** | **~53** (of ~70 source files) |

The highest-risk changes are:

1. `deploy.rs` — complete pipeline rewrite (260 lines → ~150, but entirely new logic)
1. `paths.rs` / `config.rs` — shared crate schema changes cascade to every consumer
1. `release_state.rs` — removing repo-owned staged state tracking
1. `hooks.sh` — changing the git hook trigger boundary

Files that do NOT change:

- `crates/bonesdeploy/src/commands/version.rs` / `config.rs` / `manage.rs` / `guide.rs` / `update.rs` / `status.rs`
- `crates/bonesremote/src/commands/version.rs` / `config.rs` / `status.rs`
- Most `ui/` helpers, `infra/embedded.rs`, `infra/rsync.rs`
- Test infrastructure patterns (but test data must update)

````

`docs/thinking/05_security_model_migration_plan.md`:

```md
# Security Model Migration Plan

This plan migrates BonesDeploy to the v1 security model without compatibility ballast. There are no existing installs to preserve.

Target boundary:

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary, disposable build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
````

Hard decisions already made:

- Keep bare repositories under `/home/git/<project>.git`.
- Do not push `.bones/` into bare repos.
- `bonesdeploy push` publishes the deployment dataset to `bonesremote` control-plane state.
- Use `/root/.config/bonesremote` for v1 remote control-plane state.
- No control-plane generations/history model.
- Build scripts mutate `/workspace/source`; no required `/workspace/output`.
- Prepare scripts run on the host as `foo` after shared paths are wired and before activation.
- Privileged paths, users, and groups come from bonesremote-controlled state, not repo-owned config.
- Service restart names use the existing `<project>-nginx.service` convention.
- Build containers do not mount caches.
- Secret delivery remains user-managed through `bonesdeploy secrets push`.
- The secret target is always `shared/.env`.

## Phase 0: Freeze the Contract

Before changing code, write down the v1 command and filesystem contract so the rewrite has a stable target.

Remote layout:

```text
/home/git/<project>.git
/root/.config/bonesremote/sites/<project>/
  registry.toml
  runtime.toml
  deployment/build/*.sh
  deployment/prepare/*.sh
/srv/sites/<project>/
  shared/
  releases/
  current -> releases/<release_id>
```

Deployment phases:

```text
hook trigger
validate control-plane state
resolve revision and branch policy
export source into temp dir
run build scripts in podman at /workspace/source
promote mutated source into sealed release
wire shared paths
run prepare scripts as foo
activate current as root
restart `<project>-nginx.service`
prune and cleanup
```

Script contract:

- Build scripts run in Podman with `cwd=/workspace/source`.
- Build scripts receive no `.env`, `shared/`, `current`, `releases/`, bare repo, `/root`, or control-plane mounts.
- Build scripts must not depend on `.git` existing.
- Build scripts write the deployable app shape into `/workspace/source`.
- Prepare scripts run as the site user with `cwd=<release_path>`.
- Prepare scripts can see wired shared paths and secrets.
- Prepare scripts are where migrations, optimize/cache commands, and runtime-state work belong.

Exit criteria:

- `docs/PROJECT.md` and command docs no longer describe bare-repo `.bones/` sync as the target behavior.
- The v1 script/environment contract is documented once and used by templates.

## Phase 1: Introduce Remote Control-Plane State

This is the first code boundary because it removes privileged authority from Git-owned files before the deploy pipeline is rewritten.

Implement bonesremote site state under:

```text
/root/.config/bonesremote/sites/<project>/
```

State contents:

- `registry.toml`: canonical derived state accepted by `bonesremote`.
- `runtime.toml`: user-editable runtime preferences after validation.
- `deployment/build/*.sh`: user-provided build scripts.
- `deployment/prepare/*.sh`: user-provided prepare scripts.

Keep it simple:

- Import into a temporary sibling path.
- Validate names, paths, script modes, runtime fields, and service identifiers.
- Replace the current site directory atomically enough to avoid partial reads.
- Do not add generations, rollback history, drift warnings, or audit trails.

`registry.toml` should derive or constrain privileged values:

- `site = <project>`.
- `repo_path = /home/git/<project>.git`.
- `site_root = /srv/sites/<project>`.
- `shared_root = /srv/sites/<project>/shared`.
- `releases_root = /srv/sites/<project>/releases`.
- `current_path = /srv/sites/<project>/current`.
- `runtime_user = <project>`.
- `runtime_group = <project>`.
- service names use the existing `<project>-nginx.service` convention.
- `branch` and `deploy_on_push` live here because hook filtering belongs to bonesremote.

Exit criteria:

- `bonesremote` can import, validate, read, and export one site's current dataset.
- Invalid site names cannot escape `/root/.config/bonesremote/sites` or derive arbitrary host paths.
- Privileged commands can load the registry without reading from the bare repo.

## Phase 2: Rewire `bonesdeploy push` and `pull`

Change the existing command names, not the operator workflow.

`bonesdeploy push` becomes:

```text
local .bones dataset -> SSH/stream/upload -> bonesremote import-site
```

`bonesdeploy pull` becomes:

```text
bonesremote export-site -> local .bones recovery/update
```

Do not write deployment control-plane data into `/home/git/<project>.git/bones/` anymore.

Local `.bones/bones.toml` remains operator input. It is not privileged remote authority.

Exit criteria:

- A local project can publish `.bones/runtime.toml` and deployment scripts to bonesremote state.
- Pull recovers the currently published dataset.
- Bare repos contain source and hooks only, not the control plane.

## Phase 3: Make Hooks Thin

Replace hook behavior with a narrow trigger.

Remote `post-receive` should do only enough to pass the pushed ref/revision to bonesremote:

```text
bonesremote hook post-receive --site <project>
```

`bonesremote hook post-receive` owns:

- reading stdin ref updates.
- loading control-plane state.
- checking `branch` and `deploy_on_push`.
- calling the internal deploy path for the accepted revision.

The hook must not:

- parse `.bones` from the repo.
- check out source.
- run build scripts.
- write releases or shared state.
- restart services.

Exit criteria:

- Pushing an unrelated branch does nothing.
- Pushing the configured branch deploys via bonesremote.
- `doctor` can verify the bare repo hook exists and points to the thin trigger.

## Phase 4: Replace Source Checkout with Source Export

Delete the permanent build workspace path from the deployment model.

`bonesremote` should export source with the bare repo as input and a temporary directory as output:

```text
git --git-dir=/home/git/<project>.git archive <revision> | tar -x -C <temp>/source
```

The build container sees `/workspace/source`, not the bare repo and not `.git` history.

Exit criteria:

- No `/srv/sites/<project>/build` workspace is required.
- Build input is deleted on success and failure.
- Revisions are resolved against the registry-approved repo path.

## Phase 5: Add Disposable Podman Build Execution

Replace host build scripts with container build scripts.

Template layout:

```text
.bones/deployment/build/01_install_deps.sh
.bones/deployment/build/02_build_assets.sh
.bones/deployment/prepare/01_migrate.sh
.bones/deployment/prepare/02_optimize.sh
```

Build execution:

- Run a configured image from `runtime.toml` after validation.
- Mount exported source as `/workspace/source`.
- Run build scripts in lexical order.
- Treat non-zero exit as deployment failure.
- Return the mutated `/workspace/source` path to the promotion step.

Do not hardcode Laravel commands in bonesremote. Templates can scaffold Laravel-friendly scripts, but scripts remain user-owned.

Exit criteria:

- Host deploys no longer install Composer, Node, npm, or build dependencies into the `git` user's home.
- Build has no access to secrets or runtime state.
- A failing build leaves no release and does not affect `current`.

## Phase 6: Promote and Harden the Mutated Source

Promotion turns untrusted built source into a sealed release.

Input:

```text
<temp>/source
```

Output:

```text
/srv/sites/<project>/releases/<release_id>
```

Promotion rules:

- Copy regular files and safe symlinks only.
- Reject device files, FIFOs, sockets, setuid, and setgid.
- Reject absolute symlinks and symlinks that escape the release/shared boundary.
- Set directories to `0750`.
- Set regular files to `0640`.
- Preserve executable intent as `0750` only where needed.
- Set owner/group to `root:<project>`.

Exit criteria:

- Runtime user can read and execute what it needs.
- Runtime user cannot rewrite sealed release code.
- Dangerous build output is rejected before activation.

## Phase 7: Wire Shared Paths

Replace `.env`-only wiring with declared shared path wiring.

Laravel v1 paths:

```text
release/.env                     -> shared/.env
release/storage                  -> shared/storage
release/bootstrap/cache          -> shared/bootstrap/cache
release/database/database.sqlite -> shared/database/database.sqlite
release/public/storage           -> ../storage/app/public
```

`bonesremote` should normally link existing shared paths. Base shared paths are a bonesinfra/runtime provisioning concern.

Exit criteria:

- A sealed release exposes the runtime-mutable paths Laravel needs.
- Shared targets are under registry-approved `shared_root`.
- Missing required shared paths fail loudly with an actionable error.

## Phase 8: Run Runtime Prepare as the Site User

Prepare is the first phase allowed to touch secrets and runtime state.

Run as:

```text
user: <project>
cwd:  /srv/sites/<project>/releases/<release_id>
```

Execute scripts from control-plane state:

```text
deployment/prepare/*.sh
```

Important tradeoff:

- Migrations may mutate the database before activation.
- If activation later fails, rollback is code rollback, not database rollback.
- Logs and docs must say this plainly.

Exit criteria:

- Prepare runs as `foo`, not `git` and not root.
- Failed prepare deletes the incomplete release and leaves `current` untouched.
- Successful prepare is required before activation.

## Phase 9: Activate, Restart, Prune

Privileged finalization stays in bonesremote.

Activation:

- Atomically repoint `/srv/sites/<project>/current` to the prepared release.
- Never activate a release that failed build, promotion, wiring, or prepare.

Service control:

- Restart/reload the fixed `<project>-nginx.service` service.
- Do not accept service names from repo-owned config or hook input.

Pruning:

- Keep the existing simple release retention model.
- Do not add config generations or audit-history pruning.
- Never prune the active release.

Exit criteria:

- Deploy failure before activation leaves the active release running.
- Rollback repoints `current` to a previous sealed release and restarts `<project>-nginx.service`.
- Old inactive releases are pruned according to the configured keep count.

## Phase 10: Update Doctor and Documentation

Doctor checks should match the new boundary.

Remote checks:

- bonesremote installed.
- Podman available.
- control-plane site state exists and validates.
- `/home/git/<project>.git` exists.
- thin hook installed.
- `/srv/sites/<project>/shared`, `releases`, and `current` parent exist with expected ownership shape.
- runtime user exists and is not `git`.
- `git` is not in the site group.
- `<project>-nginx.service` exists.

Docs to update:

- `docs/PROJECT.md` deployment model.
- `docs/commands/push.md` and `pull.md` control-plane semantics.
- `docs/commands/deploy.md` deploy sequence.
- `docs/commands/secrets.md` mediated placement.
- `docs/security/*` trust-boundary and permissions docs.

Exit criteria:

- Docs no longer say `.bones/` is rsynced into the bare repo.
- Docs no longer describe host-side build scripts running as `git`.
- Docs explicitly state the migration-before-activation database tradeoff.

## Suggested Implementation Order

1. Add shared registry/control-plane structs and path helpers.
1. Add bonesremote import/export of site datasets.
1. Rewire `bonesdeploy push` and `pull` to use import/export.
1. Replace remote hook with `bonesremote hook post-receive --site <site>`.
1. Replace permanent checkout/stage code with source export.
1. Add Podman build runner for `/workspace/source`.
1. Add promotion hardening.
1. Replace shared wiring.
1. Add prepare script runner as site user.
1. Rewrite deploy orchestration around the new phases.
1. Update doctor checks and docs.
1. Delete obsolete stage/post-receive/old deployment-script code.

This order keeps the trust boundary moving in one direction: first move authority out of Git, then make Git a trigger, then replace the deploy pipeline.

## Small Checks to Leave Behind

Each non-trivial phase should leave one runnable check:

- Control-plane import rejects `../evil` site names and absolute user-supplied paths.
- Hook filtering ignores non-configured branches.
- Source export produces a tree without `.git`.
- Podman build cannot read a fake mounted secret path.
- Promotion rejects an absolute symlink and a FIFO.
- Shared wiring rejects targets outside `shared_root`.
- Prepare runs as the site user, verified by a tiny script writing `id -un` to a temp file.
- Deploy failure before activation leaves `current` unchanged.

Keep these checks small. The goal is not a huge test harness; it is one tripwire per boundary.

````

`docs/thinking/05_security_model_migration_plan_addendum.md`:

```md
# Security Model Migration Plan Addendum

These decisions are now fixed:

- Service restart names stay on the existing `<project>-nginx.service` convention.
- Build containers do not mount any caches.
- Secret delivery remains user-managed through `bonesdeploy secrets push` for now.
- The secret target is always `shared/.env`.

This addendum only records the decisions above. It does not change the main migration plan.

````

`docs/todo.md`:

```md
- [ ] database.sqlite needs 0660. Currently getting 0664 by Laravel

```

`rustfmt.toml`:

```toml
# Stable options
max_width = 120
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
edition = "2021"
use_small_heuristics = "Max"

# Unstable options (require nightly or config flag)
# Uncomment when using nightly toolchain:
# imports_granularity = "Module"       # merge imports from the same module
# group_imports = "StdExternalCrate"   # group: std → external → crate
# use_field_init_shorthand = true      # prefer Point { x, y } over Point { x: x, y: y }
# format_code_in_doc_comments = true   # format Rust code inside doc comments
# wrap_comments = true                 # wrap long comments to max_width
# normalize_comments = true            # normalize /* */ to //
# format_strings = true                # break long string literals
# condense_wildcard_suffixes = true    # simplify nested wildcard patterns
# overflow_delimited_expr = true       # allow long macro/function args to overflow

```

`tests/ASSERTIONS.md`:

```md
# Test Assertions Inventory

> Note: this file is auto-generated by `cargo assertions`.

## `crates/bonesdeploy/src/commands/doctor.rs`
- sync check uses same excludes as push. (sync_check_uses_same_excludes_as_push)

## `crates/bonesdeploy/src/commands/guide.rs`
- init command is prompt free. (init_command_is_prompt_free)
- initialized report suggests setup then ssl then deploy. (initialized_report_suggests_setup_then_ssl_then_deploy)
- ready report suggests deploy. (ready_report_suggests_deploy)
- uninitialized report starts with init. (uninitialized_report_starts_with_init)

## `crates/bonesdeploy/src/commands/manage.rs`
- Escapes embedded single quotes safely for remote shell execution. (shell_quote_single_escapes_embedded_single_quotes)
- Returns an explicit empty string for empty input, not a zero-length argument. (shell_quote_single_handles_empty_string)
- Wraps a plain value in single quotes to prevent whitespace and token splitting. (shell_quote_single_wraps_plain_value_in_single_quotes)

## `crates/bonesdeploy/src/commands/push_state.rs`
- rsync args exclude local secrets directory only. (rsync_args_exclude_local_secrets_directory_only)

## `crates/bonesdeploy/src/commands/remote_data.rs`
- base data includes preview domain. (base_data_includes_preview_domain)
- Passes the SSL domain and email into the deploy data sent to bonesinfra (ssl_data_includes_domain_and_email)

## `crates/bonesdeploy/src/commands/secrets.rs`
- extract fingerprint parses fpr line. (extract_fingerprint_parses_fpr_line)
- extract fingerprint returns none without fpr line. (extract_fingerprint_returns_none_without_fpr_line)
- gpg home resolves under bones config root. (gpg_home_resolves_under_bones_config_root)

## `crates/bonesdeploy/src/commands/status.rs`
- service marker marks only active as ok. (service_marker_marks_only_active_as_ok)
- ssl state uses remote state when local flag is stale. (ssl_state_uses_remote_state_when_local_flag_is_stale)

## `crates/bonesdeploy/src/commands/tests/test_init_project.rs`
- Requires a host when neither existing config nor CLI provide one. (collect_non_interactive_requires_host_when_existing_and_cli_are_missing_it)
- Uses existing config and CLI values without prompting when non-interactive mode is active. (collect_non_interactive_uses_existing_and_cli_values_without_prompting)
- Materializes the base bonesdeploy kit and runtime config during init. (init_materializes_base_bones_assets)
- Repairs a dangling .bones symlink instead of failing with EEXIST. (init_repairs_dangling_bones_symlink)
- Keeps an already materialized local bones scaffold intact when init is run again. (init_rerun_preserves_existing_bones_assets)

## `crates/bonesdeploy/src/commands/update.rs`
- Extracts the package version from the `[package]` section of a Cargo manifest. (parses_package_version_from_manifest_package_section)
- Refreshes .bones scaffold assets from the cloned source tree without overwriting local config files. (refresh_local_bones_updates_scaffold_without_touching_configs)
- Returns an error when the manifest has no `[package]` section with a version field. (rejects_manifest_without_package_version)
- Verifies the update source repository and branch constants are set to the canonical values. (update_uses_master_branch_source_repository)

## `crates/bonesdeploy/src/config.rs`
- load applies default project root from project name. (load_applies_default_project_root_from_project_name)
- load applies default repo path from project name. (load_applies_default_repo_path_from_project_name)
- load preserves explicit repo and project root overrides. (load_preserves_explicit_repo_and_project_root_overrides)
- save includes derived repo and project root. (save_includes_derived_repo_and_project_root)
- save persists ssl settings. (save_persists_ssl_settings)

## `crates/bonesdeploy/src/infra/bonesinfra.rs`
- checkout dir lives under bones config root. (checkout_dir_lives_under_bones_config_root)
- reset checkout removes stale directory. (reset_checkout_removes_stale_directory)
- reset checkout removes stale file. (reset_checkout_removes_stale_file)

## `crates/bonesdeploy/src/infra/bonesinfra_cli.rs`
- base command launches venv python with module flag. (base_command_launches_venv_python_with_module_flag)
- parse json output reads cli stdout. (parse_json_output_reads_cli_stdout)

## `crates/bonesdeploy/src/infra/bootstrap_ssh.rs`
- defaults to root. (defaults_to_root)
- trims and rejects blank values. (trims_and_rejects_blank_values)
- uses config value. (uses_config_value)

## `crates/bonesdeploy/src/infra/embedded.rs`
- base runtime defaults read embedded kit runtime. (base_runtime_defaults_read_embedded_kit_runtime)
- runtime defaults read local runtime toml. (runtime_defaults_read_local_runtime_toml)
- runtime names come from embedded assets. (runtime_names_come_from_embedded_assets)
- scaffold runtime deployment replaces kit scripts. (scaffold_runtime_deployment_replaces_kit_scripts)
- scaffold runtime deployment writes runtime scripts. (scaffold_runtime_deployment_writes_runtime_scripts)
- vue runtime uses vite dist web root. (vue_runtime_uses_vite_dist_web_root)

## `crates/bonesdeploy/src/infra/git.rs`
- parse remote url rejects non git paths. (parse_remote_url_rejects_non_git_paths)
- parse remote url rejects non ssh urls. (parse_remote_url_rejects_non_ssh_urls)
- parse remote url rejects relative scp paths. (parse_remote_url_rejects_relative_scp_paths)
- parse scp style url parses absolute repo path. (parse_scp_style_url_parses_absolute_repo_path)
- parse scp style url trims surrounding whitespace. (parse_scp_style_url_trims_surrounding_whitespace)
- parse ssh style url defaults port to 22. (parse_ssh_style_url_defaults_port_to_22)
- parse ssh style url parses host port and repo path. (parse_ssh_style_url_parses_host_port_and_repo_path)

## `crates/bonesdeploy/src/ui/prompts.rs`
- Accepts common yes values like y, yes, and YES. (confirmation_parser_accepts_common_yes_values)
- Rejects non-affirmative values like empty string, n, and no. (confirmation_parser_rejects_non_affirmative_values)
- Mentions app services in the remote runtime prompt. (remote_runtime_prompt_lines_mention_app_services)
- Mentions VPS in the remote bootstrap prompt. (remote_setup_prompt_lines_mention_vps)

## `crates/bonesremote/src/commands/deploy.rs`
- Removes all direct children of a directory without removing the directory itself. (clear_directory_removes_all_direct_children)
- Returns deployment script files sorted and excludes subdirectories. (list_deployment_scripts_returns_sorted_files_only)
- Replaces the release tree contents with a fresh copy from the build workspace. (publish_release_tree_replaces_release_contents_with_build_workspace)

## `crates/bonesremote/src/commands/init.rs`
- approved bonesdeploy path accepts standard bin dirs. (approved_bonesdeploy_path_accepts_standard_bin_dirs)
- approved bonesdeploy path rejects unexpected dirs. (approved_bonesdeploy_path_rejects_unexpected_dirs)

## `crates/bonesremote/src/commands/post_deploy.rs`
- Keeps all releases when the active release count is within the keep limit. (prune_old_releases_keeps_active_release_when_within_keep_limit)
- Prunes the oldest inactive releases when the active release count exceeds the keep limit. (prune_old_releases_removes_oldest_inactive_releases_up_to_keep_limit)

## `crates/bonesremote/src/commands/post_receive.rs`
- `post-receive` checks out the requested revision into the staged build workspace. (post_receive_checks_out_requested_revision_into_build_workspace)
- `post-receive` fails with a permission error when the build workspace is inaccessible. (post_receive_reports_permission_denied_for_inaccessible_workspace)
- `post-receive` fails when the build workspace does not exist. (post_receive_requires_existing_build_workspace)

## `crates/bonesremote/src/commands/status.rs`
- parses unit names from systemctl output. (parses_unit_names_from_systemctl_output)

## `crates/bonesremote/src/commands/tests/test_doctor.rs`
- Accepts a yes value as indicating `AppArmor` is enabled in the kernel. (apparmor_kernel_enabled_accepts_yes)
- Rejects a no value as indicating `AppArmor` is not enabled in the kernel. (apparmor_kernel_enabled_rejects_no)
- Reads the first `AppArmor` profile assignment from a systemd unit file. (apparmor_profile_binding_reads_first_profile_assignment)
- Accepts a valid bonesdeploy `AppArmor` profile filename. (apparmor_profile_filename_accepts_bonesdeploy_profile)
- Rejects a filename that does not match the bonesdeploy profile naming convention. (apparmor_profile_filename_rejects_unrelated_file)
- Maps a bonesdeploy `AppArmor` profile name to its corresponding systemd unit name. (apparmor_unit_name_for_profile_maps_project_unit)
- Accepts a systemd unit with correctly wired `AppArmor` dependency and profile. (apparmor_unit_wiring_accepts_expected_unit_with_reordered_after_tokens)
- Rejects a systemd unit that is missing the `AppArmor` profile binding. (apparmor_unit_wiring_rejects_missing_profile_binding)
- Rejects a systemd unit that binds an unknown `AppArmor` profile. (apparmor_unit_wiring_rejects_unknown_profile_binding)

## `crates/bonesremote/src/commands/wire_release.rs`
- existing shared env is linked into workspace. (existing_shared_env_is_linked_into_workspace)
- existing shared env is not overwritten by env example. (existing_shared_env_is_not_overwritten_by_env_example)
- missing both env and example does not fail. (missing_both_env_and_example_does_not_fail)
- missing shared env with env example copies and links. (missing_shared_env_with_env_example_copies_and_links)
- remove workspace path removes files and directories. (remove_workspace_path_removes_files_and_directories)
- storage is not symlinked. (storage_is_not_symlinked)

## `crates/bonesremote/src/config.rs`
- load derives project root and repo path from project name. (load_derives_project_root_and_repo_path_from_project_name)
- load fails for invalid toml. (load_fails_for_invalid_toml)
- load fails for missing file. (load_fails_for_missing_file)
- load preserves explicit repo and project root. (load_preserves_explicit_repo_and_project_root)
- load uses defaults for missing fields. (load_uses_defaults_for_missing_fields)

## `crates/bonesremote/src/release/scripts.rs`
- deployment log path lives under build logs. (deployment_log_path_lives_under_build_logs)
- run deployment script applies group writable umask. (run_deployment_script_applies_group_writable_umask)
- run deployment script creates missing log directory. (run_deployment_script_creates_missing_log_directory)
- run deployment script preserves failing exit status. (run_deployment_script_preserves_failing_exit_status)
- run deployment script streams output to console and log. (run_deployment_script_streams_output_to_console_and_log)

## `crates/bonesremote/src/release_state.rs`
- clear staged release removes state file. (clear_staged_release_removes_state_file)
- current release name resolves from current symlink. (current_release_name_resolves_from_current_symlink)
- list releases sorted returns only directories in order. (list_releases_sorted_returns_only_directories_in_order)
- point symlink atomically creates parent dirs and points to target. (point_symlink_atomically_creates_parent_dirs_and_points_to_target)
- point symlink atomically repoints existing link. (point_symlink_atomically_repoints_existing_link)
- read staged release rejects empty file. (read_staged_release_rejects_empty_file)
- write then read staged release round trips. (write_then_read_staged_release_round_trips)

## `crates/shared/src/config.rs`
- validate host accepts hostnames and ips. (validate_host_accepts_hostnames_and_ips)
- validate host rejects shell metacharacters. (validate_host_rejects_shell_metacharacters)

## `crates/shared/src/paths.rs`
- bones config root uses xdg config home. (bones_config_root_uses_xdg_config_home)
- bones state root uses xdg state home. (bones_state_root_uses_xdg_state_home)

## `tests/cleancode/src/cleancode_file_too_long.rs`
- Verifies all Rust source files stay at or below 400 lines. (source_files_stay_under_400_lines)

## `tests/cleancode/src/cleancode_no_legacy_terms.rs`
- Detects legacy-language smell terms like legacy, hack, or workaround in source code. (no_legacy_terms)

## `tests/cleancode/src/cleancode_no_literal_wrapped_fallback.rs`
- Detects provably unnecessary fallback calls on `Some` or `Ok` wrappers. (no_literal_wrapped_fallback)

## `tests/cleancode/src/cleancode_no_manufactured_success.rs`
- Detects match arms that construct success values from error paths. (no_suspicious_fallback)

```

`tests/cleancode/Cargo.toml`:

```toml
[package]
name = "cleancode"
version = "0.1.0"
edition = "2024"

[dependencies]
syn = { version = "2", features = ["full", "extra-traits", "visit"] }

[lints]
workspace = true

```

`tests/cleancode/src/cleancode_file_too_long.rs`:

```rs
//! Integration test: all `.rs` source files must stay at or below 400 lines.
//!
//! This is a structural scan — no type inference needed.
//! Skips generated/dependency directories and local Git worktrees by convention.

use std::fs;
use std::path::Path;

const MAX_LINES: usize = 400;

/// Verifies all Rust source files stay at or below 400 lines.
#[test]
fn source_files_stay_under_400_lines() {
    let project_root = workspace_root();
    let mut violations: Vec<String> = Vec::new();
    let mut file_count = 0;

    visit_dirs(project_root, &mut file_count, &mut violations);

    assert!(file_count > 0, "No source files found. This test should be run from the project root.");

    assert!(violations.is_empty(), "File(s) exceed {} line(s):\n{}", MAX_LINES, violations.join("\n"),);
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tests/cleancode should live under the workspace root")
}

fn visit_dirs(dir: &Path, file_count: &mut usize, violations: &mut Vec<String>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if matches!(name_str.as_ref(), "target" | "vendor" | "node_modules" | ".git" | ".worktrees") {
                continue;
            }
            visit_dirs(&path, file_count, violations);
            continue;
        }

        if path.extension().is_some_and(|ext| ext == "rs") {
            *file_count += 1;
            if let Ok(content) = fs::read_to_string(&path) {
                let line_count = content.lines().count();
                if line_count > MAX_LINES {
                    let relative = path.strip_prefix(workspace_root()).unwrap_or(&path);
                    violations.push(format!("  {}: {} lines (max {MAX_LINES})", relative.display(), line_count));
                }
            }
        }
    }
}

```

`tests/cleancode/src/cleancode_no_duplicated_state.rs`:

```rs
//! Integration test: detect duplicated state strings in production Rust code.
//!
//! Repeating string-backed state in multiple files makes later changes easy to
//! miss. Product-owned paths, config keys, and environment variables should have
//! one canonical constant.
//!
//! Syntax-only: scans string literals without type inference.

use std::collections::BTreeMap;
use std::fs;
use std::ops::RangeInclusive;
use std::path::Path;

const IGNORE_DIRS: &[&str] = &["target", "vendor", "node_modules", ".git", ".worktrees", "tests"];

#[derive(Debug)]
struct StateLiteral {
    value: String,
    kind: StateKind,
    location: String,
    context: String,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum StateKind {
    Config,
    Env,
    Path,
}

/// Detects duplicated string literals that carry product state.
#[test]
fn no_duplicated_state_literals() {
    let project_root = workspace_root();
    let literals = collect_state_literals(project_root);

    assert!(!literals.is_empty(), "No production Rust state literals found under crates/.");

    let mut by_value: BTreeMap<(StateKind, String), Vec<StateLiteral>> = BTreeMap::new();
    for literal in literals {
        by_value.entry((literal.kind, literal.value.clone())).or_default().push(literal);
    }

    let violations = by_value
        .into_iter()
        .filter(|(_, literals)| literals_are_duplicated_state(literals))
        .map(|((kind, value), literals)| format_violation(kind, &value, &literals))
        .collect::<Vec<_>>();

    assert!(
        violations.is_empty(),
        "Duplicated state literals found. Review each path/config/env value and centralize intentional state:\n{}",
        violations.join("\n")
    );
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tests/cleancode should live under the workspace root")
}

fn collect_state_literals(project_root: &Path) -> Vec<StateLiteral> {
    let mut literals = Vec::new();
    let mut stack = vec![project_root.join("crates")];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name();
                if !IGNORE_DIRS.contains(&name.to_string_lossy().as_ref()) {
                    stack.push(path);
                }
                continue;
            }

            if path.extension().is_some_and(|ext| ext == "rs") {
                collect_file_literals(project_root, &path, &mut literals);
            }
        }
    }

    literals
}

fn collect_file_literals(project_root: &Path, path: &Path, literals: &mut Vec<StateLiteral>) {
    let Ok(content) = fs::read_to_string(path) else {
        return;
    };
    let ignored_lines = ignored_test_line_ranges(&content);

    for (line_number, line) in content.lines().enumerate() {
        let line_number = line_number + 1;
        if ignored_lines.iter().any(|range| range.contains(&line_number)) {
            continue;
        }

        if line.contains("env!(\"") {
            continue;
        }

        let relative = path.strip_prefix(project_root).unwrap_or(path);
        for value in parse_string_literals(line) {
            let Some(kind) = state_kind(&value) else {
                continue;
            };

            if is_noise_value(&value) {
                continue;
            }

            literals.push(StateLiteral {
                value,
                kind,
                location: format!("{}:{line_number}", relative.display()),
                context: line.trim().to_string(),
            });
        }
    }
}

fn ignored_test_line_ranges(content: &str) -> Vec<RangeInclusive<usize>> {
    let lines = content.lines().collect::<Vec<_>>();
    let mut ignored = Vec::new();
    let mut index = 0;

    while index < lines.len() {
        let trimmed = lines[index].trim();
        if !is_test_attribute(trimmed) {
            index += 1;
            continue;
        }

        let start = index + 1;
        index += 1;

        while index < lines.len() && is_attribute_line(lines[index].trim()) {
            index += 1;
        }

        let Some(end) = find_item_end_line(&lines, index) else {
            ignored.push(start..=start);
            continue;
        };

        ignored.push(start..=end);
        index = end;
    }

    ignored
}

fn is_test_attribute(line: &str) -> bool {
    matches!(line, "#[cfg(test)]" | "#[test]")
}

fn is_attribute_line(line: &str) -> bool {
    line.starts_with("#[")
}

fn find_item_end_line(lines: &[&str], start_index: usize) -> Option<usize> {
    if start_index >= lines.len() {
        return None;
    }

    let mut brace_depth = 0usize;
    let mut saw_open_brace = false;

    for (index, line) in lines.iter().enumerate().skip(start_index) {
        for ch in line.chars() {
            if ch == '{' {
                brace_depth += 1;
                saw_open_brace = true;
            } else if ch == '}' && brace_depth > 0 {
                brace_depth -= 1;
                if brace_depth == 0 && saw_open_brace {
                    return Some(index + 1);
                }
            }
        }

        if !saw_open_brace && line.trim_end().ends_with(';') {
            return Some(index + 1);
        }
    }

    if saw_open_brace { Some(lines.len()) } else { None }
}

fn parse_string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut chars = line.chars().peekable();
    let mut in_string = false;
    let mut value = String::new();
    let mut escaped = false;

    while let Some(ch) = chars.next() {
        if !in_string {
            if ch == '/' && chars.peek() == Some(&'/') {
                break;
            }
            if ch == '"' {
                in_string = true;
                value.clear();
            }
            continue;
        }

        if escaped {
            value.push(ch);
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
            continue;
        }

        if ch == '"' {
            literals.push(value.clone());
            in_string = false;
            continue;
        }

        value.push(ch);
    }

    literals
}

fn state_kind(value: &str) -> Option<StateKind> {
    if is_path_like(value) {
        return Some(StateKind::Path);
    }
    if is_env_var_like(value) {
        return Some(StateKind::Env);
    }
    if is_config_like(value) {
        return Some(StateKind::Config);
    }

    None
}

fn is_path_like(value: &str) -> bool {
    value.contains('/')
        || value.starts_with('.')
        || [".gpg", ".service", ".sh", ".socket", ".toml"].iter().any(|suffix| value.ends_with(suffix))
}

fn is_env_var_like(value: &str) -> bool {
    value.len() >= 3
        && value.chars().all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        && value.chars().any(|ch| ch.is_ascii_uppercase())
}

fn is_config_like(value: &str) -> bool {
    value.contains('_')
        && value.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
        && value.chars().any(|ch| ch.is_ascii_lowercase())
}

fn literals_are_duplicated_state(literals: &[StateLiteral]) -> bool {
    literals.len() > 1
        && literals
            .iter()
            .map(|literal| file_path(&literal.location))
            .collect::<Vec<_>>()
            .windows(2)
            .any(|pair| pair[0] != pair[1])
}

fn file_path(location: &str) -> &str {
    location.rsplit_once(':').map_or(location, |(path, _)| path)
}

fn is_noise_value(value: &str) -> bool {
    matches!(value, "." | "{}/" | "{}")
}

fn format_violation(kind: StateKind, value: &str, literals: &[StateLiteral]) -> String {
    let locations = literals
        .iter()
        .map(|literal| format!("{} `{}`", literal.location, literal.context))
        .collect::<Vec<_>>()
        .join("; ");

    format!("  {kind:?} {value:?}: {locations}")
}

#[test]
fn ignored_test_line_ranges_skip_cfg_test_modules_and_test_functions() {
    let content = r#"
const KEEP: &str = "runtime_user";

#[cfg(test)]
mod tests {
    #[test]
    fn local_test() {
        let _value = ".bones/bones.toml";
    }
}

#[test]
fn top_level_test() {
    let _value = "ssh_port";
}

const ALSO_KEEP: &str = ".bones/runtime.toml";
"#;

    let ignored = ignored_test_line_ranges(content);

    assert!(ignored.iter().any(|range| range.contains(&4) && range.contains(&10)));
    assert!(ignored.iter().any(|range| range.contains(&12) && range.contains(&15)));
    assert!(!ignored.iter().any(|range| range.contains(&2)));
    assert!(!ignored.iter().any(|range| range.contains(&17)));
}

```

`tests/cleancode/src/cleancode_no_legacy_terms.rs`:

```rs
//! Integration test: detect legacy-language smell terms in Rust source.
//!
//! Flags comments, docstrings, identifiers, attributes, and string literals
//! that describe code as `legacy`, `hack`, `workaround`,
//! `backcompat`, or `deprecated`.
//!
//! This is intentionally a high-signal smell check, not a semantic analysis.

use std::fs;
use std::path::{Path, PathBuf};

const IGNORE_DIRS: &[&str] = &["target", "vendor", "node_modules", ".git"];
const LEGACY_TERMS: &[&str] = &["legacy", "hack", "workaround", "backcompat", "deprecated"];

fn collect_source_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> = vec![project_root.join("crates")];

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !IGNORE_DIRS.contains(&name.as_str()) {
                    stack.push(path);
                }
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }

    files
}

fn terms_in_line(line: &str) -> Vec<&'static str> {
    let mut matches = Vec::new();
    let lower = line.to_ascii_lowercase();

    for term in LEGACY_TERMS {
        if lower.contains(term) {
            matches.push(*term);
        }
    }
    matches
}

/// Detects legacy-language smell terms like legacy, hack, or workaround in source code.
#[test]
fn no_legacy_terms() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("cleancode crate should be two levels deep under workspace root");
    let files = collect_source_files(project_root);

    assert!(!files.is_empty(), "No source files found in crates/.");

    let mut all_violations = Vec::new();
    let mut files_scanned = 0;

    for file in &files {
        let Ok(code) = fs::read_to_string(file) else {
            continue;
        };
        files_scanned += 1;

        for (line_number, line) in code.lines().enumerate() {
            let terms = terms_in_line(line);
            if terms.is_empty() {
                continue;
            }

            let relative = file.strip_prefix(project_root).unwrap_or(file);
            all_violations.push(format!(
                "  {}:{}: {} [{}]",
                relative.display(),
                line_number + 1,
                line.trim(),
                terms.join(", ")
            ));
        }
    }

    assert!(files_scanned > 0, "No readable Rust files were found to check.");

    assert!(all_violations.is_empty(), "Legacy-language terms found:\n{}", all_violations.join("\n"));
}

```

`tests/cleancode/src/cleancode_no_literal_wrapped_fallback.rs`:

```rs
//! Integration test: detect provably unnecessary fallback patterns.
//!
//! Flags method calls like `Some(expr).unwrap_or(fallback)` or
//! `Ok(expr).unwrap_or_else(|_| fallback)` where the receiver is
//! syntactically `Some(...)` or `Ok(...)`, making the fallback dead code.
//!
//! Works via syntax analysis only — no type inference required.

use std::fs;
use std::path::{Path, PathBuf};

use syn::{
    Expr, ExprCall, ExprMethodCall,
    visit::{self, Visit},
};

const SOURCE_DIRS: &[&str] = &["src", "app", "lib"];
const IGNORE_DIRS: &[&str] = &["target", "vendor", "node_modules", ".git"];

struct FallbackVisitor {
    violations: Vec<String>,
}

impl<'ast> Visit<'ast> for FallbackVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        // Check for: Some(expr).unwrap_or(fallback), Ok(expr).or_else(fallback), etc.
        let method_name = node.method.to_string();
        let receiver_is_literal_wrapper = match node.receiver.as_ref() {
            Expr::Call(ExprCall { func, .. }) => {
                if let Expr::Path(p) = func.as_ref() {
                    let ident = p.path.get_ident().map(|i| i.to_string());
                    matches!(ident.as_deref(), Some("Some" | "Ok"))
                } else {
                    false
                }
            }
            _ => false,
        };

        if receiver_is_literal_wrapper
            && matches!(
                method_name.as_str(),
                "unwrap_or" | "unwrap_or_else" | "or" | "or_else" | "map_or" | "map_or_else"
            )
        {
            let message = if matches!(method_name.as_str(), "unwrap_or_else" | "or_else" | "map_or_else") {
                format!("call to `.{method_name}()` on `Some(…)`/`Ok(…)` — fallback closure is unreachable")
            } else {
                format!(
                    "call to `.{method_name}()` on `Some(…)`/`Ok(…)` — fallback is unnecessary and eagerly evaluated"
                )
            };
            self.violations.push(message);
        }

        visit::visit_expr_method_call(self, node);
    }
}

fn collect_source_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> =
        SOURCE_DIRS.iter().map(|dir| project_root.join(dir)).filter(|path| path.is_dir()).collect();

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !IGNORE_DIRS.contains(&name.as_str()) {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|e| e == "rs") {
                files.push(p);
            }
        }
    }

    files
}

/// Detects provably unnecessary fallback calls on `Some` or `Ok` wrappers.
#[test]
fn no_literal_wrapped_fallback() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = collect_source_files(project_root);

    assert!(!files.is_empty(), "No source files found in src/, app/, or lib/.");

    let mut all_violations = Vec::new();
    let mut files_scanned = 0;

    for file in &files {
        let Ok(code) = fs::read_to_string(file) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&code) else {
            continue;
        };
        files_scanned += 1;

        let mut visitor = FallbackVisitor { violations: Vec::new() };
        visitor.visit_file(&syntax);

        for msg in &visitor.violations {
            let relative = file.strip_prefix(project_root).unwrap_or(file);
            all_violations.push(format!("  {} — {msg}", relative.display()));
        }
    }

    assert!(files_scanned > 0, "No parsable Rust files were found to check.");

    assert!(all_violations.is_empty(), "Provably unnecessary fallback(s) found:\n{}", all_violations.join("\n"),);
}

```

`tests/cleancode/src/cleancode_no_manufactured_success.rs`:

```rs
//! Integration test: detect suspicious fallback patterns.
//!
//! Flags `match` arms on `Option` or `Result` where the failure variant
//! (`None` / `Err`) produces a success value (`Some(…)` / `Ok(…)`),
//! indicating silent error recovery (the AI anti-pattern).
//!
//! Syntax-only: works on the AST without type inference.

use std::fs;
use std::path::{Path, PathBuf};

use syn::{
    Expr, ExprCall, ExprMatch,
    visit::{self, Visit},
};

const IGNORE_DIRS: &[&str] = &["target", "vendor", "node_modules", ".git"];
const SOURCE_DIRS: &[&str] = &["src", "app", "lib"];

struct SuspiciousMatchVisitor {
    violations: Vec<String>,
}

impl<'ast> Visit<'ast> for SuspiciousMatchVisitor {
    fn visit_expr_match(&mut self, node: &'ast ExprMatch) {
        let has_success_arm = node.arms.iter().any(|arm| is_success_pattern(&arm.pat));
        let produces_success =
            node.arms.iter().any(|arm| is_failure_pattern(&arm.pat) && arm_constructs_success(arm.body.as_ref()));

        if has_success_arm && produces_success {
            self.violations.push("suspicious match arm constructs `Ok(…)`/`Some(…)` from error path".to_string());
        }

        visit::visit_expr_match(self, node);
    }
}

fn is_success_pattern(pat: &syn::Pat) -> bool {
    matches!(pat, syn::Pat::TupleStruct(pts) if pts.path.get_ident().is_some_and(|i| matches!(i.to_string().as_str(), "Ok" | "Some")))
}

fn is_failure_pattern(pat: &syn::Pat) -> bool {
    matches!(pat, syn::Pat::TupleStruct(pts) if pts.path.get_ident().is_some_and(|i| i == "Err"))
        || matches!(pat, syn::Pat::Path(pp) if pp.path.get_ident().is_some_and(|i| i == "None"))
}

fn arm_constructs_success(expr: &Expr) -> bool {
    match expr {
        Expr::Call(ExprCall { func, .. }) => {
            matches!(func.as_ref(), Expr::Path(p) if p.path.get_ident().is_some_and(|i| matches!(i.to_string().as_str(), "Ok" | "Some")))
        }
        _ => false,
    }
}

fn collect_source_files(project_root: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack: Vec<PathBuf> =
        SOURCE_DIRS.iter().map(|dir| project_root.join(dir)).filter(|path| path.is_dir()).collect();

    while let Some(dir) = stack.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                let name = entry.file_name().to_string_lossy().to_string();
                if !IGNORE_DIRS.contains(&name.as_str()) {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|e| e == "rs") {
                files.push(p);
            }
        }
    }

    files
}

/// Detects match arms that construct success values from error paths.
#[test]
fn no_suspicious_fallback() {
    let project_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let files = collect_source_files(project_root);

    assert!(!files.is_empty(), "No source files found in src/, app/, or lib/.");

    let mut all_violations = Vec::new();
    let mut files_scanned = 0;

    for file in &files {
        let Ok(code) = fs::read_to_string(file) else {
            continue;
        };
        let Ok(syntax) = syn::parse_file(&code) else {
            continue;
        };
        files_scanned += 1;

        let mut visitor = SuspiciousMatchVisitor { violations: Vec::new() };
        visitor.visit_file(&syntax);

        for msg in &visitor.violations {
            let relative = file.strip_prefix(project_root).unwrap_or(file);
            all_violations.push(format!("  {} — {msg}", relative.display()));
        }
    }

    assert!(files_scanned > 0, "No parsable Rust files were found to check.");

    assert!(all_violations.is_empty(), "Suspicious fallback(s) found:\n{}", all_violations.join("\n"),);
}

```

`tests/cleancode/src/lib.rs`:

```rs
#![forbid(unsafe_code)]

#[cfg(test)]
mod cleancode_file_too_long;
#[cfg(test)]
mod cleancode_no_duplicated_state;
#[cfg(test)]
mod cleancode_no_legacy_terms;
#[cfg(test)]
mod cleancode_no_literal_wrapped_fallback;
#[cfg(test)]
mod cleancode_no_manufactured_success;

```
