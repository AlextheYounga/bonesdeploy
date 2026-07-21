# BonesDeploy v3

A remote release deployment tool for simple Linux servers. It produces two executables: `bonesdeploy` (local CLI for setup, provisioning, deployment, and management) and `bonesremote` (server-side release lifecycle executor, installed on the deployment host). Git remains supported as an optional trigger, but it does not own the deployment model. **We only handle Debian/Ubuntu machines.**

The command behavior is documented in this file and in the command examples in `README.md`.

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

- `releases/` contains candidates owned by the runtime user while prepare runs, then sealed as `root:<site>` before activation.
- `shared/` is owned by the runtime user (`<site>:<site>`) — only the app writes here.
- Build input is temporary and disposable; build scripts run in Podman with the source mounted at `/workspace/source`.
- Prepare scripts run as the runtime user after shared paths are wired and before `current` is repointed.
- Git hooks only trigger `bonesremote`; they do not check out source, run builds, write releases, or restart services.
- `bonesremote` is the privileged mediator for promotion, activation, and service restart.

### Release Visibility and Cancellation

`bonesdeploy releases` asks `bonesremote` for the site's release state and renders the returned JSON locally; it stores no release state on the workstation. Releases are `active`, `previous`, `building`, `preparing`, or `interrupted`. A `building` or `interrupted` release can be cancelled with `bonesdeploy releases kill <release>`; cancellation removes only that release's build container, temporary context, staged-release state, and transient deployment metadata.

BonesRemote holds one OS-backed deployment lock per site. Deploys, cancellations, and site imports use the same stable lock, which lives outside the replaceable site dataset. A deploy or import must not stage or overwrite state while a release is building, preparing, or interrupted. Before staging, BonesRemote starts and verifies the build user's systemd manager and checks rootless Podman readiness. A damaged rootless Podman namespace is reported before any release state is created; deploy does not silently reset Podman because that operation stops the build user's containers.

## Bones Scaffolding
```
.bones
├── bones.toml
├── deployment
│   ├── build/
│   │   ├── 01_install_build_deps.sh
│   │   └── 02_run_build.sh
│   └── prepare/
│       └── 01_prepare.sh
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

`[runtime]` in `.bones/bones.toml` contains the selected template, web root, runtime identity, permissions, and shared paths. Those identity values default to `{project_name}`, `{project_name}`, and `{project_name}-release` respectively. Shared paths are declared under `[runtime.shared].paths`; deploys only wire the paths listed there, so framework-specific writable paths must not be hardcoded globally.

Users can override any default by editing `.bones/bones.toml` after init.

`[dbs].services` is selected during init (or with repeated non-interactive `--db` flags). Supported values are `postgres`, `mariadb`, `mysql`, `mongodb`, `valkey`, and `redis`. Database provisioning binds every listener to localhost, generates credentials on the host, and writes connection values only to the protected `shared/.env`. Remote workstation access uses ordinary SSH port forwarding; no tunnel information is stored. MariaDB and MySQL are mutually exclusive server implementations.

Example `bones.toml`:
```toml
[app]
remote_name = "production"
project_name = "lawsnipe"
repo_path = "/home/git/lawsnipe.git"
project_root = "/srv/sites/lawsnipe"

[app.server]
ssh_user = "root"
host = "deploy.example.com"
port = "22"

[app.dns]
preview_domain = "lawsnipe-deploy-example-com.nip.io"
domain = "app.example.com"
email = "ops@example.com"
ssl_enabled = true

[app.deploy]
branch = "master"
deploy_on_push = false
releases = 5

[build]
vars = ["NEXT_PUBLIC_API_URL"]

