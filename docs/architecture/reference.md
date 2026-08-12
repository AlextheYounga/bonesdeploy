# Architecture

## 1. System Overview

BonesDeploy is a remote release deployment tool for simple Debian/Ubuntu Linux servers. It produces two Rust binaries:

- **`bonesdeploy`** — local CLI for setup, provisioning, deployment, and management. Runs on the developer's workstation.
- **`bonesremote`** — server-side release lifecycle executor. Runs as root on the deployment host and is the sole mutator of per-site deployment state.

A third component, **`bonesinfra`**, is an embedded Python provisioning runtime (pyinfra-based) that handles server bootstrap, framework-specific provisioning, database services, SSL, and infrastructure migrations. It is compiled into the `bonesdeploy` binary via `rust-embed` and materialized on demand into a Python venv under `~/.cache/bonesdeploy/bonesinfra`.

The system is organized as a Cargo workspace of four crates, with a deliberate split between the *declarative model* (`bonesdeploy-core` — config schema, paths, validation) and the *imperative agents* (`bonesdeploy` and `bonesremote` — the binaries that produce behavior).

### High-level flow

```
Developer workstation                           Deployment server
┌─────────────────────────┐                   ┌────────────────────────────┐
│ bonesdeploy init        │                   │                            │
│ bonesdeploy remote ...  │── SSH/bonesinfra──▶│ bonesinfra (Python)        │
│   (bootstrap, runtime,  │                   │   pyinfra provisioning     │
│    services, ssl,       │                   │                            │
│    helpers)             │                   │                            │
│                         │                   │                            │
│ bonesdeploy push        │── git push ──────▶│ config-pre-receive hook    │
│   (.bones -> config     │                   │   └─ bonesremote site      │
│    repo)                │                   │      receive               │
│                         │                   │                            │
│ bonesdeploy deploy      │── SSH ───────────▶│ bonesremote deploy --site  │
│ bonesdeploy rollback    │                   │ bonesremote release ...    │
│ bonesdeploy releases    │                   │ bonesremote doctor         │
│ bonesdeploy doctor      │                   │ bonesremote status         │
└─────────────────────────┘                   └────────────────────────────┘
```

## 2. Repository Structure

```
crates/
├── bonesdeploy-core/       # Foundation: config schema, path constants, validation
├── bonesdeploy/            # Local CLI binary (init, deploy, doctor, remote, secrets, etc.)
├── bonesremote/            # Remote server binary (deployment lifecycle, site state, doctor)
└── bonesinfra/             # Embedded Python provisioning runtime
    ├── src/lib.rs          # Rust shim: embeds Python, materializes venv, run() API
    └── python/             # Python package: pyinfra, frameworks, services, patches

e2e/                        # End-to-end tests (LXC containers, framework fixtures)
docs/
├── agents/                 # Agent plans, conventions
├── architecture/           # Supplementary architecture docs (security model)
└── security/               # Security invariants
tests/cleancode/            # Static analysis checks for clean-code conventions
```

### Crate dependency graph

```
bonesdeploy ──────────────▶ bonesdeploy-core
    │                            ▲
    └────────▶ bonesinfra ───────┘

bonesremote ─────────────▶ bonesdeploy-core
```

`bonesdeploy-core` is a leaf dependency with only `anyhow`, `serde`, and `toml`. `bonesinfra` depends on `bonesdeploy-core` solely for the `paths::bones_cache_root()` function. `bonesremote` uses `bonesdeploy-core` heavily for all path constants, config types, and naming conventions.

## 3. Core Concepts

### 3.1 `Bones` — Central Configuration Object

**Responsibility:**
The canonical representation of a project's deployment configuration. Stored as `bones.toml` in the `.bones/` directory. Contains all information needed to provision, deploy, and manage a site.

**Lives in:**
`crates/bonesdeploy-core/src/config.rs` (struct definition), `crates/bonesdeploy-core/src/app.rs` (the `App` sub-struct)

**Structure:**
```
Bones
├── app: App              # project_name, host, port, branch, domain, ssl, etc.
│   ├── .server           #   ssh_user, host, port (TOML: [app.server])
│   ├── .dns              #   domain, preview_domain, email, ssl_enabled (TOML: [app.dns])
│   └── .deploy           #   branch, deploy_on_push, releases_keep, repo_path (TOML: [app.deploy])
├── runtime: Runtime      # template (framework), web_root, backend, node_version, shared, permissions, extra
├── services: Services    # services: Vec<String> (postgres, redis, etc.)
└── build: Build          # timeout_seconds
```

