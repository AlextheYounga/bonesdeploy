# BonesDeploy v3

A Rust CLI that compiles into a single binary, containing embeds of boilerplate scripts along with other git remote helpers. It produces two executables: `bonesdeploy` (local CLI for setup and management) and `bonesremote` (server-side tool for remote operations, installed on the deployment host). This is designed to run on a fresh server or VPS similar to Coolify. **We only handle Debian/Ubuntu machines.**

We keep detailed documentation of each command at: `docs/commands/*.md:`

## Deployment Methodology
We have an SSH deployment user (normally `git`) that handles deployment concerns. This user has a home folder, restricted sudo ability, but no password login. We also have a per-project service user named after the project. This is not a shared `applications` user; it must be a dedicated user per project so isolation works on a shared server. This user has no home folder, no login, and no sudo ability. This is ultimately who we want to own our project files to limit attack scope.

### Just-in-Time Concerns
This project should prefer just-in-time mutations.

A concern should only be handled at the last responsible moment: immediately before the system would fail if that mutation did not occur. We should not widen permissions, rewrite symlinks, mutate shared state, or otherwise touch live project state early just because a later step might need it. The idea here is to limit the surface of attack time, so that potential vulnerabilities are not created by "jumping the gun" to solve a problem too early, long before it arises.

This principle exists to keep deployment behavior coherent and safe:

- `pre-receive` should validate and prepare isolated staging state, not mutate live state.
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
.bones
├── bones.yaml
├── runtime.yaml
├── hooks
│   ├── hooks.sh                      # shared hook library, sourced by every hook
│   ├── post-receive
│   ├── pre-push
│   └── pre-receive
├── deployment
│   ├── 01_run_deployment_concerns.sh
│   └── 02_permissions_lockup.sh (example)
├── infra                             # pyinfra deploy scripts used by `bonesdeploy remote setup|runtime|ssl`
│   ├── setup.py                      # machine bootstrap (users, git repo, firewall, bonesremote)
│   ├── runtime.py                    # per-site runtime (AppArmor, nginx, systemd services)
│   ├── ssl.py                        # TLS certificate provisioning via certbot
│   └── assets/
│       ├── apparmor/
│       │   └── project-nginx-profile.j2
│       └── nginx/
│           ├── router.conf.j2
│           ├── site-nginx.conf.j2
│           └── site-nginx.service.j2
└── runtime                           # template-specific operations used by `bonesdeploy remote runtime`
    └── operations.py

### Bones YAML
This stores crucial data we will need and is collected on running `bonesdeploy init` via user prompts.  
Collects the following project information from the user:
- `project_name`: str
- `branch`: str
- `remote_name`: existing remote selection when available, otherwise prompted; defaults to `production`. Must point to a fresh VPS, not a code host like GitHub.
- `host`: prompted when not inferable from selected remote
- `port`: defaults to `22`, prompt shown when remote inference is unavailable
- `repo_path`: inferred from selected remote URL when possible, else defaults to `/home/git/{project_name}.git`

Everything else is defaulted for Debian/Ubuntu-first usability:
- `project_root`: defaults to `/srv/sites/{project_name}`
- `web_root`: defaults to `public`
- `deploy_on_push`: defaults to `true`
- `deploy_user`: defaults to `git`
- `runtime_user`: defaults to `{project_name}`
- `runtime_group`: defaults to `{project_name}`
- `release_group`: defaults to `{project_name}-release` and gets setgid on `releases/` for inherited group access
- `releases.keep`: defaults to `5`

Users can override any default by editing `.bones/bones.yaml` after init.

Example `bones.yaml`:
```yaml
data:
  remote_name: "production"
  project_name: "lawsnipe"
  host: "deploy.example.com"
  port: "22"
  repo_path: "/home/git/lawsnipe.git"
  project_root: "/srv/sites/lawsnipe"
  web_root: "public"
  branch: "master"
  deploy_on_push: true
  deploy_user: "git"
  runtime_user: "lawsnipe"
  runtime_group: "lawsnipe"
  release_group: "lawsnipe-release"

releases:
  keep: 5

ssl:
  enabled: true
  domain: "app.example.com"
  email: "ops@example.com"
```

