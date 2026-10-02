# BonesDeploy v3

A remote release deployment tool for simple Linux servers. It produces two executables: `bonesdeploy` (local CLI for setup, provisioning, deployment, and management) and `bonesremote` (server-side release lifecycle executor, installed on the deployment host). BonesDeploy selects the configured local Git revision, builds a complete native or Compose artifact locally, and sends it over SSH for deployment. **We only handle Debian/Ubuntu machines.**

The command behavior is documented in this file and in the command examples in `README.md`.

## Deployment Methodology
We have an SSH deployment user named `deploy` that handles deployment concerns. This user has a home folder, restricted sudo ability, but no password login. We also have a per-project service user named after the project. This is not a shared `applications` user; it must be a dedicated user per project so isolation works on a shared server. This user has no home folder, no login, and no sudo ability. The project name `deploy` is reserved so the global deploy account cannot collide with a runtime identity.

The `deploy` account is a fresh-host contract. Existing hosts provisioned with the former `git` or `bonesdeploy` deploy account must be reprovisioned; no account migration or SSH fallback is provided.

### Just-in-Time Concerns
This project should prefer just-in-time mutations.

A concern should only be handled at the last responsible moment: immediately before the system would fail if that mutation did not occur. We should not widen permissions, rewrite symlinks, mutate shared state, or otherwise touch live project state early just because a later step might need it. The idea here is to limit the surface of attack time, so that potential vulnerabilities are not created by "jumping the gun" to solve a problem too early, long before it arises.

This principle exists to keep deployment behavior coherent and safe:

- the pre-deploy steps (doctor, artifact receipt, stage, wire) should validate and prepare isolated state, not mutate live state.
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
- Shared groups have too many logic traps. My apps should not have 660 or 770 permissions on all files so that a deploy user can have read/write.
- I don't like ACLs; they're far too opaque.
- Setting up inotify systems are cumbersome.

### Permission Model

Permissions are a **provisioning-time contract**, not a deployment-time repair. The ownership layout is established once during `bonesdeploy server setup` and site setup, and never rewritten by deploy commands.

**Three identity classes:**

| Identity | Owner of | Scope |
|----------|----------|-------|
| `deploy` (deploy user) | Deployment SSH entry point | Artifact transport and deployment SSH entry point |
| `root` | Config and release state | Artifact receipt and control-plane import |
| `<site>` (runtime user) | Shared files, `/run/<site>`, writable paths | Mutates runtime state |
| `root` | System units, config dirs, deployment state, sealed releases | Provisions and runs the allowlisted BonesRemote lifecycle |

**Key mechanics:**

- `releases/` contains candidates owned by the runtime user while prepare runs, then sealed as `root:<site>` before activation.
- `shared/` is owned by the runtime user (`<site>:<site>`) — only the app writes here.
- Build input is temporary and disposable. Native scripts run only in local Docker against the exported committed source and upload one complete post-build artifact. Production never executes native application build scripts.
- Prepare scripts run as the runtime user after shared paths are wired and before `current` is repointed. Most application dependencies and native extensions are built locally. Rails and Django instead rebuild dependencies against the host distribution runtime in the staged release before validation, migrations, and activation.
- Local Git selects the committed source tree; production receives only the built artifact and never needs an application repository or first push.
- The `deploy` SSH session may sudo only exact config-sync and deploy commands; BonesRemote retains ownership of promotion, activation, and service restart.
- `bonesdeploy site export` is separate local administration: it connects as the configured root SSH user and streams a read-only ZIP of `shared/` directly to a private local file. It does not use the deploy identity, sudo, or BonesRemote.

### Release Visibility and Cancellation

`bonesdeploy site releases` asks `bonesremote` for the site's release state and renders the returned JSON locally; it stores no release state on the workstation. Releases are `active`, `previous`, `building`, `preparing`, or `interrupted`. A `building` or `interrupted` release can be cancelled with `bonesdeploy site releases kill <release>`; cancellation removes only that release's build container, temporary context, staged-release state, and transient deployment metadata.