**Serialization model:**
`Bones`, `Runtime`, `Build`, `Services`, and `Shared` use derived serde `Serialize`/`Deserialize`. Only `App` uses **manual** serialization to reshape the TOML layout (sections like `[app.server]`, `[app.dns]`, `[app.deploy]`) into the flat Rust struct.

**Used by:**
Both binaries (`bonesdeploy`, `bonesremote`) and the Python provisioning layer. `bonesdeploy` loads it from `.bones/bones.toml`; `bonesremote` loads it from `/root/.config/bonesremote/sites/<site>/bones.toml`.

**Depends on:**
Serialization crates (`serde`, `toml`), path derivation functions (`paths` module), validation functions.

**Extension model:**
`Runtime.extra` captures arbitrary framework-specific TOML keys via `#[serde(flatten)]` into a `BTreeMap<String, toml::Value>`. New framework configuration fields are added here rather than extending the `Runtime` struct. Derived `BONES_*` environment variables (e.g. `BONES_RUNTIME_NODE_VERSION`) are extracted from the config at build time.

---

### 3.2 Path Constants (`paths` module)

**Responsibility:**
The single source of truth for all product-owned filesystem paths — local workstation paths, server-side paths, and constant filenames. All other modules derive subpaths from these constants rather than introducing independent path roots.

**Lives in:**
`crates/bonesdeploy-core/src/paths.rs`

**Exports:**
~60 `pub const` path/filename constants plus ~25 path derivation functions (e.g. `default_repo_path_for(project_name)`, `ssl_certificate_path(domain)`, `bonesremote_site_root(site)`).

**Usage pattern:**
All code references these constants by name. No hardcoded path strings exist outside this module. Both Rust binaries and the Python layer (via its own `DeploymentPaths` class) maintain derived representations of these paths.

**Extension model:**
New paths that represent product-owned layout (directories, filenames, install locations) are added here. One-off runtime paths derived from user input may remain local to their consumer.

---

### 3.3 `SiteMutation` — Serialization Guard

**Responsibility:**
Bundles a deployment lock, a validated configuration snapshot, and a site identity into a single guard object. Every operation that mutates per-site state must receive a `SiteMutation`. This enforces that the deployment serialization lock and confused-deputy check (`project_name == site`) are always applied together.

**Lives in:**
`crates/bonesremote/src/release/site_mutation.rs`

**Used by:**
All bonesremote commands that change site state: `deploy`, `release rollback`, `release kill`, `release drop-failed`, `release prune`, `site import`, `site receive`, `service restart`.

**Depends on:**
`DeploymentLock` (file lock), `Bones` (config validation).

**Constructors:**
- `acquire(site)` — standard case: lock, then load config
- `acquire_with_config(site, config)` — first-import case: lock, adopt pre-validated config
- `adopt(site, config, lock)` — cancellation case: adopt config loaded before terminating running deployment

**Extension model:**
Single implementation. New site-mutating commands should use `SiteMutation::acquire()`.

---

### 3.4 `DeploymentPhase` — Deployment State Machine

**Responsibility:**
Represents the 11 phases of a deployment's lifecycle. Each phase is persisted to disk as part of a `DeploymentRecord`. The phase determines whether the site is considered idle (ready for a new deployment) and whether cancellation is safe.

**Lives in:**
`crates/bonesremote/src/release/state/record.rs`

**Phases:**
| Phase | Meaning |
|-------|---------|
| `Created` | Release directory created |
| `SourceExported` | `git archive` completed into build context |
| `Built` | Build scripts finished inside container |
| `Promoted` | Artifacts copied out of container into release dir |
| `Prepared` | Shared paths wired, prepare scripts ran |
| `Sealed` | Container removed, release directory made immutable |
| `Activated` | `current` symlink atomically switched |
| `Verified` | Services restarted successfully |
| `Completed` | Old releases pruned, context cleaned up |
| `CleanupPending` | Post-commit maintenance failed (non-blocking) |
| `Failed` | Pre-commit failure being aborted |

**Key predicates:**
- `is_committed()` — `true` for `Activated`..`CleanupPending`. A committed record means the site can accept a new deployment.
- `may_have_mutated_runtime()` — `true` from `Promoted` onward. Cancellation may be refused without explicit policy.

**Extension model:**
Add a phase variant only for a new deployment stage. Existing phase transitions are defined in the lifecycle orchestrator (`commands/deploy/lifecycle.rs`).

---

### 3.5 `DeploymentRecord` — In-Flight Deployment Metadata