### Hooks
Hooks are static shell scripts embedded in the `bonesdeploy` binary. They are written to `.bones/hooks/` once during `bonesdeploy init`, and they source shared functions from `.bones/hooks/hooks.sh`. After that, they belong to the user and can be edited freely. They are synced to the remote bare repo via `bonesdeploy push` and can be restored locally with `bonesdeploy pull`.

- `pre-push` => Local hook, symlinked to `.git/hooks/pre-push`. This checks to see if we are pushing to our bonesdeploy designated remote. If so, then we run `bonesdeploy doctor --local` and we fail if the doctor command expresses any warning or errors.
- `pre-receive` => Short-circuits when `deploy_on_push = false`. Otherwise it resolves the configured deployment branch from stdin's pushed refs (skipping deletes and pushes to other branches), then runs `bonesremote doctor` and `bonesremote release stage --config ...` to prepare build and release directories and write staged release state.
- `post-receive` => Re-resolves the deployment ref, then runs the full deployment pipeline by calling, in order: `bonesremote hooks post-receive --config ... --revision <newrev>` (checkout into `build/workspace`), `bonesremote release wire --config ...` (just-in-time shared file and directory wiring), `bonesremote hooks deploy --config ...`, and `bonesremote hooks post-deploy --config ...`, then `sudo bonesremote service restart --config ...` to restart the per-site nginx service.

### Deployment Folder
This folder stores deployment scripts that are run by `bonesremote hooks deploy`. Files in this folder must be ordered sequentially like `01_install_deps.sh`, `02_run_build.sh`. They are named in numerical order and all of these scripts are always run.

## Crate Structure
This Cargo workspace has three crates under `crates/`:
- `bonesdeploy` for the local CLI binary
- `bonesremote` for the server-side binary
- `shared` for code that must be common to both binaries

### Path Centralization
All product-owned paths must live in `crates/shared/src/paths.rs`.

Other modules may derive subpaths by joining values from `shared::paths`, but they must not introduce their own independent path roots, filenames, or install locations.

This applies to Rust code, pyinfra deploy scripts, Jinja2 templates, and docs examples that describe the system layout.

```
bonesdeploy/
├── Cargo.toml                  # workspace root
├── kit/                        # embedded assets (scaffolding templates)
│   ├── bones.yaml
│   ├── hooks/
│   │   ├── hooks.sh
│   │   ├── post-receive
│   │   ├── pre-push
│   │   └── pre-receive
│   ├── deployment/
│   └── setup/                  # nginx + pyinfra deploy scripts for shared server bootstrap
│       ├── apparmor/
│       ├── nginx/
│       ├── setup.py
│       ├── runtime.py
│       └── ssl.py
├── templates/                  # per-framework starter overlays (see below)
│   ├── laravel/
│   │   ├── bones.yaml
│   │   ├── deployment/
│   │   └── runtime/
│   └── ...
├── crates/
│   ├── bonesdeploy/            # local CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs         # clap setup, command dispatch
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── init.rs
│   │       │   ├── doctor.rs
│   │       │   ├── push.rs
│   │       │   ├── deploy.rs
│   │       │   ├── rollback.rs
│   │       │   ├── remote_setup.rs
│   │       │   ├── remote_ssl.rs
│   │       │   └── version.rs
│   │       ├── config.rs       # bones.yaml structs + load/save + local file discovery
│   │       ├── embedded.rs     # rust-embed from kit/, scaffold writing
│   │       ├── git.rs          # git CLI operations: remote validation, repo checks
│   │       ├── prompts.rs      # interactive user input collection, returns config
│   │       └── ssh.rs          # openssh session management + rsync
│   └── bonesremote/            # server-side binary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── commands/
│           │   ├── mod.rs
│           │   ├── init.rs
│           │   ├── doctor.rs
│           │   ├── stage_release.rs
│           │   ├── wire_release.rs
│           │   ├── activate_release.rs
│           │   ├── drop_failed_release.rs
│           │   ├── rollback.rs
│           │   ├── post_receive.rs
│           │   ├── deploy.rs
│           │   ├── post_deploy.rs
│           │   └── version.rs
│           ├── config.rs       # bones.yaml structs + remote file discovery
│           ├── privileges.rs   # sudoers drop-in install + privilege checks
│           ├── privileges.rs   # sudoers drop-in install + privilege checks
│           └── release_state.rs # staged-release state file read/write
└── docs/
```

