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
| `git` (deploy user) | Application bare repo | Ingress only |
| `root` | `.bones` bare repo and config state | Ingress and control-plane import |
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

Python infra scripts and templates live in the `bonesinfra` crate (`crates/bonesinfra/python/`), are embedded into the `bonesdeploy` binary, and are materialized on demand into a venv under `~/.cache/bonesdeploy/bonesinfra`; see `crates/bonesinfra/src/lib.rs`. Project configuration lives under `~/.config/bonesdeploy/projects`, while the application GPG keyring lives under `~/.local/share/bonesdeploy/gnupg`.

### Bones TOML
This stores crucial data we will need and is collected on running `bonesdeploy init` via user prompts.  
Collects the following project information from the user:
- `project_name`: str
- `branch`: str
- `remote_name`: existing remote selection when available, otherwise prompted; defaults to `production`. Must point to a fresh VPS, not a code host like GitHub.
- `host`: prompted when not inferable from selected remote
- `port`: defaults to `22`, prompt shown when remote inference is unavailable
Everything else is defaulted or derived for Debian/Ubuntu-first usability:
- `ssh_user`: defaults to `root`
- `deploy_on_push`: defaults to `false`
- `releases`: defaults to `5`

`[framework]` in `.bones/bones.toml` contains the selected template, language runtime versions, web root, permissions, and shared paths. `node_version`, `python_version`, `ruby_version`, and `php_version` are explicit runtime selections; provisioning installs the selected host runtime and build scripts receive the same derived values. The runtime identity (`runtime_user`, `runtime_group`) is always derived from `project_name` and is not stored in `bones.toml`. Shared paths are declared under `[framework.shared].paths`; deploys only wire the paths listed there, so framework-specific writable paths must not be hardcoded globally. Shared storage paths must be created by the framework itself; BonesDeploy only wires the declared paths into each release. Node-based builds use `BONES_FRAMEWORK_NODE_VERSION`, which takes precedence over any conflicting `NODE_VERSION` in `.env.build`.

Users can override any default by editing `.bones/bones.toml` after init.

`[services].services` is selected during init (or with repeated non-interactive `--service` flags). Supported values are `postgres`, `mariadb`, `mysql`, `mongodb`, `valkey`, and `redis`. Database provisioning binds every listener to localhost, generates credentials on the host, and writes connection values only to the protected `shared/.env`. Redis and Valkey use separate per-project instances; PostgreSQL, MariaDB, MySQL, and MongoDB use database-scoped accounts. Remote workstation access uses ordinary SSH port forwarding; no tunnel information is stored. MariaDB and MySQL are mutually exclusive server implementations.

Example `bones.toml`:
```toml
[app]
remote_name = "production"
project_name = "lawsnipe"

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

[framework]
template = "next"
web_root = "public"
```

### Build-time configuration
`.env.build` at the project root declares non-secret values injected into the build container at build time. It is committed to Git and parsed without shell evaluation.

```env
# .env.build
# Committed, non-secret values used while building this project.
# Do not place passwords, tokens, or private keys here.
NEXT_PUBLIC_API_URL=https://api.example.com
NEXT_PUBLIC_SITE_NAME=Example
# Laravel only: pin the Composer release used by the build.
COMPOSER_VERSION=2.8.12
```

Rules:
- `.env.build` is committed to Git and visible in plaintext.
- It is never copied into runtime `shared/`.
- Missing `.env.build` means no additional build variables — existing projects do not break.
- `BONES_*` names are reserved and cannot be used.
- Duplicate keys and invalid names fail clearly during the build.

The build environment consists of:
1. Existing generic variables (`PROJECT_NAME`, `WEB_ROOT`, etc.).
2. Values from committed `.env.build`.
3. Derived `BONES_*` values from `bones.toml`.
4. Fixed internal values such as `BUILD_CACHE_DIR`.

Laravel builds use Composer `2.8.12` by default. Set `COMPOSER_VERSION` in
`.env.build` to select another stable `x.y.z` Composer release compatible with
the selected PHP version. Builds download the pinned PHAR directly with curl,
verify its SHA-256 checksum, and use bounded network timeouts.