**Responsibility:**
Carries the identity and crash-detection fields for an in-progress deployment. Stored in the centralized `SiteState` JSON document.

**Lives in:**
`crates/bonesremote/src/release/state/record.rs`

**Key fields:**
`release`, `source_revision`, `phase`, `pid`, `process_start_ticks`, `started_at`, `previous_release`, `context` (build context path), `error`.

The `pid` + `process_start_ticks` pair survives PID reuse for crash detection: if the PID is gone or `/proc/<pid>/stat` start ticks don't match, the deployment is stale.

---

### 3.6 `DeploymentLock` — Per-Site Advisory File Lock

**Responsibility:**
Serializes all site mutations (deployments, rollbacks, imports, cancellations) across concurrent processes. Uses POSIX `flock()` with `try_lock()` — if another process holds the lock, the operation fails immediately with a clear error directing the user to check `bonesdeploy releases`.

**Lives in:**
`crates/bonesremote/src/release/state/mod.rs` (lines 62-95)

**Crash safety:**
The lock is tied to a file descriptor. If the process crashes, the kernel releases the lock.

---

### 3.7 `SiteState` — Centralized Per-Site State Document

**Responsibility:**
Single source of truth for a site's runtime-mutated state. Replaces older separate files (`active-deployment.json`, `staged-release`). Migrates legacy files on first read.

**Lives in:**
`crates/bonesremote/src/release/state/store.rs`

**Schema:**
```json
{
  "schema_version": 1,
  "active": { ... DeploymentRecord ... },
  "staged_release": "20260804_190321-46a0b75c-a7f2"
}
```

All writes use `atomic_write` (temp file, fsync, rename, directory fsync). Malformed state on read triggers quarantine via `release recover`.

---

### 3.8 Framework Pattern (Rust side)

**Responsibility:**
Defines per-framework prompt questions, validation, configuration overrides, and `.env.build` examples for the `bonesdeploy init` wizard.

**Lives in:**
`crates/bonesdeploy/src/frameworks.rs` (dispatch registry) and `crates/bonesdeploy/src/frameworks/<name>.rs` (per-framework modules).

**Per-framework contract:**
Each framework module exports these functions:
- `questions() -> &'static [Question]` — interactive prompt schema (`Text`, `Bool`, `Choice` kinds)
- `validate_answers(answers) -> Result<()>` — invoked by the dispatch
- `configure(cfg: &mut Bones)` — (optional) post-answer config augmentation (e.g. static builds override `web_root`)
- `environment_example(...) -> String` — generates a sample `.env.build` content

**Existing frameworks:**
`django`, `laravel`, `next`, `nuxt`, `rails`, `sveltekit`, `vue`

**Extension model:**
Add a new module under `src/frameworks/<name>.rs`, implement the four-function contract, and register it in `src/frameworks.rs`. Also add corresponding Python framework files (see §3.9).

---

### 3.9 Framework Pattern (Python side)

**Responsibility:**
Declares what artifacts and services a framework owns (manifest) and how to provision them (runtime).

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/frameworks/<name>/`

**Package structure per framework:**

| File | Exports | Purpose |
|------|---------|---------|
| `__init__.py` | (empty) | Makes the directory an importable Python package |
| `manifest.py` | `artifacts(ctx)`, `services(ctx)`, `mode(ctx)` | Declares all paths and systemd services the framework owns |
| `runtime.py` | `deploy(ctx)` | Orchestrates framework provisioning (language install, render configs, start services) |
| `custom.py` | `deploy(ctx)` | User extension point (no-op default; materialized into project's `.bones/infra/` for editing) |
| `templates/` | Jinja2 templates | Nginx, AppArmor, and other framework configuration templates |

**Framework discovery:**
`project.py` reads `[runtime].template` from `bones.toml`. It first checks for a
project-local `.bones/infra/` package. If present, the local package takes strict
precedence and the built-in is never consulted. If absent, it imports the
canonical built-in at `bonesinfra.frameworks.<name>`. Users materialize a
framework's package into their `.bones/infra/` directory via `bonesinfra project
materialize`, after which the local copy is authoritative.

**Extension model:**
Add a new package under `frameworks/<name>/` with all five components above.
Register the name in `project.py`'s `BUILTIN_FRAMEWORKS` allowlist. Add
corresponding Rust-side framework module and deployment assets.

---

### 3.10 `DeployContext` — Python Config Object

**Responsibility:**
The Python-side equivalent of `Bones`. Created from `bones.toml` and passed to all framework, service, and infrastructure functions.

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/config/context.py`