### Per-Framework Templates
Runtime templates ship starter overlays that `bonesdeploy remote runtime` uses when scaffolding infrastructure for a matching framework. Each template lives under `crates/bonesdeploy/embeds/runtimes/` — framework runtime assets (`operations.py`, Jinja2 templates) stay together:

- `runtimes/laravel/`        → Laravel (PHP + PHP-FPM)

Templates inherit the same `bones.yaml` schema and only customize permissions paths, deployment scripts, and the runtime operations captured in `.bones/runtime/operations.py`.

### BonesDeploy CLI Commands
- **init**:
  - Gets or creates the `.bones` folder with our default scaffolding.
  - Updates `.gitignore` to add .bones folder.
  - Loads existing config from `.bones/bones.yaml` or collects user input via prompts.
  - Creates local deployment remote if missing using `{deploy_user}@{host}:{repo_path}`, constructed from the production VPS target configured during prompts.
  - Prints next-step guidance to run `bonesdeploy remote setup` and `bonesdeploy remote runtime` before first deploy.
  - Saves config to `.bones/bones.yaml`.

- **doctor**
  - This command checks all concerns in your local environment.
  - Loads config from `.bones/bones.yaml`
  - Runs local checks:
    - `.bones` folder is set up correctly. Deployment scripts are named appropriately.
    - Local `pre-push` hook is symlinked properly.
  - Runs remote checks (skipped with `--local`):
    - `bonesremote` executable exists on remote and can be run globally.
    - `{project_name}.git/bones` folder exists on remote (needs `bonesdeploy push` warning)
    - `{project_name}.git/bones/hooks` matches with `{project_name}.git/hooks` inside the remote bare repo.
  - The `--local` flag skips all remote checks. The `pre-push` hook uses this flag since the remote is independently validated by `bonesremote doctor` in the `pre-receive` hook.

- **push**
  - Uses an `rsync -av --delete` command to push up our local `.bones` folder to the bare repo.
  - We will create a `bones` folder under our `{project_name}.git/` folder so that everything is self-contained inside git.
  - Deletes sample git hooks in bare repo, so that our files will be the only files to worry about in the `{project_name}.git/hooks` folder.
  - Runs commands on remote that symlinks our `{project_name}.git/bones/hooks` files are symlinked with `{project_name}.git/hooks` properly.

- **pull**
  - Uses an `rsync -av --delete` command to pull the remote `{project_name}.git/bones/` folder back into local `.bones/`.
  - Recreates the local `.git/hooks/pre-push` symlink so the repository regains its pre-push check after recovery.

- **deploy**
  - Manually runs remote `pre-receive` and `post-receive` hooks over SSH without pushing commits.
  - Sets `BONES_FORCE_DEPLOY=1` so manual deploy runs even when `deploy_on_push = false`.

- ****remote setup****
  - Runs `.bones/infra/setup.py` via `pyinfra` against the configured host as root (or `BONES_BOOTSTRAP_SSH_USER`).
  - Passes `bones.yaml` deployment values plus computed paths and variables as pyinfra data.
  - Initializes bare git repository at `repo_path`.
  - Creates initial placeholder release with default page.
  - Installs `bonesremote` from source and runs `bonesremote init`.
  - Provisions machine-level dependencies (users, groups, firewall, system packages).

