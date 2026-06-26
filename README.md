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
```

### Server (bonesremote)

```sh
sudo cargo install --locked --root /usr/local --git https://github.com/AlextheYounga/bonesdeploy.git bonesremote --force
```

Then run the remote setup:

```sh
sudo bonesremote init
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
2. Prompt for project name, branch, remote name, host, and port
3. Add `.bones` to `.gitignore`
4. Symlink the `pre-push` hook into `.git/hooks/`
5. Create a local deployment git remote if needed

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

This SSHs into the host and runs `bonesremote deploy --site <project>`, which orchestrates the current server-side pipeline: stage release → export source from the bare repo into a temp build context → run deployment scripts → promote the release → wire shared paths → activate → restart services → prune old releases.

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
    └── 01_*.sh          # deployment scripts (run sequentially)
```

Hooks are written to `.bones/hooks/` once during init. `pre-push` is now a self-contained guard; remote `post-receive` is a thin trigger that delegates to `sudo bonesremote hook post-receive --site <project>`. After that they belong to you — edit freely. Deployment scripts in `.bones/deployment/` must be numbered (e.g. `01_install_deps.sh`, `02_build.sh`) and are always run in order.

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