**Structure:**
```python
@dataclass
class DeployContext:
    app: AppConfig            # project_name, repo_path, project_root, server (host, ssh_user, port)
    runtime: RuntimeConfig    # backend, web_root, runtime_user, runtime_group, data (extra keys)
    services: ServicesConfig  # tuple of service names
```

`template_data(ctx)` flattens the context into a dict for Jinja2 template rendering.

---

### 3.11 `DeploymentPaths` — Python Canonical Paths

**Responsibility:**
Computes all server-side filesystem paths from the project name, repo path, root path, and web root. Used by templates and framework manifest declarations.

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/config/paths.py`

107 frozen dataclass fields plus helper methods (`systemd_service(name)`, `apparmor_profile(name)`, etc.). Mirrors `bonesdeploy-core::paths` but with server-side resolution.

---

### 3.12 Embedded Assets

**Responsibility:**
Two independent `rust-embed` systems embed non-Rust content into the binaries:

**In `bonesdeploy`:**
Three separate embedded asset collections under `src/infra/assets/`:
- `kit.rs` — deployment shell functions, `.gitignore`
- `skill.rs` — AI agent orientation and topic docs (SKILL.md, commands, workflows, methodology)
- `frameworks.rs` — per-framework TOML defaults, deployment scripts, `.env.build` examples

**In `bonesinfra`:**
`PythonSource` struct embeds the entire `python/` directory (excluding venv, pycache, tests, docs). Materialized on first use to `~/.cache/bonesdeploy/bonesinfra` with content-hash invalidation (if any embedded file changes, the checkout is re-extracted and reinstalled).

---

### 3.13 pyinfra Runner — Remote Provisioning Engine

**Responsibility:**
Provides SSH-based remote server provisioning using the pyinfra library. Connects to the target host, plans operations in a graph, then executes them in batch.

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/pyinfra/runner.py`

**Flow:**
```
run(ctx, deploy=callback)
  ├─ build_inventory(ctx)        # host, SSH user, port
  ├─ connect_all(state)          # establish SSH session
  ├─ deploy(ctx)                 # record operations (does not execute yet)
  └─ run_ops(state)              # execute all queued operations
```

The `operations.py` wrapper provides convenience functions (`mkdir()`, `render()`, `letsencrypt_cert_paths()`) that wrap pyinfra operations with `_sudo=True`.

---

### 3.14 Patches System

**Responsibility:**
Version-gated migration steps that run during `bonesdeploy update`. Each patch has a semver introduction version and independently tracked completion markers per project and scope (local/remote).

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/patches/`

**Patch definition:** `Patch(identifier, introduced_in, local_apply)` in `registry.py`.
**Execution:** `patches apply --target-version <ver> --scope local|remote`.

Local patches run on the workstation (e.g. git init on the config repo). Remote patches connect as root via pyinfra (e.g. migrate the `.bones.git` repo location). Both check marker files before execution so already-applied patches are skipped.

**Extension model:**
Add a `Patch` to `registry.py` with an `introduced_in` version. Implement the local/remote apply functions. Version gates are semver comparisons — only patches introduced up to the target version are selected.

---

### 3.15 Language Runtimes

**Responsibility:**
Abstracts the installation and configuration of programming language runtimes (Ruby, Python, PHP, Node.js) on the deployment server.

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/services/languages/`

**Pattern:**
`LanguageRuntime` ABC with `install(ctx)`, `install_version(ctx, version)`. Each concrete implementation (e.g. `PHPRuntime`, `PythonRuntime`) defines a `config_key`, `default_version`, and `version_pattern`. Framework `runtime.py` modules select and invoke the appropriate language runtime.

---

### 3.16 Database Services