- **remote runtime**:
  - Prompts for a framework template, refreshes `.bones/runtime/`, and writes `.bones/runtime.yaml`.
  - Reapplies template-specific defaults into `.bones/bones.yaml` only when they still match generic or previous-template values.
  - After a `y/N` confirmation, runs `.bones/infra/runtime.py` via `pyinfra` against the configured host as the deploy user.
  - Loads the template's `operations.py` at runtime to install framework-specific packages and services.
  - Configures per-site runtime assets: AppArmor profile, nginx router + per-site config + systemd service, and runs `bonesremote doctor`.
  - Does not handle SSL; use `remote ssl` for TLS configuration.

- **remote ssl**
  - Runs `.bones/infra/ssl.py` via `pyinfra` against the configured host as root.
  - Uses certbot with a webroot challenge to obtain/renew certificates for the configured domain.
  - Re-renders the per-site runtime nginx router with TLS enabled, listening on 443 and redirecting HTTP to HTTPS.
  - Separate from `remote runtime` to keep certificate management decoupled from app runtime concerns.

- **rollback**
  - SSHes into the configured host and runs `bonesremote release rollback --config ...`, which repoints `current` to the previous release without rebuilding.

- **version**:
  - Echoes "bonesdeploy 0.1.0".

### BonesDeployRemote CLI Commands
- **Release commands** live under `bonesremote release ...`
- **Hook commands** live under `bonesremote hooks ...`
- **Service commands** live under `bonesremote service ...`
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
- **hooks post-receive**
	- Checks out the resolved revision (or the configured branch if `--revision` is omitted) into `build/workspace`.
- **hooks deploy**
	- Runs deployment scripts in `build/workspace` (with shared paths already wired), copies release-ready output into staged `releases/<timestamp>`, drops failed staged releases on error, and activates release on success.
- **hooks post-deploy**
	- Prunes old releases beyond the configured `releases.keep` count. Does not mutate ownership or permissions — those are established at provisioning time.
- **service restart**
	- Restarts the per-site nginx systemd service. This is the only `bonesremote` command that requires root privileges.
- **version**:
  - Echoes "bonesdeploy 0.1.0".

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
- User runs `git push production master` or some similar command where the remote name aligns with our bones.yaml configuration.
- The `pre-push` hook checks to see if we are pushing to our bones remote (in this example, `production`). If so, it runs `bonesdeploy doctor` and fails on warnings/errors.

### Hook Event Order on `git push`

`pre-push -> pre-receive -> post-receive`

1. Git receives pack data in the remote bare repo and runs `pre-receive`.
2. If `deploy_on_push = false`, `pre-receive` exits early and no deploy steps run.
3. Otherwise `pre-receive` resolves the pushed deployment ref:
   - If the configured branch is not in the pushed refs, or the push deletes it, `pre-receive` exits successfully without staging.
   - If the configured branch was pushed, `pre-receive` runs `bonesremote doctor`, then `bonesremote release stage --config "$BONES_YAML"`.
4. If `pre-receive` exits successfully, Git updates refs.
5. Git runs `post-receive`, which re-resolves the deployment ref the same way.
6. `post-receive` runs `bonesremote hooks post-receive --config "$BONES_YAML" --revision <newrev>` (checkout into `build/workspace`).
7. `post-receive` runs `bonesremote release wire --config "$BONES_YAML"` (just-in-time symlinking shared paths into the build workspace).
8. `post-receive` runs `bonesremote hooks deploy --config "$BONES_YAML"` (deployment scripts in build workspace + copy to release + activate/drop-failed).
9. `post-receive` runs `bonesremote hooks post-deploy --config "$BONES_YAML"` (pruning old releases).
10. `post-receive` runs `sudo bonesremote service restart --config "$BONES_YAML"` (restart the per-site nginx).

`bonesdeploy deploy` re-runs the same remote pipeline by setting `BONES_FORCE_DEPLOY=1` and invoking `pre-receive` and `post-receive` over SSH. The force flag bypasses the `deploy_on_push` short-circuit and resolves the deployment ref via `git rev-parse` instead of stdin.