BonesRemote holds one OS-backed deployment lock per site. Deploys, cancellations, and site imports use the same stable lock, which lives outside the replaceable site dataset. A deploy or import must not stage or overwrite state while a release is building, preparing, or interrupted. Before staging, BonesRemote validates and safely receives the artifact before any release state is created.

Deployment state files (`active-deployment.json` and `staged-release`) are written atomically (temp file, fsync, rename, directory fsync) so a crash or disk-full condition never leaves truncated state that status, cancellation, or idle checks cannot parse. If malformed state is ever found, `bonesremote release recover --site <site>` quarantines it after proving no deployment is running.

## Bones Scaffolding
```
.env                      # gitignored local environment + managed BONES_* configuration block
.env.build                # committed, non-secret build inputs
infra/secrets/.env.gpg    # encrypted production application environment
deployment/
├── build/
│   ├── 01_install_build_deps.sh
│   └── 02_run_build.sh
└── prepare/
    └── 01_prepare.sh
```

Python infra code and templates live in the `bonesinfra` crate (`crates/bonesinfra/python/`). A generated pure-Python wheel is committed as `crates/bonesinfra/assets/bonesinfra-<version>-py3-none-any.whl`, checked against the Python source by `crates/bonesinfra/build.rs`, embedded into the `bonesdeploy` binary, and installed unchanged into each project's project-scoped cached venv. Regenerate it with `cargo build-wheel` after Python changes. Managed templates are fully materialized under `infra/templates/`, then framework paths declared in `crates/bonesinfra/src/framework_paths.rs` are pruned for the selected runtime. See `crates/bonesinfra/src/lib.rs`.

### Project Environment
Rust is the sole parser of the project-root `.env`. It models two environments:

- `LocalEnvironment` is the gitignored root `.env`, conceptually `.env.local`. Application-owned content (comments, values, order) survives byte-for-byte; it also carries exactly one BonesDeploy-managed, comment-delimited `BONES_*` configuration block holding project identity, SSH connection, deployment branch, domains, framework, web root, runtime backend, Compose settings, scheduled-backup settings, and framework-specific scalar settings.
- `ProductionEnvironment` is the decrypted `infra/secrets/.env.gpg`, conceptually `.env.production`, published to the host as `shared/.env` only by `bonesdeploy secrets push`. It contains application runtime keys and never `BONES_*` keys.

Build-only public settings live in the committed `.env.build`. Framework templates declare `NODE_VERSION`; when set, this value is passed to build scripts as `NODE_VERSION` and takes precedence over version files in the repository. Provisioning defaults to `24.19.0`.

BonesDeploy does not select or provision built-in databases and caches. Native
sites use independently managed services. Compose sites declare supporting
services in their Compose file.

