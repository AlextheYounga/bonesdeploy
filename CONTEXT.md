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

Permissions are a **provisioning-time contract**, not a deployment-time repair. The ownership layout is established once during `bonesdeploy server setup` and site setup, and never rewritten by deploy commands.

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

`bonesdeploy site releases` asks `bonesremote` for the site's release state and renders the returned JSON locally; it stores no release state on the workstation. Releases are `active`, `previous`, `building`, `preparing`, or `interrupted`. A `building` or `interrupted` release can be cancelled with `bonesdeploy site releases kill <release>`; cancellation removes only that release's build container, temporary context, staged-release state, and transient deployment metadata.

BonesRemote holds one OS-backed deployment lock per site. Deploys, cancellations, and site imports use the same stable lock, which lives outside the replaceable site dataset. A deploy or import must not stage or overwrite state while a release is building, preparing, or interrupted. Before staging, BonesRemote starts and verifies the build user's systemd manager and checks rootless Podman readiness. A damaged rootless Podman namespace is reported before any release state is created; deploy does not silently reset Podman because that operation stops the build user's containers.

Deployment state files (`active-deployment.json` and `staged-release`) are written atomically (temp file, fsync, rename, directory fsync) so a crash or disk-full condition never leaves truncated state that status, cancellation, or idle checks cannot parse. If malformed state is ever found, `bonesremote release recover --site <site>` quarantines it after proving no deployment is running.

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

Python infra scripts and templates live in the `bonesinfra` crate (`crates/bonesinfra/python/`) and are embedded into the `bonesdeploy` binary. The complete distribution is materialized into each project's `infra/.framework/`; a project-scoped cached venv holds only dependencies and editable-install metadata. See `crates/bonesinfra/src/lib.rs`.

### Project Environment
`bonesdeploy init` collects project settings and writes the canonical flat root `.env`. It includes the project name, SSH connection, deployment branch, domain, selected framework, web root, and services.

Build-only public settings live in the committed `.env.build`. Framework templates declare `NODE_VERSION`; when set, this value is passed to build scripts as `NODE_VERSION` and takes precedence over version files in the repository. Provisioning defaults to `24.19.0`.

`[services].services` is selected during init (or with repeated non-interactive `--service` flags). Supported values are `postgres`, `mariadb`, `mysql`, `mongodb`, `valkey`, and `redis`. Database provisioning binds every listener to localhost, generates credentials on the host, and writes connection values only to the protected `shared/.env`. Redis and Valkey use separate per-project instances; PostgreSQL, MariaDB, MySQL, and MongoDB use database-scoped accounts. Remote workstation access uses ordinary SSH port forwarding; no tunnel information is stored. MariaDB and MySQL are mutually exclusive server implementations.

Example `.env`:
```dotenv
PROJECT_NAME=lawsnipe
REMOTE_NAME=production

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

[runtime]
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

### Update Patches
`bonesdeploy update` invokes the embedded `bonesinfra patches apply` command after each local or remote binary update. Python owns the ordered registry, version gates, local Git migrations, remote pyinfra operations, and per-project/per-scope completion markers. Completed patches are recorded per project and scope, so interrupted updates retry safely without rerunning successful patches. Local markers use the project data directory; remote markers use `/var/lib/bonesdeploy/patches/<site>/`. Remote patch plans connect as root through the local embedded BonesInfra runtime; Python is not installed on the deployment host. `--skip-local` and `--skip-remote` also skip their respective patches.

### Deployment Folder
This folder stores build and prepare scripts. Build scripts live in `deployment/build/`, must use the `NN_name.sh` convention (for example, `01_install_deps.sh`, `02_run_build.sh`), and run in lexical order inside bonesremote's `buildpack-deps:bookworm` container with `cwd=/workspace/source`; other files, including `README.md`, are ignored. Bonesremote prepares the image and executes scripts through the build user's systemd user manager with `systemd-run --machine=<site>-build@ --user`, rather than changing UID with `runuser`. The long-lived build container is a transient user service that tracks Podman's monitor process, while each script still streams its output through foreground `podman exec`. Before scripts run, Bonesremote streams the deployment bundle into the container's disposable filesystem at `/workspace/deployment`; it does not bind-mount root-owned control-plane state. The build container receives the exported source tree and private persistent build cache at `/workspace/cache`; it does not receive `.env`, `shared/`, `current`, `releases/`, the bare repo, or host BonesRemote control-plane files. The cache is provisioned by BonesInfra at `/var/lib/bonesdeploy/users/<site>-build/cache` and is used only for tool and package downloads. Prepare scripts live in `deployment/prepare/`, use the same naming convention, run in lexical order as the site runtime user with `cwd` set to a runtime-owned candidate release, and are the right place for migrations, cache warmups, and other runtime-state work.

## Crate Structure
This Cargo workspace has four crates under `crates/`:
- `bonesdeploy` for the local CLI binary
- `bonesremote` for the server-side binary
- `bonesinfra` for the embedded Python provisioning runtime (pyinfra operations, framework templates) and the Rust wrapper that materializes and runs it
- `bonesdeploy-core` for code that must be common to both binaries

### Path Centralization
All product-owned paths must live in `crates/bonesdeploy-core/src/paths.rs`.

Other modules may derive subpaths by joining values from `bonesdeploy-core::paths`, but they must not introduce their own independent path roots, filenames, or install locations.

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
│   │   ├── python/             # Python package (pyinfra operations, core services, Jinja2 templates)
│   │   └── src/                # embeds python/, materializes it, runs `python -m bonesinfra`
│   └── bonesdeploy-core/  # config schema + central paths
└── docs/
```