**Responsibility:**
Provisions database engines (PostgreSQL, MariaDB, MySQL, MongoDB, Valkey, Redis) on the deployment server. Each is a `RuntimeService` subclass.

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/services/runtime/`

**Contract:**
`provision(ctx)` — installs, creates user/database, seeds connection values to `shared/.env`.
`manifest_artifacts(ctx)` / `manifest_services(ctx)` — declares paths and systemd units for manifest inspection.

Registered in the `SERVICES` dict in `__init__.py`. Activated via `[services].services = ["postgres", "redis"]` in `bones.toml`.

---

### 3.17 Manifest System

**Responsibility:**
Read-only remote inspection of a deployed site. Checks whether declared artifacts exist on the server and whether managed services are running.

**Lives in:**
`crates/bonesinfra/python/src/bonesinfra/manifest.py`

**Artifact types:** `file`, `directory`, `link` — each associated with an owner (`framework`, `runtime`, `setup`, `ssl`, `docker`).

**Inspection:** Uses pyinfra facts (`File`, `Directory`, `Link`, `SystemdStatus`, `SystemdEnabled`) to resolve actual state during an SSH session. Reports as text (human-readable) or JSON.

Artifacts and services are collected by merging: common artifacts, framework-specific artifacts from `manifest.py`, database service artifacts, and SSL artifacts.

---

### 3.18 Doctor (bonesremote)

**Responsibility:**
Validates server environment, per-site configuration, and security posture. Read-only — makes no changes.

**Lives in:**
`crates/bonesremote/src/commands/doctor/`

**Check categories:**
- `system.rs` — Debian/Ubuntu distribution, Podman availability
- `site.rs` — config state, bare repo, branch ref, thin hook, user/group identities, directory layout
- `services.rs` — systemd target membership and service active state
- `apparmor.rs` — kernel support and service
- `security/` — identity isolation (unique UID/GID per site, no login shells, no cross-site group membership), runtime sudo absence, privileged config root-control, release activation immutability, POSIX ACL detection

---

### 3.19 Secrets System (GPG)

**Responsibility:**
Manages GPG-encrypted environment secrets under `.bones/secrets/`. The GPG home is isolated at `~/.local/share/bonesdeploy/gnupg`. Per-project keys are auto-generated.

**Lives in:**
`crates/bonesdeploy/src/commands/secrets/`

**Subcommands:**
- `init` — creates `.bones/secrets/.env.gpg` with framework defaults
- `edit` — decrypts, opens in `$EDITOR`, re-encrypts on save
- `push` — decrypts and uploads plaintext to remote `shared/.env` via SSH

---

## 4. Major Execution Flows

### 4.1 `bonesdeploy init`

```
Cli::Init
  └─ commands/init/mod.rs::run()
       ├─ frameworks.rs            # framework template selection
       ├─ init/config.rs           # collect config (interactive or --template/--framework-var)
       │    ├─ ui/prompts.rs       # inquire-based interactive prompts
       │    ├─ infra/git.rs        # infer remote connection details from git remotes
       │    └─ frameworks/<fw>.rs  # framework-specific questions and validation
       ├─ init/scaffold.rs         # filesystem materialization
       │    ├─ infra/assets/kit.rs # scaffold deployment functions, .gitignore
       │    ├─ infra/assets/frameworks.rs  # scaffold per-framework TOML defaults + scripts
       │    ├─ config.rs::save()   # write bones.toml with annotations
       │    └─ bonesinfra::run()   # "project materialize" to copy framework templates
       ├─ secrets/gpg.rs           # generate GPG key pair
       ├─ secrets/mod.rs           # create default .env.gpg
       └─ infra/git.rs             # add git remote for config repo
```

### 4.2 `bonesdeploy remote bootstrap` (server provisioning)

```
Cli::Remote::Bootstrap
  └─ commands/remote/bootstrap.rs::run()
       ├─ bonesinfra::run_with_stdin("setup", "apply", stdin_json)
       │    └─ Python: cli/commands/setup/__init__.py::deploy_setup()
       │         ├─ packages.py       # apt install base + supplementary packages
       │         ├─ users.py          # create deploy user, runtime users, groups
       │         ├─ directories.py    # create git repos, project directories
       │         ├─ sudoers.py        # write /etc/sudoers.d/bonesdeploy
       │         ├─ firewall.py       # configure UFW, fail2ban
       │         ├─ bonesremote.py    # download + verify bonesremote binary
       │         ├─ placeholder.py    # create placeholder release
       │         ├─ disable_algif_aead.py  # kernel module blacklist
       │         └─ image_store.py    # configure rootless podman image store
       └─ bonesinfra::run("runtime", "apply")  # apply runtime provisioning