[runtime]
template = "next"
web_root = "public"
```

### Build-time configuration
`[build]` in `.bones/bones.toml` declares which environment variables should be injected into the build container at build time. It supports two sources:

1. **`vars`** — names of environment variables from `shared/.env` (secrets). Values come from `bonesdeploy secrets push`, not this file.
2. **Any other key under `[build]`** — literal (non-secret) value injected as a same-named environment variable, e.g. `BUILD_TARGET = "production"` is exposed as `$BUILD_TARGET`. These are build-time configuration constants, not secrets.

```toml
# .bones/bones.toml
[build]
# `vars` pulls from shared/.env — never put secret values here.
vars = [
  "NEXT_PUBLIC_API_URL",
  "NEXT_PUBLIC_GA_ID",
]

# Keys under [build] (except `vars`) are injected directly as env vars.
# Non-secret, safe to commit, e.g. $BUILD_TARGET in build scripts.
BUILD_TARGET = "production"
```

The file is pushed with `bonesdeploy push` and read by `bonesremote` during the build phase. It first injects literal keys into the build container, then overlays the `vars` values from `shared/.env` on top. If a name appears in both places, the value from `.env` wins.

Top-level keys are for **non-secret build constants** (e.g. toolchain versions, feature flags). They are committed to version control and visible in plaintext. Putting secret values at the top level would expose them to the build container, the repository, and potentially inlined client bundles — use `vars` and `bonesdeploy secrets push` instead.

### Hooks
The optional git push transport uses two thin internal adapters (local `pre-push` guard and remote `post-receive` trigger) that are embedded in the binaries. They are not visible or editable under `.bones/`. Set `deploy_on_push = true` in `.bones/bones.toml` to enable git-triggered deploys.

- `pre-push` => Installed by `bonesdeploy init` into `.git/hooks/pre-push`. This checks if we are pushing to the bonesdeploy designated remote. If so, it runs `bonesdeploy doctor --local` and fails if doctor reports warnings or errors.
- `post-receive` => Installed automatically into the bare repo. Derives `<site>` from `GIT_DIR` and runs `sudo bonesremote hook post-receive --site <site>`. `bonesremote` then reads branch policy and config from `/root/.config/bonesremote/sites/<site>/`. The canonical script is embedded in the `bonesremote` binary and installed as a side-effect of `bonesdeploy push`.

### Deployment Folder
This folder stores build and prepare scripts that are published into bonesremote site state. Build scripts live in `.bones/deployment/build/`, must use the `NN_name.sh` convention (for example, `01_install_deps.sh`, `02_run_build.sh`), and run in lexical order inside bonesremote's `buildpack-deps:bookworm` container with `cwd=/workspace/source`; other files, including `README.md`, are ignored. Bonesremote prepares the image and executes scripts through the build user's systemd user manager with `systemd-run --machine=<site>-build@ --user`, rather than changing UID with `runuser`. The long-lived build container is a transient user service that tracks Podman's monitor process, while each script still streams its output through foreground `podman exec`. Before scripts run, Bonesremote streams the deployment bundle into the container's disposable filesystem at `/workspace/deployment`; it does not bind-mount the root-owned control-plane path. The build container receives the exported source tree and private persistent build cache at `/workspace/cache`; it does not receive `.env`, `shared/`, `current`, `releases/`, the bare repo, or host bonesremote control-plane files. The cache is provisioned by BonesInfra at `/var/lib/bonesdeploy/users/<site>-build/cache` and is used only for tool and package downloads. Prepare scripts live in `.bones/deployment/prepare/`, use the same naming convention, run in lexical order as the site runtime user with `cwd` set to a runtime-owned candidate release, and are the right place for migrations, cache warmups, and other runtime-state work. For each prepare script, Bonesremote opens the root-owned shared `functions.sh` and script, then streams both as one stdin input to the runtime-user shell; the runtime user receives no filesystem access to the deployment bundle. Before prepare scripts run, `bonesremote` wires each `[runtime.shared].paths` entry into the candidate; after prepare succeeds, it seals the release before activation.

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
│   │   ├── kit/                # embedded scaffolding templates
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
│   │       ├── privileges.rs   # privilege checks for root-only commands
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
  - Loads existing config from `.bones/bones.toml` or collects user input via prompts.
  - For fresh init, waits until prompts complete before creating `.config/bonesdeploy/<project>.bones/` and the local `.bones` symlink.
  - Updates `.gitignore` to add .bones folder.
  - Creates local deployment remote if missing using `{deploy_user}@{host}:{repo_path}`, constructed from the production VPS target configured during prompts.
  - Prints next-step guidance to run `bonesdeploy remote setup` and `bonesdeploy remote runtime` before first deploy.
  - Saves config to `.bones/bones.toml`.
  - Runtime template selection and per-template questions are sourced from `crates/bonesdeploy/src/runtimes/<fw>.rs` (typed Rust, embedded in the binary). `init` no longer calls `bonesinfra runtime questions` or prefetches `bonesinfra`.
  - `--template <name>` selects a runtime template non-interactively. `--runtime-var <key=value>` (repeated) overrides template variables; answers are validated against the template's question schema before writing `bones.toml`.

- **doctor**
  - This command checks all concerns in your local environment.
  - Checks are reported as pass, pending, or failure. A pending first Git push is expected after remote setup and exits successfully; broken prerequisites still exit non-zero.
  - Loads config from `.bones/bones.toml`
  - Runs local checks:
    - `.bones` folder exists and is a symlink (warns if it is not a symlink to `~/.config/bonesdeploy/<project>.bones/`).
    - Deployment scripts under `.bones/deployment/build/` and `.bones/deployment/prepare/` are ordered with numeric prefixes.
    - Local `pre-push` guard is installed properly when `deploy_on_push = true`. Checks for the presence and version marker in the baked script.
  - Runs remote checks (skipped with `--local`):
    - Opens a privileged SSH session and runs `bonesremote doctor --site <project>`.
    - `bonesremote doctor --site <project>` checks Podman availability, deploy-user sudo wiring, AppArmor availability, imported control-plane state under `/root/.config/bonesremote/sites/<project>/`, the build user's existence and home, the bare repo and thin `post-receive` hook, runtime user/group constraints, `shared/` and `releases/` layout, and `<project>-nginx.service`. An empty bare repo is reported as pending until the configured branch is pushed.
  - The `--local` flag skips all remote checks. The `pre-push` hook uses this flag because it is only a local guard before optional git-triggered deploys.

- **push**
  - Archives the local `.bones/` dataset, excluding local secrets, and streams it to `bonesremote site import --site <project>` over SSH.
  - `bonesremote` validates the dataset and atomically replaces the current remote site state under `/root/.config/bonesremote/sites/<project>/`.
  - The bare repo is no longer the control-plane storage target for `push`.

- **pull**
  - Streams the current remote site dataset back from `bonesremote site export --site <project>` and extracts it into local `.bones/`.
  - Re-installs the local pre-push guard so the repository regains its pre-push check after recovery.

- **deploy**
  - Publishes the local `.bones/` dataset into remote bonesremote site state first, then SSHes into the configured host and runs `bonesremote deploy --site <project>` directly.
  - Omits the `--revision` flag, so `bonesremote deploy` uses the configured branch from `bones.toml`.

- ****remote setup****
  - Delegates to the hidden `bonesinfra` checkout by running `python -m bonesinfra setup apply --config <path>` against the configured host as root (or `BONES_BOOTSTRAP_SSH_USER`).
  - Passes `bones.toml` deployment values plus computed paths and variables as JSON on stdin.
  - Initializes bare git repository at `repo_path`.
  - Creates initial placeholder release with default page.
  - Installs `bonesremote` from source.
  - Installs the deploy-user sudoers policy through `bonesinfra` host provisioning.
  - Provisions machine-level dependencies (users, groups, firewall, system packages).

- **remote runtime**:
  - Prompts for a framework template, refreshes `.bones/runtime/`, and writes the selected settings into `.bones/bones.toml`.
  - Reapplies template-specific defaults into `.bones/bones.toml` only when they still match generic or previous-template values.
  - After a `y/N` confirmation, delegates to the hidden `bonesinfra` checkout by running `python -m bonesinfra runtime apply --config <path> --runtime-config <path>` against the configured host as the configured `ssh_user`.
  - Loads the template's `operations.py` at runtime to install framework-specific packages and services.
  - Configures per-site runtime assets: AppArmor profile, nginx router + per-site config + systemd service, and runs `bonesremote doctor`.
  - Does not handle SSL; use `remote ssl` for TLS configuration.

- **remote dbs**:
  - Provisions the services selected in `[dbs]`; `bonesdeploy setup` runs this after bootstrap.
  - Keeps all database listeners loopback-only and does not publish credentials into the remote control-plane dataset.

- **remote ssl**
  - Delegates to the hidden `bonesinfra` checkout by running `python -m bonesinfra ssl apply --config <path>` against the configured host as root.
  - Uses certbot with a webroot challenge to obtain/renew certificates for the configured domain.
  - Re-renders the per-site runtime nginx router with TLS enabled, listening on 443 and redirecting HTTP to HTTPS.
  - Separate from `remote runtime` to keep certificate management decoupled from app runtime concerns.

- **rollback**
  - SSHes into the configured host and runs `bonesremote release rollback --site <project>`, which repoints `current` to the previous release without rebuilding and restarts `<project>.target`.

- **secrets**
  - Subcommands: `init`, `edit`, `push`.
  - Manages GPG-encrypted environment secrets under `.bones/secrets/`.
  - `secrets init` bootstraps the `.bones/secrets/` directory and GPG recipients.
  - `secrets edit` decrypts `.bones/secrets/.env.gpg` for editing and re-encrypts on save.
  - `secrets push` uploads the decrypted `.env` to the remote `shared/.env` over SSH.

- **config**
  - Reads or prints values from `.bones/bones.toml`.
  - `--file <path>` overrides the config file location (defaults to `.bones/bones.toml`).
  - `<key>` prints a single value when supplied; when omitted, dumps the whole file.

- **skill**
  - Embedded documentation for AI agents, plus the state-aware next-step compass.
  - `bonesdeploy skill` prints the orientation doc (`SKILL.md`) baked into the binary.
  - `bonesdeploy skill list` prints the names of every embedded topic doc.
  - `bonesdeploy skill doc <name>` prints a specific topic doc (`commands`, `workflows`, `methodology`).
  - `bonesdeploy skill next [--format text|json]` supersedes `guide` and inspects `.bones/bones.toml` and the remote host, then suggests the next prompt-free command. `--format json` returns the same `Report` struct `status` consumes. The hidden `guide` command remains as a compatibility alias.
  - Topic docs are markdown files under `crates/bonesdeploy/skill/` and are embedded with `rust-embed` alongside `kit/` and `runtimes/`.
- **version**:
  - Echoes the installed `bonesdeploy` version.

### BonesRemote CLI Commands
- **Release commands** live under `bonesremote release ...`
- **Service commands** live under `bonesremote service ...`
- **deploy**:
  - Runs the full deployment lifecycle as a single command (the primary entrypoint used by both `post-receive` hook and `bonesdeploy deploy`).
  - Orchestrates: stage release → source export from the bare repo into a temp build context → build scripts → runtime-writable candidate release → shared wiring → prepare scripts as the site user → seal release → activate → restart `<site>.target` → post-deploy pruning.
  - On failure before activation, automatically drops the staged release. If the service restart fails after activation,
    restores and restarts the previous release before dropping the failed release.
  - `--site <name>`: imported site identifier used to load root-owned registry state
  - `--revision <rev>`: optional exact commit to check out; defaults to configured branch
- **doctor**:
  - Host mode checks `bonesremote` in `PATH`, Podman, AppArmor support, and the deploy-user sudoers drop-in.
  - `--site <name>` also checks the imported site boundary: validated control-plane state, bare repo and thin hook, runtime identity constraints, `shared/` and `releases/` layout, and `<site>.target`.
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
	- Restarts the per-site systemd lifecycle target (`<project>.target`), which restarts all registered site services. This is the only `bonesremote` command that requires root privileges.

BonesInfra owns site service membership. BonesRemote restarts exactly `<project>.target` for deploy and rollback.
- **version**:
  - Echoes the installed `bonesremote` version.

## Security Notes
- Sudo access for the deployment user is strictly limited by the `/etc/sudoers.d/bonesdeploy` drop-in provisioned by `bonesinfra` on the host.
- No broader sudo privileges are granted — the deploy user cannot run arbitrary commands as root, read root-owned files, or write outside their owned directories.
- All release artifacts are created with the setgid bit on `releases/` so the runtime group inherits read access without needing a post-deploy chown.
- The build workspace (`build/`) is private to the deploy user (`0700`), invisible to other processes.
- Runtime processes are sandboxed via systemd `ProtectSystem=strict`, `NoNewPrivileges=yes`, `PrivateTmp=yes`, and AppArmor profiles — limiting blast radius even if a service is compromised.
- Per-project systemd services run as the dedicated runtime user, not a shared `www-data` — so service isolation is enforced at the OS level, not just the application level.

## Flow
- User runs `bonesdeploy init`, and the procedures outlined above are executed.
- User can make any changes to their deployment scripts in `.bones/` (e.g., customizing `deployment/build/` files or adding project-specific logic).
- User runs `bonesdeploy push` to publish the `.bones/` dataset to bonesremote site state under `/root/.config/bonesremote/sites/<site>/`.
- Before the first deploy (and after initial setup), the source code must be pushed to the remote bare repo so bonesremote can access it:
  ```
  git push <remote_name> <branch>
  ```
- `bonesdeploy doctor` checks the local and remote environment, including whether the configured deploy branch exists locally and in the remote bare repo.
- Doctor uses exit status for actionable failures; an empty remote repository before the first branch push is a successful pending state so setup can finish cleanly.
- User runs `bonesdeploy deploy` to perform the actual remote release deployment.

### Primary Deploy Flow

1. `bonesdeploy deploy` publishes local `.bones/` state, then SSHes into the configured host.
2. It runs `bonesremote deploy --site <site>`.
3. `bonesremote deploy` orchestrates the full pipeline:
   - **stage_release** — Create timestamped release state
   - **release_checkout** — Export the configured branch revision from the bare repo via `git archive` (a clean tar stream without `.git` metadata); the stream is extracted into a temporary build context
    - **release_build** — Run `deployment/build/*.sh` inside bonesremote's `buildpack-deps:bookworm` container at `/workspace/source`. If `[build].vars` declares env var names, those vars are read from `shared/.env` on the host and injected into the container via `--env`.
   - **release_promote** — Copy safe artifacts into a runtime-owned candidate release
   - **wire_shared** — Symlink declared shared paths into the candidate release
   - **release_prepare** — Run `deployment/prepare/*.sh` as the site runtime user
   - **release_finalize** — Seal the prepared release as `root:<site>`
   - **activate_release** — Atomically repoint `current`
   - **restart_services** — Restart `<site>.target`, which restarts all registered site services
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
       - **release_build** — Run `deployment/build/*.sh` inside bonesremote's `buildpack-deps:bookworm` container at `/workspace/source`. If `[build].vars` declares env var names, those vars are read from `shared/.env` on the host and injected into the container via `--env`.
      - **release_promote** — Copy safe artifacts into a runtime-owned candidate at `releases/<release>`
      - **wire_shared** — Link shared runtime paths
      - **release_prepare** — Run `deployment/prepare/*.sh` as the site runtime user
      - **release_finalize** — Seal the prepared release as `root:<site>`
      - **activate_release** — Repoint `current`
      - **restart_services** — Restart `<site>.target`, which restarts all registered site services
      - **post_deploy** — Prune old releases beyond `releases`
      - On failure: **drop_failed_release** — Restore the previous release when activation occurred, then clean up the
        failed staged release

`bonesdeploy deploy` performs the same remote pipeline by SSHing into the host and running `bonesremote deploy --site <site>` directly (without `--revision`, so it uses the configured branch). Git-triggered deploy is optional plumbing, not the primary model.