### Per-Framework Templates
Framework templates ship starter overlays that `bonesdeploy init` uses when scaffolding a matching framework. BonesDeploy keeps framework defaults and deployment scripts under `crates/bonesdeploy/assets/frameworks/<fw>/`; canonical infrastructure source and templates live in BonesInfra and are materialized into `infra/.framework/`:

- `frameworks/laravel/`    → Laravel (PHP + PHP-FPM)
- `frameworks/django/`     → Django (Python + Gunicorn)
- `frameworks/next/`       → Next.js (Node)
- `frameworks/nuxt/`       → Nuxt (Node)
- `frameworks/sveltekit/`  → SvelteKit (Node)
- `frameworks/vue/`        → Vue (Node)
- `frameworks/rails/`      → Rails (Ruby; supported releases are provisioned from verified source archives)

Django resolves its configured `python_version` minor to a BonesInfra-pinned CPython patch release, verifies the official source archive checksum, and installs it under `/opt/bonesdeploy/python/<patch>`. It never changes Debian's `/usr/bin/python3`; Django releases create their `.venv` with the versioned `python3.<minor>` executable.

Templates inherit the same `bones.toml` schema and customize permissions paths, deployment scripts, and the runtime operations captured in the generated `infra/runtime.py` per project.

Projects materialize the complete BonesInfra distribution under `infra/.framework/` and preserve project-owned hooks under `infra/custom/`. `bonesinfra runtime apply` executes the project-local package and composes its selected framework runtime with custom provisioning; it has no cached-source fallback.