Derived `BONES_*` values win over `.env.build` collisions because they represent canonical Bones configuration. Runtime secrets belong in `shared/.env` via `bonesdeploy secrets push`.

### Hooks
The optional git push transport uses thin adapters: a local `pre-push` guard embedded in the `bonesdeploy` binary and a remote `post-receive` trigger embedded in the `bonesremote` binary. The config repo uses a separate `pre-receive` trigger installed by provisioning. Neither adapter is visible or editable under `.bones/`. Set `deploy_on_push = true` in `.bones/bones.toml` to enable git-triggered deploys.

- `pre-push` => Installed by `bonesdeploy init` into `.git/hooks/pre-push`. This checks if we are pushing to the bonesdeploy designated remote. If so, it runs `bonesdeploy doctor --local` and fails if doctor reports warnings or errors.
- `post-receive` (app repo) => Installed automatically into the bare repo at `/home/git/<project>.git/`. Derives `<site>` from `GIT_DIR` and runs `sudo bonesremote hook post-receive --site <site>`. `bonesremote` then reads branch policy and config from `/root/.config/bonesremote/sites/<site>/`.
- `config-pre-receive` (config repo) => Installed during provisioning into `/root/.config/bonesremote/repos/<project>.bones.git/`. Derives `<site>` from `GIT_DIR` by stripping `.bones.git`, reads the pushed revision, and calls `bonesremote site receive --site <site> --revision <rev>` directly as root before Git accepts the update. `bonesremote` archives the revision from the bones repo via `git archive`, validates the dataset, and atomically replaces the control-plane state.

### Config Repo
`bonesdeploy push` publishes the `.bones/` directory to a dedicated root-owned bare repo at `/root/.config/bonesremote/repos/<project>.bones.git`. A fresh `bonesdeploy init` creates the local repository, its `.gitignore`, and the `root` `origin` remote. Existing projects need the equivalent migration setup before using this transport. The push workflow:
1. Stages and commits all content with `"automated commit"`.
2. Pushes `master` to `root@<host>:/root/.config/bonesremote/repos/<project>.bones.git`.

On the server, the `config-pre-receive` hook triggers `bonesremote site receive`, which:
1. Archives the pushed revision via `git archive --format=tar <rev>` from the bones repo.
2. Extracts and validates the dataset (same validation as `site import`).
3. Acquires the deployment lock, ensures the site is idle, and atomically replaces the control-plane state under `/root/.config/bonesremote/sites/<site>/`.

### Update Patches
`bonesdeploy update` runs the ordered, embedded migration patches after each local or remote binary update. Completed patches are recorded per project and scope, so interrupted updates retry safely without rerunning successful patches. Local patches use the project data directory; remote patches use `/var/lib/bonesdeploy/patches/<site>/` and run through the root SSH session. `--skip-local` and `--skip-remote` also skip their respective patches.

### Deployment Folder
This folder stores build and prepare scripts that are published into bonesremote site state. Build scripts live in `.bones/deployment/build/`, must use the `NN_name.sh` convention (for example, `01_install_deps.sh`, `02_run_build.sh`), and run in lexical order inside bonesremote's `buildpack-deps:bookworm` container with `cwd=/workspace/source`; other files, including `README.md`, are ignored. Bonesremote prepares the image and executes scripts through the build user's systemd user manager with `systemd-run --machine=<site>-build@ --user`, rather than changing UID with `runuser`. The long-lived build container is a transient user service that tracks Podman's monitor process, while each script still streams its output through foreground `podman exec`. Before scripts run, Bonesremote streams the deployment bundle into the container's disposable filesystem at `/workspace/deployment`; it does not bind-mount the root-owned control-plane path. The build container receives the exported source tree and private persistent build cache at `/workspace/cache`; it does not receive `.env`, `shared/`, `current`, `releases/`, the bare repo, or host bonesremote control-plane files. The cache is provisioned by BonesInfra at `/var/lib/bonesdeploy/users/<site>-build/cache` and is used only for tool and package downloads. Prepare scripts live in `.bones/deployment/prepare/`, use the same naming convention, run in lexical order as the site runtime user with `cwd` set to a runtime-owned candidate release, and are the right place for migrations, cache warmups, and other runtime-state work. For each prepare script, Bonesremote opens the root-owned shared `functions.sh` and script, then streams both as one stdin input to the runtime-user shell; the runtime user receives no filesystem access to the deployment bundle. Before prepare scripts run, `bonesremote` wires each `[framework.shared].paths` entry into the candidate; after prepare succeeds, it seals the release before activation.