```

### 4.3 `bonesdeploy deploy`

```
Cli::Deploy
  └─ commands/deploy.rs::run()
       ├─ push_state::run()           # publish .bones to config repo
       │    └─ git add/commit/push to root@<host>:.../<project>.bones.git
       │         └─ (remote) config-pre-receive hook
       │              └─ bonesremote site receive
       │                   └─ site.rs::receive()  # validate + atomically replace site state
       ├─ secrets::push()             # decrypt + upload .env to shared/.env
       └─ SSH: bonesremote deploy --site <site>
            └─ commands/deploy/lifecycle.rs::run_full()
                 ├─ SiteMutation::acquire(site)   # lock + validate config
                 ├─ ensure_site_idle(site)        # verify no in-flight deployment
                 ├─ run_staged_deployment()
                 │    ├─ Stage:    stage::run()         → Created
                 │    ├─ Export:   checkout::run()       → SourceExported
                 │    ├─ Build:    build::run()          → Built
                 │    ├─ Promote:  build::promote()      → Promoted
                 │    ├─ Prepare:  wire_shared + prepare → Prepared
                 │    ├─ Seal:     build::finalize()     → Sealed
                 │    ├─ Preflight: validate_ready (nginx -t)
                 │    ├─ Activate: activate::run()       → Activated
                 │    ├─ Verify:   service::run()        → Verified
                 │    └─ Maintain: prune + cleanup       → Completed
                 └─ (on failure) abort / rollback / cleanup_pending
```

### 4.4 `bonesdeploy doctor`

```
Cli::Doctor
  └─ commands/doctor.rs::run()
       ├─ Local checks:
       │    ├─ .bones exists + is symlink
       │    ├─ bones.toml loads
       │    ├─ deployment scripts follow NN_name.sh convention
       │    ├─ local branch exists
       │    └─ pre-push hook installed (when deploy_on_push)
       └─ Remote checks (unless --local):
            └─ SSH: bonesremote doctor --site <site>
                 ├─ doctor/system.rs      # distro, podman
                 ├─ doctor/apparmor.rs    # AppArmor support
                 ├─ doctor/site.rs        # config state, repo, users, layout
                 ├─ doctor/services.rs    # systemd target + service health
                 └─ doctor/security/      # identity isolation, sudo, paths, release immutability
```

### 4.5 `bonesdeploy remote runtime`

```
Cli::Remote::Runtime
  └─ commands/remote/runtime.rs::run()
       ├─ bonesinfra::run("runtime", "apply", "--config", "...")
       │    └─ Python: project.load_runtime(config)
       │         └─ imports runtime.py (local infra/ or built-in frameworks/<fw>/)
       │              └─ runtime.deploy(ctx)
       │                   ├─ linux/runtime.setup(ctx)     # AppArmor + nginx router
       │                   ├─ languages/<lang>.install()   # install language runtime
       │                   ├─ linux/application.deploy_server() or nginx/site.render_*()
       │                   ├─ frameworks/<fw>/custom.deploy(ctx)  # user hook
       │                   └─ linux/runtime.start_services(ctx)
       └─ SSH: bonesremote doctor --site <site>