Static runtimes deploy from a `web_root` subdirectory of each release that nginx serves (e.g. Next's `out/`). A static site only works if the app is configured to emit that directory: for `is_static = true`, Next.js must set `output: "export"` in `next.config.js`/`next.config.mjs`/`next.config.ts`; otherwise the first deploy fails with *"Static Next.js deployments require out/index.html"*.

### BonesDeploy CLI Commands
- **init**:
  - Loads the root `.env` or collects user input via prompts.
  - For fresh init, waits until prompts complete before writing the root `.env`, committed `.env.build`, `deployment/`, and `infra/`.
  - Updates `.gitignore` to keep `.env` local while leaving `.env.build` trackable.
  - Creates local deployment remote if missing using `{deploy_user}@{host}:{repo_path}`, constructed from the production VPS target configured during prompts.
  - Prints next-step guidance to run `bonesdeploy server setup --yes` and `bonesdeploy site setup --yes` before first deploy.
  - Saves connection and site inputs to the root `.env`.
  - Framework template selection and per-template questions are sourced from `crates/bonesdeploy/src/frameworks/<fw>.rs` (typed Rust, embedded in the binary). BonesDeploy materializes deployment assets from `crates/bonesdeploy/assets/frameworks/<fw>/` and the complete BonesInfra distribution into `infra/.framework/`.
  - `--template <name>` selects a framework template non-interactively. `--framework-var <key=value>` (repeated) overrides template variables; answers are validated against the template's question schema before writing `.env`.

- **doctor**
  - Root `bonesdeploy doctor` runs both `server doctor` and `site doctor`, reporting both failures when necessary.
  - `bonesdeploy site doctor --local` checks only the local root `.env`, `infra/`, and numbered deployment scripts.
  - Site remote checks open a privileged SSH session and run `bonesremote doctor --site <project>`.
    - `bonesremote doctor --site <project>` requires root and checks Podman availability, AppArmor availability, imported control-plane state under `/root/.config/bonesremote/sites/<project>/`, the build user's existence and home, the bare repo and thin `post-receive` hook, runtime user/group constraints, `shared/` and `releases/` layout, and `<project>-nginx.service`. An empty bare repo is reported as pending until the configured branch is pushed.
    - The security audit is read-only and fail-closed. It verifies site identity isolation (unique UIDs/GIDs, no login shells, no cross-site group membership, deploy not in runtime groups), runtime sudo absence, privileged configuration root-control (recursively inspecting systemd, sudoers, nginx, AppArmor, and BonesRemote state plus their parent chains without following symlink targets), and release activation (current must be a valid symlink resolving inside the site's releases directory; active release roots and activation parents must be immutable to the runtime identity). `bonesremote doctor --site <project> --exhaustive` additionally inspects every entry in that active release for permission drift; this can take time on large releases. The exact deploy-user sudoers policy is rendered and validated by `bonesinfra` during provisioning rather than probed with fabricated commands during doctor. POSIX ACLs on protected paths are detected through extended attributes and reported as UNVERIFIED. Supplementary groups are collected through `id -G`. Required evidence that cannot be collected is reported as UNVERIFIED and causes doctor to fail.
   - Server doctor verifies Debian/Ubuntu, Podman, AppArmor, deploy identity, BonesRemote roots and binary, sudoers, shared image store, firewall, fail2ban, and unattended-upgrades. `--verbose` prints successful remote reports.

- **site manifest**
  - Inspects every project-specific filesystem artifact and managed systemd service expected by the effective framework, service, and SSL strategy. Shared host-wide packages, daemons, and configuration are excluded.
  - Delegates to the embedded BonesInfra runtime as `python -m bonesinfra manifest show --env-file <path> --format <format>`.
  - Uses typed Python declarations inside BonesInfra, resolves path keys through `DeploymentPaths`, and performs read-only PyInfra fact checks.
  - Reports present, missing, and wrong-kind paths, plus active and enabled state for managed services. `--format json` is intended for automation and never includes file contents or secrets.

- **deploy**
  - Pushes decrypted local secrets into remote `shared/.env`, then SSHes into the configured host and runs `bonesremote deploy --site <project>` directly.
  - Uses the branch configured in the root `.env`.

- **server setup**
  - Delegates to `python -m bonesinfra server apply --env-file <path>` using only SSH host, user, and port.
  - Provisions shared packages, hardening, firewall, image store, deploy identity, BonesRemote roots and binary, and sudoers.
  - Does not read project runtime, service, framework, DNS, or release settings.

- **site setup**
  - Verifies server readiness before any site mutation.
  - Runs site base, services, runtime, and site doctor in that order.
  - Site base creates one bare repository, site identities, paths, root-owned control-plane state, and a placeholder release.
  - Does not push Git or secrets, configure SSL, or deploy a release.

`bonesdeploy update` resolves the latest published GitHub release, validates that
its `v<version>` tag matches both package manifests, clones that exact tag for
patches and scaffold updates, installs the matching crates.io `bonesdeploy`, and
downloads the matching static `x86_64` `bonesremote` asset. ARM hosts fail
clearly because release binaries currently support only `x86_64` Debian/Ubuntu.

- **site runtime**:
  - Reapplies the configured runtime settings from the root `.env` to the host and provisions the selected framework's runtime.
  - Delegates to the embedded `bonesinfra` runtime by running `python -m bonesinfra runtime apply --env-file <path>` against the configured host as the configured `ssh_user`.
  - Imports and runs the project's `infra/runtime.py` (local vendored package) or the selected canonical BonesInfra framework package, which installs framework-specific packages and services.
  - Configures per-site runtime assets: AppArmor profile, nginx router + per-site config + systemd service, and runs `bonesremote doctor`.
  - Does not handle SSL; use `site ssl` for TLS configuration.

- **site services**:
  - Provisions the services selected in `[services]`; `bonesdeploy site setup` runs this after server readiness and site base provisioning.
  - Keeps all database listeners loopback-only and does not publish credentials into the remote control-plane dataset.

- **site ssl**
  - Delegates to the embedded `bonesinfra` runtime by running `python -m bonesinfra ssl apply --config <path>` against the configured host as root.
  - Uses certbot with a webroot challenge to obtain/renew certificates for the configured domain.
  - Re-renders the per-site runtime nginx router with TLS enabled, listening on 443 and redirecting HTTP to HTTPS.
  - Separate from `site runtime` to keep certificate management decoupled from app runtime concerns.

- **rollback**
  - SSHes into the configured host and runs `bonesremote release rollback --site <project>`, which acquires the site lock and repoints `current` to the previous release without rebuilding, then restarts `<project>.target`. If the restart fails, the original release is restored and restarted.

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
   - `bonesdeploy skill next [--format text|json]` inspects `.env` and the remote host, then suggests the next prompt-free command across `uninitialized`, `server_missing`, `site_missing`, `ssl_missing`, and `ready` states.
  - Topic docs are markdown files under `crates/bonesdeploy/assets/skill/` and are embedded with `rust-embed` alongside `kit/` and `frameworks/`.
- **version**:
  - Echoes the installed `bonesdeploy` version.

### BonesRemote CLI Commands
- **Release commands** live under `bonesremote release ...`
- **Service commands** live under `bonesremote service ...`
- **deploy**:
  - Runs the full deployment lifecycle as a single command (the primary entrypoint used by both `post-receive` hook and `bonesdeploy deploy`).
  - Orchestrates: stage release → source export from the bare repo into a temp build context → build scripts → runtime-writable candidate release → shared wiring → prepare scripts as the site user → seal release → activate → restart `<site>.target` → post-deploy pruning.
  - Before activation, validates the site's nginx configuration with `nginx -t -c /srv/conf/<site>/nginx.conf`. On failure before activation, automatically drops the staged release. If the service restart fails after activation,
    restores and restarts the previous release before dropping the failed release.
  - `--site <name>`: imported site identifier used to load root-owned registry state
  - `--revision <rev>`: optional exact commit to check out; defaults to configured branch
- **doctor**:
  - Host mode checks `bonesremote` in `PATH`, Podman, AppArmor support, and the deploy-user sudoers drop-in.
  - `--site <name>` also checks the imported site boundary: validated control-plane state, bare repo and thin hook, runtime identity constraints, `shared/` and `releases/` layout, and `<site>.target`.
- **release stage**
	- Creates a staged release tree under `releases/`, ensures `build/workspace` and `shared/`, then writes staged release state before checkout. Release directories are created exclusively: the identity embeds the resolved source commit plus a random suffix (for example `20260804_190321-46a0b75c-a7f2`), and a second deployment staged within the same second retries with a fresh name instead of reusing or erasing an existing release directory.
- **release wire**
	- Wires shared paths into `build/workspace` after checkout, replacing any existing build workspace paths with symlinks to the shared directory.
- **release activate**
	- Atomically switches `current` to the staged release and clears staged release state. Activation refuses to promote into a nonempty release directory.
- **release drop-failed**
	- Deletes a failed staged release and clears staged release state.
- **release recover**
	- Quarantines malformed `active-deployment.json` state into the site's `recovery/` directory. It first acquires the site deployment lock, proving no deployment process is alive, so malformed state written by a crash can never wedge status, cancellation, or idle checks while a deploy runs.
- **release rollback**
	- Acquires the site deployment lock and requires an idle site, then repoints `current` to the previous release. It is transactional: after switching `current`, it restarts and verifies the target, and if verification fails it restores the original release and restarts it before returning an error.
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
- User can make any changes to their deployment scripts in `deployment/` and project infrastructure in `infra/custom/`.
- Before the first deploy (and after initial setup), the source code must be pushed to the remote bare repo so bonesremote can access it:
  ```
  git push <remote_name> <branch>
  ```
- `bonesdeploy site doctor` checks the local and site environment, including whether the configured deploy branch exists locally and in the remote bare repo. Root `bonesdeploy doctor` composes server and site diagnostics.
- Doctor uses exit status for actionable failures; an empty remote repository before the first branch push is a successful pending state so setup can finish cleanly.
- User runs `bonesdeploy deploy` to perform the actual remote release deployment.

### Primary Deploy Flow

1. `bonesdeploy deploy` pushes encrypted secrets, then SSHes into the configured host.
2. It runs `bonesremote deploy --site <site>`.
3. `bonesremote deploy` orchestrates the full pipeline:
   - **stage_release** — Create timestamped release state
   - **release_checkout** — Export the configured branch revision from the bare repo via `git archive` (a clean tar stream without `.git` metadata); the stream is extracted into a temporary build context
    - **release_build** — Run `deployment/build/*.sh` inside bonesremote's `buildpack-deps:bookworm` container at `/workspace/source`. `.env.build` from the exported source tree is parsed into a mode-0600 temporary env file and passed to Podman with `--env-file`, keeping its values out of process argv.   - **release_promote** — Copy safe artifacts into a runtime-owned candidate release
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

1. **pre-push** (local): Runs `bonesdeploy site doctor --local` if pushing to the configured bones remote and `deploy_on_push = true`. Aborts on warnings or errors.
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
        - **release_build** — Run `deployment/build/*.sh` inside bonesremote's `buildpack-deps:bookworm` container at `/workspace/source`. `.env.build` from the exported source tree is parsed into a mode-0600 temporary env file and passed to Podman with `--env-file`, keeping its values out of process argv.      - **release_promote** — Copy safe artifacts into a runtime-owned candidate at `releases/<release>`
       - **wire_shared** — Link shared runtime paths

`bonesdeploy deploy` performs the same remote pipeline by SSHing into the host and running `bonesremote deploy --site <site>` directly (without `--revision`, so it uses the configured branch). Git-triggered deploy is optional plumbing, not the primary model.