## Crate Structure
This Cargo workspace has four crates under `crates/`:
- `bonesdeploy` for the local CLI binary
- `bonesremote` for the server-side binary
- `bonesinfra` for the embedded Python provisioning runtime (pyinfra operations, framework templates) and the Rust wrapper that materializes and runs it
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
│   ├── bonesinfra/
│   │   ├── python/             # Python package (pyinfra operations, runtimes, Jinja2 templates)
│   │   └── src/                # embeds python/, materializes it, runs `python -m bonesinfra`
│   └── shared/                 # config schema + central paths
└── docs/
```

### Per-Framework Templates
Framework templates ship starter overlays that `bonesdeploy remote framework` uses when scaffolding infrastructure for a matching framework. Each template lives in the embedded `bonesinfra` package (`crates/bonesinfra/python/src/bonesinfra/frameworks/`) — framework runtime assets and Jinja2 templates stay together:

- `frameworks/laravel/`        → Laravel (PHP + PHP-FPM)
- `frameworks/django/`         → Django (Python + Gunicorn)
- `frameworks/next/`           → Next.js (Node)
- `frameworks/nuxt/`           → Nuxt (Node)
- `frameworks/sveltekit/`     → SvelteKit (Node)
- `frameworks/vue/`           → Vue (Node)
- `frameworks/rails/`         → Rails (Ruby)

Templates inherit the same `bones.toml` schema and customize permissions paths, deployment scripts, and the runtime operations captured in the `bonesinfra` crate.

### BonesDeploy CLI Commands
- **init**:
  - Loads existing config from `.bones/bones.toml` or collects user input via prompts.
  - For fresh init, waits until prompts complete before creating `.config/bonesdeploy/projects/<project>.bones/` and the local `.bones` symlink.
  - Updates `.gitignore` to add `.bones` and explicitly keep the generated `.env.build` trackable even when the project ignores `.env.*` files.
  - Creates local deployment remote if missing using `{deploy_user}@{host}:{repo_path}`, constructed from the production VPS target configured during prompts.
  - Prints next-step guidance to run `bonesdeploy remote setup` and `bonesdeploy remote framework` before first deploy.
  - Saves config to `.bones/bones.toml`.
  - Framework template selection and per-template questions are sourced from `crates/bonesdeploy/src/frameworks/<fw>.rs` (typed Rust, embedded in the binary). `init` no longer calls `bonesinfra runtime questions` or prefetches `bonesinfra`.
  - `--template <name>` selects a framework template non-interactively. `--framework-var <key=value>` (repeated) overrides template variables; answers are validated against the template's question schema before writing `bones.toml`.

- **doctor**
  - This command checks all concerns in your local environment.
  - Checks are reported as pass, pending, or failure. A pending first Git push is expected after remote setup and exits successfully; broken prerequisites still exit non-zero.
  - Loads config from `.bones/bones.toml`
  - Runs local checks:
    - `.bones` folder exists and is a symlink (warns if it is not a symlink to `~/.config/bonesdeploy/projects/<project>.bones/`).
    - Deployment scripts under `.bones/deployment/build/` and `.bones/deployment/prepare/` are ordered with numeric prefixes.
    - Local `pre-push` guard is installed properly when `deploy_on_push = true`. Checks for the presence and version marker in the baked script.
  - Runs remote checks (skipped with `--local`):
    - Opens a privileged SSH session and runs `bonesremote doctor --site <project>`.
    - `bonesremote doctor --site <project>` requires root and checks Podman availability, AppArmor availability, imported control-plane state under `/root/.config/bonesremote/sites/<project>/`, the build user's existence and home, the bare repo and thin `post-receive` hook, runtime user/group constraints, `shared/` and `releases/` layout, and `<project>-nginx.service`. An empty bare repo is reported as pending until the configured branch is pushed.
    - The security audit is read-only and fail-closed. It verifies site identity isolation (unique UIDs/GIDs, no login shells, no cross-site group membership, deploy not in runtime groups), runtime sudo absence, privileged configuration root-control (recursively inspecting systemd, sudoers, nginx, AppArmor, and BonesRemote state plus their parent chains without following symlink targets), and release activation (current must be a valid symlink resolving inside the site's releases directory; active release roots and activation parents must be immutable to the runtime identity). `bonesremote doctor --site <project> --exhaustive` additionally inspects every entry in that active release for permission drift; this can take time on large releases. The exact deploy-user sudoers policy is rendered and validated by `bonesinfra` during provisioning rather than probed with fabricated commands during doctor. POSIX ACLs on protected paths are detected through extended attributes and reported as UNVERIFIED. Supplementary groups are collected through `id -G`. Required evidence that cannot be collected is reported as UNVERIFIED and causes doctor to fail.
  - The `--local` flag skips all remote checks. The `pre-push` hook uses this flag because it is only a local guard before optional git-triggered deploys. `--verbose` prints the complete successful remote doctor report instead of collapsing it to the `remote doctor` check.

- **push**
  - Publishes the local `.bones/` directory to a dedicated root-owned bare config repo at `/root/.config/bonesremote/repos/<project>.bones.git` on the server via `git push`.
  - A fresh `bonesdeploy init` writes `.bones/.gitignore` (excludes plaintext `.env`), initialises the local Git repo in `.bones/`, and adds the config-repo origin. Existing projects require this migration setup before using the Git transport.
  - Before pushing, stages and autocommits `.bones` content.
  - The server-side `config-pre-receive` hook triggers `bonesremote site receive`, which atomically replaces the current remote site state under `/root/.config/bonesremote/sites/<project>/` before Git accepts the update.

- **pull**
  - Streams the current remote site dataset back from `bonesremote site export --site <project>` and extracts it into local `.bones/`.
  - Re-installs the local pre-push guard so the repository regains its pre-push check after recovery.

- **deploy**
  - Publishes the local `.bones/` dataset into remote bonesremote site state first, then SSHes into the configured host and runs `bonesremote deploy --site <project>` directly.
  - Pushes the decrypted local `.bones/secrets/.env.gpg` into the remote `shared/.env` before starting the deployment.
  - Omits the `--revision` flag, so `bonesremote deploy` uses the configured branch from `bones.toml`.

- ****remote setup****
  - Delegates to the embedded `bonesinfra` runtime by running `python -m bonesinfra setup apply --config <path>` against the configured host as root (or `BONES_BOOTSTRAP_SSH_USER`).
  - Passes `bones.toml` deployment values plus computed paths and variables as JSON on stdin.
  - Initializes bare git repository at `repo_path`.
  - Creates initial placeholder release with default page.
   - Downloads and checksum-verifies the matching static `x86_64` `bonesremote` Linux release binary from GitHub Releases.
   - Does not install Rust or Cargo on the remote host.
  - Installs the deploy-user sudoers policy through `bonesinfra` host provisioning, with anchored site and revision arguments so trailing or malformed arguments are denied.
   - Provisions machine-level dependencies (users, groups, firewall, system packages).

`bonesdeploy update` resolves the latest published GitHub release, validates that
its `v<version>` tag matches both package manifests, clones that exact tag for
patches and scaffold updates, installs the matching crates.io `bonesdeploy`, and
downloads the matching static `x86_64` `bonesremote` asset. ARM hosts fail
clearly because release binaries currently support only `x86_64` Debian/Ubuntu.

- **remote framework**:
  - Prompts for a framework template, refreshes `.bones/runtime/`, and writes the selected settings into `.bones/bones.toml`.
  - Reapplies template-specific defaults into `.bones/bones.toml` only when they still match generic or previous-template values.
  - After a `y/N` confirmation, delegates to the embedded `bonesinfra` runtime by running `python -m bonesinfra runtime apply --config <path> --runtime-config <path>` against the configured host as the configured `ssh_user`.
  - Loads the template's `operations.py` at runtime to install framework-specific packages and services.
  - Configures per-site runtime assets: AppArmor profile, nginx router + per-site config + systemd service, and runs `bonesremote doctor`.
  - Does not handle SSL; use `remote ssl` for TLS configuration.

- **remote services**:
  - Provisions the services selected in `[services]`; `bonesdeploy setup` runs this after bootstrap.
  - Keeps all database listeners loopback-only and does not publish credentials into the remote control-plane dataset.

- **remote ssl**
  - Delegates to the embedded `bonesinfra` runtime by running `python -m bonesinfra ssl apply --config <path>` against the configured host as root.
  - Uses certbot with a webroot challenge to obtain/renew certificates for the configured domain.
  - Re-renders the per-site runtime nginx router with TLS enabled, listening on 443 and redirecting HTTP to HTTPS.
  - Separate from `remote framework` to keep certificate management decoupled from app runtime concerns.

- **rollback**
  - SSHes into the configured host and runs `bonesremote release rollback --site <project>`, which repoints `current` to the previous release without rebuilding and restarts `<project>.target`.

- **secrets**
  - Subcommands: `init`, `edit`, `push`.
  - Manages GPG-encrypted environment secrets under `.bones/secrets/`.
  - `init` bootstraps `.bones/secrets/.env.gpg` with the selected runtime's defaults; `secrets init` remains an idempotent manual equivalent.
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
  - Topic docs are markdown files under `crates/bonesdeploy/skill/` and are embedded with `rust-embed` alongside `kit/` and `frameworks/`.
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
    - **release_build** — Run `deployment/build/*.sh` inside bonesremote's `buildpack-deps:bookworm` container at `/workspace/source`. `.env.build` from the exported source tree is parsed and injected into the container via `--env`.   - **release_promote** — Copy safe artifacts into a runtime-owned candidate release
   - **wire_shared** — Symlink declared shared paths into the candidate release
   - **release_prepare** — Run `deployment/prepare/*.sh` as the site runtime user
   - **release_finalize** — Seal the prepared release as `root:<site>`
   - **activate_release** — Atomically repoint `current`
   - **restart_services** — Restart `<site>.target`, which restarts all registered site services
   - **post_deploy** — Prune old releases beyond `releases`
   - On failure: **drop_failed_release** — Clean up staged release

## Hook Event Order

### App Repo: `git push` (deployment trigger)

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
        - **release_build** — Run `deployment/build/*.sh` inside bonesremote's `buildpack-deps:bookworm` container at `/workspace/source`. `.env.build` from the exported source tree is parsed and injected into the container via `--env`.      - **release_promote** — Copy safe artifacts into a runtime-owned candidate at `releases/<release>`
       - **wire_shared** — Link shared runtime paths

### Config Repo: `bonesdeploy push` (control-plane update)

`git init -> commit -> push (master) -> config-pre-receive`

1. **push** (local): `bonesdeploy push` stages and autocommits changes, then pushes `master` to `root@<host>:/root/.config/bonesremote/repos/<project>.bones.git`. Fresh projects receive the Git repository setup during `bonesdeploy init`.
2. **config-pre-receive** (remote): Derives `<site>` from `GIT_DIR`, reads the pushed revision from stdin, and calls `bonesremote site receive --site <site> --revision <rev>` directly as root before accepting the push.
3. **site receive** (remote): Archives the revision from the bones repo via `git archive`, validates the dataset, acquires the deployment lock, and atomically replaces control-plane state under `/root/.config/bonesremote/sites/<site>/`.
      - **release_prepare** — Run `deployment/prepare/*.sh` as the site runtime user
      - **release_finalize** — Seal the prepared release as `root:<site>`
      - **activate_release** — Repoint `current`
      - **restart_services** — Restart `<site>.target`, which restarts all registered site services
      - **post_deploy** — Prune old releases beyond `releases`
      - On failure: **drop_failed_release** — Restore the previous release when activation occurred, then clean up the
        failed staged release

`bonesdeploy deploy` performs the same remote pipeline by SSHing into the host and running `bonesremote deploy --site <site>` directly (without `--revision`, so it uses the configured branch). Git-triggered deploy is optional plumbing, not the primary model.