```

---

## 5. Extension Points

| Need | Extend / reuse | Existing example | Location |
|------|---------------|-----------------|----------|
| Add a web framework | Rust: `frameworks/<fw>.rs`, Python: `frameworks/<fw>/` (`__init__.py`, `manifest.py`, `runtime.py`, `custom.py`, `templates/`) | `laravel`, `django` | `crates/bonesdeploy/src/frameworks/` and `crates/bonesinfra/python/src/bonesinfra/frameworks/` |
| Add a database service | Python: `services/runtime/<name>.py` + register in `SERVICES` dict | `postgres.py`, `redis.py` | `crates/bonesinfra/python/src/bonesinfra/services/runtime/` |
| Add a language runtime | Python: `services/languages/<name>.py`, extend `LanguageRuntime` ABC | `php.py`, `python.py` | `crates/bonesinfra/python/src/bonesinfra/services/languages/` |
| Add a CLI command (bonesdeploy) | `commands/<name>.rs` + variant in `cli/args.rs::Command` enum | `commands/status.rs` | `crates/bonesdeploy/src/commands/` |
| Add a CLI command (bonesremote) | `commands/<name>.rs` + variant in `cli/args.rs::Command` enum | `commands/status.rs` | `crates/bonesremote/src/commands/` |
| Add a remote provisioning step | Python: `cli/commands/<group>/__init__.py` + Typer command group | `cli/commands/setup/` | `crates/bonesinfra/python/src/bonesinfra/cli/commands/` |
| Add a migration patch | Python: `patches/registry.py` — `Patch(id, version, apply_fn)` | `0001-config-repo` | `crates/bonesinfra/python/src/bonesinfra/patches/` |
| Add a doctor check | `commands/doctor/<category>.rs` in bonesremote | `doctor/site.rs` | `crates/bonesremote/src/commands/doctor/` |
| Add a new config field | `Runtime.extra` (TOML map) for framework-specific; add to struct for global | `php_version` in laravel | `crates/bonesdeploy-core/src/config.rs` |
| Add a new shared constant/path | `paths` module in bonesdeploy-core | `DEFAULT_WEB_ROOT` | `crates/bonesdeploy-core/src/paths.rs` |
| Embed new static assets | `rust-embed` derive in appropriate asset module | `KitAssets`, `FrameworkAssets` | `crates/bonesdeploy/src/infra/assets/` |
| Override framework provisioning | Materialize framework, edit `.bones/infra/runtime.py` + `.bones/infra/custom.py` | User-local `.bones/infra/` | `.bones/infra/` |
| Add a deployment script | `NN_name.sh` in `.bones/deployment/build/` or `prepare/` | `01_install_deps.sh` | `.bones/deployment/{build,prepare}/` |

---

## 6. Dependency and Ownership Rules

### Crate-level boundaries

- **`bonesdeploy-core`** may not depend on any other workspace crate. It is the foundation.
- **`bonesdeploy`** depends on `bonesdeploy-core` and `bonesinfra`. It is the local CLI.
- **`bonesremote`** depends on `bonesdeploy-core`. It is the remote agent. It does NOT depend on `bonesinfra`.
- **`bonesinfra`** depends on `bonesdeploy-core` (minimally, for cache path). It is the Python runtime bridge.

### Config ownership

- `bonesdeploy-core` defines the canonical `Bones` struct, all path constants, and validation functions.
- Both binaries load config via `bonesdeploy_core::config::load()` or `bonesdeploy_core::config::load_runtime()`.
- Config is saved by `bonesdeploy` (on init, on edit). It is read on the server by `bonesremote`.

### Path ownership

- All product-owned paths are defined in `crates/bonesdeploy-core/src/paths.rs`.
- Other modules may derive subpaths by joining these constants but must not introduce independent path roots.
- The Python layer maintains its own `DeploymentPaths` class that mirrors the Rust constants.

### Permission model

- Provisioning-time contract: ownership layout is established once during `remote setup` and never rewritten by deploy commands.
- Three identity classes: `git` (deploy user, bare repo), `<site>` (runtime user, shared files, `/run/<site>`), `root` (sealed releases, system units, config dirs).
- Build scripts run in Podman as an unprivileged build user. Prepare scripts run as the runtime user. Only `bonesremote` (running as root) promotes, activates, and restarts services.

### State ownership

- `SiteState` (JSON) owns deployment metadata. It is the single source of truth.
- `DeploymentLock` serializes all mutations per site. Any command that mutates site state must go through `SiteMutation::acquire()`.
- Git-imported site state (via `site receive` or `site import`) is validated against a confused-deputy check (`project_name == site`) before replacement.

### External API encapsulation

- SSH connectivity is handled by `infra/ssh.rs` (Rust, for bonesdeploy ↔ bonesremote) and `pyinfra/runner.py` (Python, for bonesinfra provisioning).
- Git operations are wrapped in `infra/git.rs`.
- GPG operations are contained in `commands/secrets/gpg.rs` with an isolated keyring.

---

## 7. Architectural Conventions

### State persistence
- All state writes are atomic: temp file, fsync, rename, directory fsync. Never truncate a file in place.
- JSON is used for machine-readable state (`SiteState`, doctor reports, status, manifest). TOML is used for human-editable config.

### File naming
- Deployment scripts use `NN_name.sh` convention (`01_install_deps.sh`, `02_run_build.sh`). Scripts run in lexical order. Other files (README.md, etc.) are ignored.
- Release directories use the format `{timestamp}-{commit}-{suffix}` for uniqueness.

### Just-in-time mutation principle
- Pre-deploy steps (doctor, stage, checkout, wire) validate and prepare isolated state, not mutate live state.
- Build steps operate on isolated workspace state.
- Activation concerns happen at activation time.
- Permission hardening happens after successful activation, not before.
- If a deploy fails pre-activation, it leaves no live-state mutations.

### Framework convention
- Every framework has an importable Python package (`__init__.py`, `manifest.py`, `runtime.py`, `custom.py`, `templates/`) and a Rust question module.
- Framework-specific TOML overrides go through `Runtime.extra` (serde flatten).
- Project-local `.bones/infra/` takes strict precedence over built-in framework implementations. When the local package exists it is authoritative; the built-in is never consulted as fallback.

### Binary communication
- `bonesdeploy` communicates with `bonesremote` via SSH command execution (not an API).
- `bonesdeploy` communicates with `bonesinfra` via subprocess calls (`bonesinfra::run()` / `run_with_stdin()`).
- `bonesremote` communicates with `bonesinfra` only indirectly — `bonesdeploy` runs `bonesinfra` against the server during provisioning; `bonesremote` never calls `bonesinfra`.

### Error handling
- All functions return `anyhow::Result<T>`. Errors are chained with context.
- CLI entry points print errors as formatted chains to stderr and exit with non-zero code.
- Aborted deployments record the error in `DeploymentRecord.error` for diagnostic surfacing.

### Test patterns
- **bonesdeploy tests:** Isolated git repos + HOME override via `TestEnv` in `tests/common.rs`. Subprocess-based CLI tests.
- **bonesremote unit tests:** `set_sites_root_for_tests()` sets a thread-local override for state paths. Temp directories for filesystem operations. Pure function tests for predicates.
- **bonesremote integration tests:** Skeletal (CLI argument parsing only).
- **bonesinfra:** Embedded Python tests run via `cargo test` through `tests/pytest.rs`.
- End-to-end tests run in LXC containers and are NOT expected as part of routine development.

---

## 8. Known Architectural Tensions

### Duplicate config/path representations
`bonesdeploy-core` defines paths in Rust while `bonesinfra` defines them in Python (`DeploymentPaths`). These must be kept in sync manually. There is no automated consistency check between the two representations.

### Two provisioning layers
`bonesinfra` (Python/pyinfra) and `bonesremote` (Rust) both perform server-side operations, but they do so in separate domains: provisioning time vs deploy time. `bonesremote` never calls `bonesinfra`. The split is intentional (different lifecycle phases, different privilege contexts) but means some concerns (like user creation paths) appear in both systems.

### Legacy state migration
Older versions stored deployment state in separate files (`active-deployment.json`, `staged-release`). The `store` module migrates these to the unified `SiteState` format on first read. The migration path is one-way and the old files are deleted after migration.

### `CONTEXT.md` references `crates/shared/` but the actual crate is named `bonesdeploy-core`
The `CONTEXT.md` document uses the historical name `shared` for what is now the `bonesdeploy-core` crate. The path centralization rule ("all product-owned paths must live in `crates/shared/src/paths.rs`") refers to `crates/bonesdeploy-core/src/paths.rs`.

### Limited bonesremote integration test coverage
The `bonesremote` integration test suite (`tests/cli.rs`) only validates CLI argument parsing. Most behavioral guarantees are validated through unit tests (state persistence, phase transitions, idle checks, symlink atomicity). There are no end-to-end deployment pipeline tests in the Rust test suite — those live in `e2e/`.

### Thread-local test overrides
`bonesremote` uses a `thread_local!` with `RefCell` for `SITES_ROOT_OVERRIDE` rather than dependency injection. This makes test setup simple but means tests within the same binary must be serial if they use the override. In practice, Rust tests do run in separate threads by default, so this works but is a known constraint.

---

## 9. Reconnaissance Guide for Future Agents

When implementing a change in this repository:

1. **Identify the responsibility** involved in the requested change (config schema? provisioning? deployment lifecycle? CLI UX?).

2. **Find the existing concept** that currently owns that responsibility — use this document's §3 (Core Concepts) and §5 (Extension Points) as a map.

3. **Search for sibling implementations** and callers of the concept you're about to modify. For example, before adding a new framework, study an existing one in both the Rust (`crates/bonesdeploy/src/frameworks/laravel.rs`) and Python (`crates/bonesinfra/python/src/bonesinfra/frameworks/laravel/`) layers.

4. **Prefer extending an existing concept** over creating a parallel abstraction. The framework pattern, patch system, service ABCs, and doctor check modules are all explicit extension points.

5. **Use an existing implementation as the structural example** — copy the file pattern, function signatures, registration mechanism, and test approach.

6. **Create a new architectural concept only when** no existing concept reasonably owns the behavior. Before doing so, verify against this document that you're not reinventing a disguised version of an existing abstraction.

7. **Check both sides of the Rust/Python boundary** — a change to paths in `bonesdeploy-core` may need a corresponding update in Python's `DeploymentPaths`. A new config field in `Runtime` may need to flow into Python's `DeployContext.template_data()`.

8. **Respect the dependency direction** — `bonesdeploy-core` is a leaf. `bonesremote` does not depend on `bonesinfra`. `bonesinfra` only uses `bonesdeploy-core` for cache path resolution.

9. **Follow the just-in-time mutation principle** — if a change touches the deployment lifecycle, keep pre-activation mutations isolated from live state.