Example `.env`:
```dotenv
# Local environment for the application.
APP_URL=http://app.example.test

# >>> BonesDeploy managed configuration >>>
BONES_PROJECT_NAME=lawsnipe
BONES_SSH_USER=root
BONES_HOST=deploy.example.com
BONES_PORT=22
BONES_BRANCH=main
BONES_DOMAIN=app.example.com
BONES_EMAIL=ops@example.com
BONES_TEMPLATE=next
BONES_RUNTIME_BACKEND=native
BONES_WEB_ROOT=public
BONES_NODE_VERSION=24.19.0
BONES_COMPOSE_PORT=
BONES_COMPOSE_WAIT_TIMEOUT=120
BONES_BACKUP_SCHEDULE=0 0 * * *
BONES_BACKUP_RETENTION_DAYS=30
BONES_BORG_PASSPHRASE=<generated>
# <<< BonesDeploy managed configuration <<<
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
3. Safe derived `BONES_*` values projected directly from local `Bones`
   configuration.
4. Fixed internal values such as `BUILD_CACHE_DIR`.

Laravel builds use Composer `2.8.12` by default. Set `COMPOSER_VERSION` in
`.env.build` to select another stable `x.y.z` Composer release compatible with
the selected PHP version. Builds download the pinned PHAR directly with curl,
verify its SHA-256 checksum, and use bounded network timeouts.

Derived `BONES_*` values win over `.env.build` collisions because they represent
canonical local Bones configuration. Separately, config sync sends a narrow
`RemoteDeploymentConfig` containing release retention and only the selected
backend's remotely consumed values. The encrypted `infra/secrets/.env.gpg` is
the source of truth for the complete remote `shared/.env`; `bonesdeploy secrets
push` atomically replaces that file without reading or merging another
environment file. The remote environment is application runtime data, not
BonesRemote control-plane configuration.

### Update Patches
`bonesdeploy update` invokes the embedded `bonesinfra patches apply` command after each local or remote binary update. Python owns the ordered registry, version gates, local Git migrations, remote pyinfra operations, and per-project/per-scope completion markers. Completed patches are recorded per project and scope, so interrupted updates retry safely without rerunning successful patches. Local markers use the project data directory; remote markers use `/var/lib/bonesdeploy/patches/<site>/`. Remote patch plans connect as root through the local embedded BonesInfra runtime; Python is not installed on the deployment host. `--skip-local` and `--skip-remote` also skip their respective patches.

### Scheduled Backups
Newly initialized projects get one encrypted Borg repository per site at `/var/lib/bonesdeploy/backups/<site>.borg` (`repokey-blake2`). `bonesdeploy init` generates the Borg passphrase into the managed `.env` block as `BONES_BORG_PASSPHRASE`; re-initializing preserves the existing backup configuration and only generates a passphrase when absent. Server setup installs Borg. Site provisioning writes the passphrase to the root-only `/root/.config/bonesremote/sites/<site>/.borg_passphrase` (mode `0600`), creates the repository, and renders `/etc/cron.d/bonesdeploy-<site>-backup` from the configured schedule.

The cron entry runs `bonesremote backup run --site <site> --keep-days <retention>` as root, piped to journald through `systemd-cat`; there is no user-facing backup command and no other trigger. Each run archives only the site's `shared/` directory under the site lock as `<site>_<YYYYMMDD_HHMMSS>` (UTC), then prunes archives older than the configured retention (default 30 days). Projects without a passphrase (initialized before this feature) keep their previous behavior. Backups are local to the deployment server: external replication, restoration workflows, and database dumps are the user's responsibility and are not automated.

### Native Artifact Build
Every native deploy resolves the configured branch to one exact committed
revision, exports it, and runs the numbered build scripts locally in Docker with
the pinned `linux/amd64` builder. Docker is the only local build engine. The
CLI pulls the pinned builder automatically when it is not installed locally. The
configured build timeout applies independently to each local Docker build
operation; a value of `0` leaves those operations unbounded. Failed or timed-out
native builds restore mounted ownership before their temporary container state is
removed. Compose builds similarly clean up their generated release image tags.
build receives fixed public contract metadata and values explicitly declared in
the committed `.env.build`; it does not inherit ambient variables or receive the
root `.env`, runtime environment, credentials, host home, SSH agent, or Docker
socket.

The local build packages the complete post-build context as a `tar.gz` and
streams it over SSH with a manifest. BonesRemote checks the site, exact branch
commit, pinned builder identity, compressed length, and SHA-256 digest before
extracting bounded relative paths and safe relative symlinks. It then promotes,
wires shared paths, prepares, seals, activates, restarts, prunes, or rolls back
through the one shared lifecycle. Production hosts create no native build user,
cache, rootless container state, or image store, and no failure falls back to a
server-side build.

Receipt limits are explicit: 64 KiB manifest, 2 GiB compressed payload, 100,000
archive entries, 4 KiB paths and symlink targets, and 4 GiB expanded file
content. Compose image inventories are limited to 128 services. Previous-
installation migration, registries, private credentials, signing, SBOMs, and
resumable upload are deferred; the current transport is a complete artifact
over SSH with a verified manifest and payload digest.

### Deployment Folder
This folder stores build and prepare scripts. Build scripts live in `deployment/build/`, must use the `NN_name.sh` convention (for example, `01_install_deps.sh`, `02_run_build.sh`), and run in lexical order in local Docker with `cwd=/workspace/source`. Other files, including `README.md`, are ignored. The build container receives the exported source tree, scoped local cache, fixed public contract metadata, and committed public `.env.build` values. It does not receive the root `.env`, runtime secrets, `shared/`, `current`, `releases/`, any production application repository, host home, SSH agent, credential stores, or Docker socket. Prepare scripts live in `deployment/prepare/`, use the same naming convention, run in lexical order as the site runtime user with `cwd` set to a runtime-owned candidate release, and are the right place for migrations, cache warmups, and other runtime-state work.

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
Framework templates ship starter overlays that `bonesdeploy init` uses when scaffolding a matching framework. BonesDeploy keeps framework defaults and deployment scripts under `crates/bonesdeploy/assets/frameworks/<fw>/`; canonical infrastructure templates live in BonesInfra and are materialized into `infra/templates/`:

- `frameworks/laravel/`    → Laravel (PHP + PHP-FPM)
- `frameworks/django/`     → Django (Python + Gunicorn)
- `frameworks/angular/`    → Angular (static browser application)
- `frameworks/next/`       → Next.js (Node)
- `frameworks/nuxt/`       → Nuxt (Node)
- `frameworks/sveltekit/`  → SvelteKit (Node)
- `frameworks/vue/`        → Vue (Node)
- `frameworks/rails/`      → Rails (the production host's distribution Ruby + Puma)

Django site setup installs distribution Python, virtualenv support, development headers, and native package build dependencies from the host's configured Debian or Ubuntu APT repositories. The legacy `python_version` setting is accepted but no longer selects an interpreter. The local build installs `requirements.txt` into `.python-packages` and packages release launchers under `.venv/bin`; remote prepare only validates the packaged application, runs migrations, and collects static files. Deployment does not require production package-index access, and incompatible artifacts fail before activation.

Rails site setup installs Ruby, Bundler, development headers, and native gem build dependencies from the host's configured Debian or Ubuntu APT repositories. The exact `ruby_version` setting controls only the local build toolchain. The local build packages the production `vendor/bundle`; remote prepare checks that bundle and runs migrations without installing or compiling gems. Incompatible artifacts fail before activation, and deployment does not require production gem-source access.

Templates inherit the same `bones.toml` schema and customize permissions paths, deployment scripts, and the runtime operations captured in the generated `infra/runtime.py` per project.

Projects materialize the universal BonesInfra wheel and managed templates under `infra/`, prune unselected framework template paths, and preserve project-owned hooks under `infra/custom/`. `bonesinfra runtime apply` executes the complete installed wheel and composes its selected framework runtime with custom provisioning. Updates rematerialize the complete managed template tree before applying the same pruning rules.

Static runtimes deploy from a `web_root` subdirectory of each release that nginx serves. Angular uses the current `@angular/build:application` builder with `dist` as its output base and serves `dist/browser/`. For `is_static = true`, Next.js must set `output: "export"` in `next.config.js`/`next.config.mjs`/`next.config.ts`; otherwise the first deploy fails with *"Static Next.js deployments require out/index.html"*.

### BonesDeploy CLI Commands
- **init**:
  - Loads the root `.env` or collects user input via prompts.
  - Validates the project name, host, SSH port, and Git branch before writing project configuration or scaffolding.
  - For fresh init, waits until prompts complete before writing the root `.env`, committed `.env.build`, `deployment/`, and `infra/`.
  - Updates `.gitignore` to keep `.env` local while leaving `.env.build` trackable.
  - Prints next-step guidance to run `bonesdeploy server setup --yes` and `bonesdeploy site setup --yes` before first deploy.
  - Saves connection and site inputs to the root `.env`.
  - Framework template selection and per-template questions are sourced from `crates/bonesdeploy/src/frameworks/<fw>.rs` (typed Rust, embedded in the binary). BonesDeploy materializes deployment assets from `crates/bonesdeploy/assets/frameworks/<fw>/`, the committed versioned BonesInfra wheel as `infra/bonesinfra-<version>-py3-none-any.whl`, and managed templates under `infra/templates/`.
  - `--template <name>` selects a framework template non-interactively. `--framework-var <key=value>` (repeated) overrides template variables; answers are validated against the template's question schema before writing `.env`.

- **doctor**
  - Root `bonesdeploy doctor` runs both `server doctor` and `site doctor`, reporting both failures when necessary.
  - `bonesdeploy site doctor --local` checks only the local root `.env`, `infra/`, and numbered deployment scripts.
   - Site remote checks open a privileged SSH session, synchronize the narrow backend-specific control-plane snapshot to `/srv/conf/<site>/bones.json`, then run `bonesremote doctor --site <project>`.
    - `bonesremote doctor --site <project>` requires root and reads the synchronized `/srv/conf/<project>/bones.json` snapshot for the runtime backend and Docker ingress port (missing snapshot is reported as pending with guidance). It checks AppArmor availability, imported control-plane state under `/root/.config/bonesremote/sites/<project>/`, runtime user/group constraints, `shared/` and `releases/` layout, and `<project>-nginx.service`. Docker daemon and image checks run only when the synchronized descriptor declares the Docker backend — never inferred from `/run/<site>`.
    - The security audit is read-only and fail-closed. It verifies site identity isolation (unique UIDs/GIDs, no login shells, no cross-site group membership, deploy not in runtime groups), runtime sudo absence, privileged configuration root-control (recursively inspecting systemd, sudoers, nginx, AppArmor, and BonesRemote state plus their parent chains without following symlink targets), and release activation (current must be a valid symlink resolving inside the site's releases directory; active release roots and activation parents must be immutable to the runtime identity). `bonesremote doctor --site <project> --exhaustive` additionally inspects every entry in that active release for permission drift; this can take time on large releases. BonesInfra renders and validates the deploy-user sudoers policy during provisioning rather than probing with fabricated commands during doctor. POSIX ACLs on protected paths are detected through extended attributes and reported as UNVERIFIED. Supplementary groups are collected through `id -G`. Required evidence that cannot be collected is reported as UNVERIFIED and causes doctor to fail.
   - Server doctor verifies Debian 12+ or Ubuntu 24.04+ on `x86_64`, AppArmor, deploy identity, BonesRemote roots and binary, sudoers, firewall, fail2ban, unattended-upgrades, and the etckeeper installation. `--verbose` prints successful remote reports.

- **site manifest**
  - Inspects every project-specific filesystem artifact and managed systemd service expected by the effective framework, service, and SSL strategy. Shared host-wide packages, daemons, and configuration are excluded.
   - Delegates read-only inspection to the embedded BonesInfra runtime as `python -m bonesinfra manifest show --request-stdin`, then renders the returned JSON report in Rust as either the public text tree or unchanged JSON.
  - Uses typed Python declarations inside BonesInfra, resolves path keys through `DeploymentPaths`, and performs read-only PyInfra fact checks.
   - The text tree uses colored status markers for present, missing, and wrong-kind paths and for managed-service health. `--format json` retains complete artifact and service metadata for automation and never includes file contents or secrets.

- **site delete**
  - Runs a local, read-only manifest preflight before any remote mutation and requires the exact configured project name unless `--yes` is supplied.
  - Passes that validated, secret-free inventory directly to BonesInfra, which idempotently stops declared services and removes declared artifacts. After teardown succeeds, BonesRemote removes the site's registration under the deployment lock; missing resources and registration are successful no-ops.
  - Retains local project source, Git configuration, secrets, and caches. It does not remove shared host packages or daemons.

- **site export**
  - Uses the configured root SSH identity to run a fixed `zip -q -r -y - shared` command from the canonical site root and streams the result to the workstation.
  - Includes hidden files such as `shared/.env`, stores symbolic links without following them, and writes a `0600` local ZIP atomically without overwriting an existing path.
  - Is a live best-effort view, does not acquire the deployment lock or stop services, and is not a Borg backup operation.

- **site import**
  - Accepts an export-compatible ZIP rooted beneath `shared/`, requires exact project-name confirmation unless `--yes` is supplied, and replaces rather than merges remote shared data.
  - Streams the archive to a private same-filesystem transaction, rejects unsafe paths, links, types, conflicts, permissions, and resource usage, then stops `<site>.target` only for atomic directory exchange.
  - Ignores archive `shared/.env`, preserves the existing protected remote file, normalizes imported ownership and modes, restarts and verifies registered services, and restores the previous tree after restart failure.
  - Persists root-owned transaction state and uses directory inode identities to recover a process interruption without guessing whether exchange occurred. Application-managed sessions, caches, logs, uploads, and similar files remain opaque.

- **deploy**
  - SSHes into the configured host as `deploy`, synchronizes the narrow backend-specific control-plane snapshot through `sudo -n bonesremote config sync --site <project>`, then runs the existing root-required lifecycle through `sudo -n bonesremote deploy --site <project>`.
  - Does not modify the remote environment. Run `bonesdeploy secrets push` explicitly to replace `shared/.env` from the encrypted local source.
  - Sends the artifact built from the configured local branch; production does not resolve a source branch.

- **server setup**
  - Delegates to `python -m bonesinfra server apply --request-stdin`, feeding a connection-only request (SSH host, user, port) on stdin.
  - Provisions shared packages, hardening (including an sshd drop-in that disables password login for root), firewall, deploy identity, BonesRemote roots and binary, and sudoers.
  - Installs etckeeper and initializes `/etc` as a Git-backed etckeeper repository using package defaults.
  - Every successful mutating BonesInfra provisioning flow (server, site, services, runtime, SSL, helpers) ends with an etckeeper commit of its resulting `/etc` changes; read-only `manifest` and patch flows do not commit.
  - Does not read project runtime, service, framework, DNS, or release settings.

- **site setup**
  - Verifies server readiness before any site mutation.
  - Runs site base, runtime, and site doctor in that order.
  - Site base creates site identities, paths, root-owned control-plane state, and a placeholder release.
  - For projects with a configured Borg passphrase, site base also provisions the scheduled backup: root-only passphrase file, encrypted repository, and the `/etc/cron.d` schedule entry. Borg itself is installed during server setup.
   - Does not push Git or secrets, configure SSL, or deploy a release.
   - Can provision a site afresh after deletion because completed deletion removes the prior BonesRemote registration.

`bonesdeploy update` resolves the latest published GitHub release, validates that
its `v<version>` tag matches both package manifests, clones that exact tag for
patches and scaffold updates, installs the matching crates.io `bonesdeploy`, and
downloads the matching static `x86_64` `bonesremote` asset. ARM hosts fail
clearly because release binaries currently support only `x86_64` Debian/Ubuntu.

- **site runtime**:
  - Reapplies the configured runtime settings from the root `.env` to the host and provisions the selected framework's runtime.
   - Delegates to the embedded `bonesinfra` runtime by running `python -m bonesinfra runtime apply --request-stdin` against the configured host as the configured `ssh_user`, feeding the typed site request on stdin.
  - Native sites run project and framework provisioning. Compose sites configure Docker's official Debian/Ubuntu apt repository, install Docker Engine and the Compose plugin, render the generic Compose systemd unit, and optionally configure nginx for `BONES_COMPOSE_PORT`.
  - Configures per-site runtime assets: AppArmor profile, nginx router + per-site config + systemd service, and runs `bonesremote doctor`.
  - Does not install, configure, or start Cloudflare Quick Tunnels. Temporary public ingress is an explicit `site tunnel` lifecycle.
  - Does not handle SSL; use `site ssl` for TLS configuration.

- **site ssl**
  - Delegates to the embedded `bonesinfra` runtime by running `python -m bonesinfra ssl apply --request-stdin` against the configured host as root, feeding the typed site request on stdin.
  - Uses certbot with a webroot challenge to obtain/renew certificates for the configured domain.
  - Re-renders the per-site runtime nginx router with TLS enabled, listening on 443 and redirecting HTTP to HTTPS.
  - Separate from `site runtime` to keep certificate management decoupled from app runtime concerns.

- **site tunnel**
  - `start [--yes]` explicitly installs Cloudflared, creates a loopback-only root-nginx route to the per-site nginx Unix socket, and enables/starts `<site>-cloudflared.service` independently of `<site>.target`.
  - `status` reports the active ephemeral `trycloudflare.com` URL from the service journal, reports startup while the URL is pending, or reports that the tunnel is stopped.
  - `stop [--yes]` stops/disables the service and removes its unit and nginx route. The server-wide Cloudflared package remains installed for other sites.
  - Native sites are supported. Compose sites require `BONES_COMPOSE_PORT` so nginx remains the origin; application containers are never exposed directly by this feature.

- **rollback**
  - SSHes into the configured host and runs `bonesremote release rollback --site <project>`, which acquires the site lock and repoints `current` to the previous release without rebuilding, then restarts `<project>.target`. If the restart fails, the original release is restored and restarted.

- **secrets**
  - Subcommands: `init`, `edit`, `push`.
  - Manages the GPG-encrypted production environment at `infra/secrets/.env.gpg`.
  - First initialization merges missing framework keys into the production environment, generates framework-native settings, validates, and encrypts the result. If the encrypted file already exists it is returned untouched; later additions go through `secrets edit`.
  - `secrets edit` decrypts `infra/secrets/.env.gpg` for editing and re-encrypts on save.
  - `secrets push` validates the decrypted content and sends it to the typed `bonesremote shared install-environment` operation, which atomically replaces remote `shared/.env` under the same `SiteMutation` lock used by deploy and shared import. It does not read, merge, or upload the local root `.env`.

- **skill**
  - Embedded documentation for AI agents, plus the state-aware next-step compass.
  - `bonesdeploy skill` prints the orientation doc (`SKILL.md`) baked into the binary.
  - `bonesdeploy skill list` prints the names of every embedded topic doc.
  - `bonesdeploy skill doc <name>` prints a specific topic doc (`commands`, `workflows`, `methodology`).
  - `bonesdeploy skill next [--format text|json]` inspects `.env` and the remote host, then suggests the next prompt-free command across `uninitialized`, `server_missing`, `site_missing`, `ssl_missing`, and `ready` states. A Quick Tunnel is optional and does not affect readiness.
  - Topic docs are markdown files under `crates/bonesdeploy/assets/skill/` and are embedded with `rust-embed` alongside `kit/` and `frameworks/`.
- **version**:
  - Echoes the installed `bonesdeploy` version.

### BonesRemote CLI Commands
- **Release commands** live under `bonesremote release ...`
- **Service commands** live under `bonesremote service ...`
- **Shared commands** live under `bonesremote shared ...`; `import` receives a ZIP on stdin and `install-environment` receives validated dotenv plaintext. Both derive paths from `--site`, require root, and acquire the site mutation lock.
- **config sync**:
  - `--site <name>` receives the sanitized control-plane descriptor as JSON on stdin, validates it, and atomically installs root-owned `/srv/conf/<site>/bones.json`. BonesDeploy invokes it through sudo before deploy.
- **deploy**:
  - Runs the full deployment lifecycle as root after the `deploy` SSH identity invokes the exact allowed command through sudo. It receives the locally built artifact; Git push is not part of deployment.
   - Orchestrates one shared lifecycle: artifact receipt → candidate release → shared wiring → backend preparation → seal release → activate → restart `<site>.target` → post-deploy pruning. Native sites receive a verified local Docker artifact and run host prepare scripts. Compose sites receive a verified image artifact, load its images, and skip numbered prepare scripts.
   - Native preflight validates nginx. Compose activation reconciles the stack from `current` with `up --detach --no-build --pull never --remove-orphans --wait`. On failure before activation, automatically drops the staged release. If the service restart fails after activation,
    restores and restarts the previous release before dropping the failed release.
  - `--site <name>`: validated site identifier used to load the synchronized control-plane snapshot and existing root-owned deployment state
- **doctor**:
  - Host mode checks `bonesremote` in `PATH`, the supported Debian 12+/Ubuntu 24.04+ `x86_64` platform, AppArmor support, and the deploy-user sudoers drop-in.
  - `--site <name>` loads `/srv/conf/<name>/bones.json` for the explicit runtime backend, checks the imported site boundary, and validates `<site>.target`. Compose checks include Docker Engine/plugin availability, Compose configuration, container health, optional loopback ingress, and one reduced-guarantee warning.
- **release stage**
  - Creates one exclusively named candidate in the existing root-owned lifecycle; the candidate is sealed as `root:<site>` before activation.
- **release wire**
	- Wires shared paths into the promoted candidate release, replacing declared paths with symlinks to the shared directory.
- **release activate**
  - Atomically validates and switches `current` to the staged release, restarts services, and restores the previous release if verification fails.
- **release drop-failed**
	- Deletes a failed staged release and clears staged release state.
- **release recover**
	- Quarantines malformed `active-deployment.json` state into the site's `recovery/` directory. It first acquires the site deployment lock, proving no deployment process is alive, so malformed state written by a crash can never wedge status, cancellation, or idle checks while a deploy runs.
- **release rollback**
	- Acquires the site deployment lock and requires an idle site, then repoints `current` to the previous release. It is transactional: after switching `current`, it restarts and verifies the target, and if verification fails it restores the original release and restarts it before returning an error.
- **service restart**
	- Restarts the per-site systemd lifecycle target (`<project>.target`), which restarts all registered site services. Requires root privileges.

- **backup run**
	- Cron-invoked internal operation (requires root) for scheduled backups. It acquires the site deployment lock, reads the root-only passphrase file, creates one Borg archive of the site's `shared/` directory named `<site>_<YYYYMMDD_HHMMSS>` (UTC), and prunes archives older than `--keep-days`. The passphrase is passed to Borg through its environment, never as a command argument, and is never logged. Not intended for manual use.

BonesInfra owns site service membership. BonesRemote restarts exactly `<project>.target` for deploy and rollback.
- **version**:
  - Echoes the installed `bonesremote` version.

## Security Notes
- Sudo access for the deployment user is strictly limited by the `/etc/sudoers.d/bonesdeploy` drop-in provisioned by `bonesinfra` on the host.
- No broader sudo privileges are granted — the deploy user cannot run arbitrary commands as root, read root-owned files, or write outside their owned directories.
- All release artifacts are created with the setgid bit on `releases/` so the runtime group inherits read access without needing a post-deploy chown.
- The build workspace (`build/`) is private to the deploy user (`0700`), invisible to other processes.
- Native runtime processes are sandboxed via systemd, AppArmor, and dedicated runtime identities. Compose files are trusted privileged input and may select container authority outside that model; BonesDeploy does not automatically mount the Docker socket.

## Flow
- User runs `bonesdeploy init`, and the procedures outlined above are executed.
- User can make any changes to their deployment scripts in `deployment/` and project infrastructure in `infra/custom/`.
- `bonesdeploy site doctor` checks the local and site environment, including whether the configured deploy branch resolves locally. Root `bonesdeploy doctor` composes server and site diagnostics.
- Doctor uses exit status for actionable failures; site setup does not require a first Git push.
- User runs `bonesdeploy deploy` to perform the actual remote release deployment.

### Primary Deploy Flow

1. `bonesdeploy deploy` resolves and exports the configured committed revision, then builds a complete local artifact. Only after packaging succeeds does it check or push production secrets, SSH into the configured host as `deploy`, synchronize the sanitized control-plane snapshot through `sudo -n bonesremote config sync --site <site>`, and run `sudo -n bonesremote deploy --site <site>` with that retained artifact.
2. `bonesremote deploy`, running as root through the exact sudoers grant, loads the synchronized snapshot and orchestrates the existing pipeline:
   - **stage_release** — Create timestamped release state
    - **artifact_receipt** — Verify the manifest, digest, and bounded archive before extracting it into a temporary context
   - **release_promote** — Copy verified artifact contents into a runtime-owned candidate release
    - **wire_shared** — Symlink declared shared paths into the candidate release
    - **release_prepare** — Native: run `deployment/prepare/*.sh` as the site runtime user. Compose: skip numbered prepare scripts; migrations belong in the stack.
    - **release_finalize** — Seal the prepared release as `root:<site>`
    - **activate_release** — Atomically repoint `current`
    - **restart_services** — Restart `<site>.target`. The Compose unit reconciles the stable `bonesdeploy-<site>` project from `current` and waits for readiness.
    - **post_deploy** — Prune old releases beyond `releases`
    - On failure: **drop_failed_release** — Clean up staged release. Activation failure restores the previous release definition; Compose named volumes persist and their data is not rolled back.
