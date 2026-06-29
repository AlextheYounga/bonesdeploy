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
| `git` (deploy user) | Bare repo, release dirs, build workspace | Creates immutable release artifacts |
| `<site>` (runtime user) | Shared files, `/run/<site>`, writable paths | Mutates runtime state |
| `root` | System units, config dirs, users/groups | Provisions and restarts services |

**Key mechanics:**

- `releases/` has the setgid bit (`2750`) so group `foo-release` is inherited by new release dirs without chown.
- `shared/` is owned by the runtime user (`foo:foo 0711`) — only the app writes here.
- `build/` is private to the deploy user (`git:git 0700`) — invisible to other processes.
- `bonesremote service restart` is the only command that needs `sudo` — a narrow sudoers drop-in allows it.
- No deploy step calls `chown`, `chmod -R`, or otherwise mutates ownership after provisioning.

## Bones Scaffolding
```
.bones
├── bones.toml
├── runtime.toml
├── hooks
│   ├── hooks.sh                      # shared hook library, sourced by every hook
│   ├── post-receive
│   └── pre-push
├── deployment
│   ├── 01_install_build_deps.sh
│   └── 02_run_build.sh
```

Python infra scripts and templates are managed separately by the hidden `bonesinfra` checkout under `~/.config/bonesdeploy/_lib/bonesinfra`; see `crates/bonesdeploy/src/infra/bonesinfra.rs`.

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
```

### Hooks
Hooks are static shell scripts embedded in the `bonesdeploy` binary. They are written to `.bones/hooks/` once during `bonesdeploy init`, and they source shared functions from `.bones/hooks/hooks.sh`. After that, they belong to the user and can be edited freely. They are synced to the remote bare repo via `bonesdeploy push` and can be restored locally with `bonesdeploy pull`.

- `pre-push` => Local hook, symlinked to `.git/hooks/pre-push`. This checks to see if we are pushing to our bonesdeploy designated remote. If so, then we run `bonesdeploy doctor --local` and we fail if the doctor command expresses any warning or errors.
- `post-receive` => Resolves the configured deployment ref from stdin, then runs a single `bonesremote deploy --config "$BONES_TOML" --revision <newrev>` command. This unified command orchestrates the full pipeline: doctor → stage_release → post_receive (git checkout) → wire_release → deploy (scripts + activate) → service restart → post_deploy (prune). If the push didn't touch the configured branch, or the branch was deleted, post-receive exits without deploying.

### Deployment Folder
This folder stores deployment scripts that are run by `bonesremote deploy`. Files in this folder must be ordered sequentially like `01_install_deps.sh`, `02_run_build.sh`. They are named in numerical order and all of these scripts are always run.

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

- `runtimes/laravel/`        → Laravel (PHP + PHP-FPM)
- `runtimes/django/`         → Django (Python + Gunicorn)
- `runtimes/next/`           → Next.js (Node)
- `runtimes/nuxt/`           → Nuxt (Node)
- `runtimes/sveltekit/`     → SvelteKit (Node)
- `runtimes/vue/`           → Vue (Node)
- `runtimes/rails/`         → Rails (Ruby)

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
    - Required files exist: `bones.toml`, `hooks/hooks.sh`, `hooks/` dir, `deployment/` dir.
    - Local `pre-push` hook is symlinked properly when `deploy_on_push = true`.
  - Runs remote checks (skipped with `--local`):
    - `bonesremote` executable exists on remote and can be run globally.
    - `{project_name}.git/bones` folder exists on remote (needs `bonesdeploy push` warning)
    - `{project_name}.git/bones/hooks` matches with `{project_name}.git/hooks` inside the remote bare repo when `deploy_on_push = true`.
  - The `--local` flag skips all remote checks. The `pre-push` hook uses this flag because it is only a local guard before optional git-triggered deploys.

- **push**
  - Uses an `rsync -av --delete` command to push up our local `.bones` folder to the bare repo.
  - We will create a `bones` folder under our `{project_name}.git/` folder so that everything is self-contained inside git.
  - Deletes sample git hooks in bare repo, so that our files will be the only files to worry about in the `{project_name}.git/hooks` folder.
  - Runs commands on remote that symlinks our `{project_name}.git/bones/hooks` files are symlinked with `{project_name}.git/hooks` properly.

- **pull**
  - Uses an `rsync -av --delete` command to pull the remote `{project_name}.git/bones/` folder back into local `.bones/`.
  - Recreates the local `.git/hooks/pre-push` symlink so the repository regains its pre-push check after recovery.

- **deploy**
  - SSHes into the configured host and runs `bonesremote deploy --config <remote_bones_toml>` directly, without pushing commits or using git hooks.
  - Omits the `--revision` flag, so `bonesremote deploy` uses the configured branch from `bones.toml`.

- ****remote setup****
  - Delegates to the hidden `bonesinfra` checkout by running `python -m bonesinfra setup apply --config <path>` against the configured host as root (or `BONES_BOOTSTRAP_SSH_USER`).
  - Passes `bones.toml` deployment values plus computed paths and variables as JSON on stdin.
  - Initializes bare git repository at `repo_path`.
  - Creates initial placeholder release with default page.
  - Installs `bonesremote` from source and runs `bonesremote init`.
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
  - SSHes into the configured host and runs `bonesremote release rollback --config ...`, which repoints `current` to the previous release without rebuilding.

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
  - Orchestrates: doctor → stage_release → post_receive (git checkout) → wire_release → deploy scripts → activate → service restart → post_deploy.
  - On failure, automatically drops the staged release.
  - `--config <path>`: path to `bones.toml`
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
- Sudo access for the deployment user is strictly limited to passwordless execution of `bonesremote service restart --config *` via the `/etc/sudoers.d/bonesdeploy` drop-in installed by `bonesremote init`.
- No broader sudo privileges are granted — the deploy user cannot run arbitrary commands as root, read root-owned files, or write outside their owned directories.
- All release artifacts are created with the setgid bit on `releases/` so the runtime group inherits read access without needing a post-deploy chown.
- The build workspace (`build/`) is private to the deploy user (`0700`), invisible to other processes.
- Runtime processes are sandboxed via systemd `ProtectSystem=strict`, `NoNewPrivileges=yes`, `PrivateTmp=yes`, and AppArmor profiles — limiting blast radius even if a service is compromised.
- Per-project systemd services run as the dedicated runtime user, not a shared `www-data` — so service isolation is enforced at the OS level, not just the application level.

## Flow
- User runs `bonesdeploy init`, and the procedures outlined above are executed.
- User can make any changes to their deployment scripts or hooks in `.bones/` (e.g., customizing `deployment/` files or adding project-specific logic).
- User runs `bonesdeploy push` to sync the `.bones/` folder to the remote bare repo.
- User runs `bonesdeploy deploy` to perform the actual remote release deployment.

### Primary Deploy Flow

1. `bonesdeploy deploy` SSHes into the configured host.
2. It runs `bonesremote deploy --config <remote_bones_toml>`.
3. `bonesremote deploy` orchestrates the full pipeline:
   - **doctor** — Check server environment
   - **stage_release** — Create timestamped release dir, ensure build workspace
   - **post_receive** — `git checkout -f <branch>` into `build/workspace`
   - **wire_release** — Symlink shared paths from `runtime.toml` into workspace
   - **deploy** (inner) — Run deployment scripts, copy to release, activate symlink
   - **restart_services** — `sudo bonesremote service restart --config ...`
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
     bonesremote deploy --config "$BONES_TOML" --revision <newrev>
     ```
   - This command orchestrates the full pipeline:
     - **doctor** — Check server environment
     - **stage_release** — Create timestamped release dir, ensure build workspace
     - **post_receive** — `git checkout -f <branch>` into `build/workspace`
     - **wire_release** — Symlink shared paths from `runtime.toml` into workspace
     - **deploy** (inner) — Run deployment scripts, copy to release, activate symlink
     - **restart_services** — `sudo bonesremote service restart --config ...`
     - **post_deploy** — Prune old releases beyond `releases`
     - On failure: **drop_failed_release** — Clean up staged release

`bonesdeploy deploy` performs the same remote pipeline by SSHing into the host and running `bonesremote deploy --config <remote_bones_toml>` directly (without `--revision`, so it uses the configured branch). Git-triggered deploy is optional plumbing, not the primary model.
