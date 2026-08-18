# Architecture

A more exhaustive inventory of every type, module, and flow lives in
[`docs/architecture/reference.md`](docs/architecture/reference.md). This document
is intentionally smaller: it exists to help a contributor decide *where new
behavior belongs* and *which existing concepts to snap into*.

## 1. Mental Model

Four major pieces. Each owns a distinct responsibility and does not bleed into
the others.

```
bonesdeploy-core            bonesdeploy
  Shared language             Local orchestration
  Config, paths, validation   User commands, init, SSH/Git/GPG
        ▲                          │
        │                          ▼
        │                      bonesinfra
        │                        Machine provisioning
        │                        Frameworks, runtimes, services
        │                        system configuration (pyinfra)
        │
        ▲
        │
     bonesremote
       Deployment execution
       Release lifecycle, locking, state, activation
```

**`bonesdeploy-core`** — the shared vocabulary of the system. Defines the
canonical configuration schema (`Bones`), every product-owned filesystem path,
and the validation rules for project/site names and services. A leaf crate with
no workspace dependencies. Both binaries and the Python provisioning layer
depend on it.

**`bonesdeploy`** — the local CLI binary the developer runs. Owns interactive
initialization, root `.env` authoring, SSH/Git/GPG integration, and delegation
to the other two pieces. It does not execute deployments itself; it either
provisions via `bonesinfra` or triggers `bonesremote` over SSH.

**`bonesinfra`** — an embedded Python provisioning runtime. Compiles a Python
package into the `bonesdeploy` binary via `rust-embed`, materializes it on demand
into a venv, and uses `pyinfra` to provision the remote server (users, packages,
frameworks, databases, SSL, firewalls). Owns *what gets installed* at
provisioning time.

**`bonesremote`** — the server-side binary that runs as root on the deployment
host. Owns the release lifecycle: staging, building, promoting, sealing,
activating, rolling back, pruning. It is the sole mutator of per-site deployment
state. It never calls `bonesinfra`.

## 2. Responsibility / Ownership Map

This is the heart of the document. If a responsibility is listed here, the named
owner is canonical. Bypassing it creates a competing abstraction.

| Responsibility | Owner | Reuse / extend | Do not |
| --- | --- | --- | --- |
| Deployment configuration | `Bones` (`bonesdeploy-core`), loaded from root `.env` | Extend the canonical config model via `Runtime.extra` | Invent parallel config structs in either binary or Python |
| Product filesystem layout | `paths` / `DeploymentPaths` | Add path constants here | Scatter path literals in commands, templates, or scripts |
| Framework-specific config questions | Framework Rust module (`frameworks/<fw>.rs`) | Add sibling module + register in `frameworks.rs` | Inline prompt logic in init command |
| Framework provisioning | Materialized `infra/provision/core` + `infra/provision/custom` packages, loaded by `bonesinfra.project` | Extend managed core or project-owned custom content | Special-case framework behavior in setup/runtime commands |
| Language runtime installation | `LanguageRuntime` ABC | Add subclass in `services/languages/` | Install runtimes directly from framework `runtime.py` |
| Database / server service provisioning | `RuntimeService` ABC + `SERVICE` registry | Add subclass + register in `SERVICE` dict | Put DB provisioning in framework code |
| Remote site mutation | `SiteMutation` | Acquire it before any site state change | Create independent locking or config-validation paths |
| Deployment lifecycle stages | Lifecycle modules (`release/lifecycle/`) | Add behavior to existing stage | Create a separate deployment flow |
| Per-site persisted state | `SiteState` + `state/` store | Read/write through the store API | Touch state files directly |
| Infrastructure migrations | Python `Patch` registry (`patches/registry.py`) | Add a registered, version-gated patch | Scatter one-off version checks throughout code |
| SSH connectivity | `infra/ssh.rs` (Rust), `pyinfra/runner.py` (Python) | Use existing session helpers | Open raw SSH channels |
| Git operations | `infra/git.rs` | Use existing git wrappers | Shell out to git ad hoc |
| GPG / secrets | `commands/secrets/gpg.rs` | Use the isolated keyring + helpers | Import GPG state from elsewhere |
| Doctor / health checks | `commands/doctor/` (both binaries) | Add a check module under the doctor tree | Probe system state from deploy/init commands |
| Embedded static assets | `rust-embed` asset modules | Add to the appropriate asset collection | Check in loose files that the binary must read at runtime |

## 3. Where Does New Behavior Belong?

Before adding code, classify the responsibility.

```
Need another programming language?
  → LanguageRuntime

Need another database / server service?
  → RuntimeService

Need another supported application framework?
  → Framework contract (Rust questions + Python package in bonesinfra/frameworks/<fw>/)

Need another filesystem location owned by BonesDeploy?
  → paths / DeploymentPaths

Need to mutate remote deployment state?
  → Existing bonesremote command + SiteMutation

Need another deployment stage?
  → Existing deployment lifecycle (add/modify a phase in release/lifecycle/)

Need version-dependent infrastructure migration?
  → Patch

Need functionality that talks to Git?
  → infra/git.rs

Need functionality that talks over SSH?
  → infra/ssh.rs (local CLI) or pyinfra/runner.py (provisioning)

Need a new CLI command?
  → Add a variant to the clap Command enum + a commands/<name>.rs handler

Need a new config field?
  → bonesdeploy-core config struct (global) or Runtime.extra (framework-specific)
```

If none of these fit:

1. Search for sibling behavior — something similar almost certainly exists.
2. Determine which subsystem owns the responsibility.
3. Only then consider introducing a new architectural concept.

## 4. Reusable Concepts

Only concepts an agent should actively reuse appear here. Implementation details
that simply exist (e.g. `DeploymentRecord` fields, embedded asset hashing) live
in the reference document.

### Configuration

```text
### Bones

Represents: The canonical deployment configuration for a project.

Use when:
Reading, writing, or validating project deployment config.

Contract:
Bones (bonesdeploy-core/src/config.rs)
├── app: App            # project_name, host, port, branch, domain, ssl, repo_path
├── runtime: Runtime    # template, web_root, backend, node_version, shared, extra
├── services: Services  # services: Vec<String>
└── build: Build        # timeout_seconds

Runtime.extra is a serde-flattened BTreeMap for framework-specific keys.

Existing implementations:
- Loaded from the project root `.env` via config::load()
- Loaded remotely from the site root `.env` via config::load_runtime()

To add another:
Add a field to the struct (global) or use Runtime.extra (framework-specific).

Canonical example:
crates/bonesdeploy-core/src/config.rs

Do not:
- Invent a parallel config struct in bonesdeploy, bonesremote, or bonesinfra
- Parse `.env` with ad-hoc readers bypassing config::load
- Duplicate the config schema in Python (use DeployContext, which is built from the same `.env`)
```

```text
### DeployContext

Represents: The Python-side projection of Bones, passed to all provisioning code.

Use when:
Writing Python framework/service/patch code that needs config.

Contract:
DeployContext (bonesinfra/python/.../config/context.py)
├── app: AppConfig
├── runtime: RuntimeConfig   # includes `data` dict for framework-specific keys
└── services: ServicesConfig

template_data(ctx) flattens it for Jinja2 rendering.

Existing implementations:
- Built from the root `.env` by DeployContext.from_files()

To add another:
Extend AppConfig / RuntimeConfig dataclasses. Framework-specific values go in RuntimeConfig.data.

Canonical example:
crates/bonesinfra/python/src/bonesinfra/config/context.py

Do not:
- Re-parse `.env` inside framework or service code
- Construct DeployContext manually outside the CLI entry points
```

```text
### DeploymentPaths

Represents: Canonical server-side filesystem layout, derived from project name and roots.

Use when:
Referencing any server-side path in Python templates, framework code, or services.

Contract:
DeploymentPaths (bonesinfra/python/.../config/paths.py)
- 41 frozen fields covering git repos, project dirs, config dirs, sockets, and logs
- Helpers include systemd_service(name), systemd_service_requirement(name),
  apparmor_profile(name), runtime_service_socket(name), and runtime_service_dir(name)

Existing implementations:
- Single frozen dataclass, computed from project_name + repo_path + root + web_root

To add another:
Add a field here. The Rust-side constants in bonesdeploy-core/src/paths.rs must agree.

Canonical example:
crates/bonesinfra/python/src/bonesinfra/config/paths.py

Do not:
- Hardcode path strings in framework templates or service code
- Introduce a second path-resolution mechanism
- Let the Rust and Python path representations drift out of sync
```

### Provisioning

```text
### Framework Contract

Represents: Support for a specific web application framework (Django, Laravel, Next, ...).

Use when:
Adding or modifying support for an application framework.

Contract:
Two halves that must stay in sync.

Rust side (crates/bonesdeploy/src/frameworks/<fw>.rs), selected through
`frameworks.rs`:
├── questions() -> &'static [Question]
├── centralized validate_answers(answers) -> Result<()>
├── configure(cfg: &mut Bones)              # defaults/configuration
├── environment_example(...) -> String
├── build_environment_example(...) -> String
└── defaults() -> FrameworkDefaults        # web root, language, permissions

Python side (materialized under `infra/provision/`):
├── core/   → managed framework package (`manifest.py`, `runtime.py`, templates/)
└── custom/ → project-owned package composed after core
    (`manifest.py`, `runtime.py`)

BonesDeploy owns framework selection, centralized validation, defaults,
permission defaults, environment generation, and deployment assets. BonesInfra
owns the managed framework source and materializes it into
`infra/provision/core/`. Project-owned extensions live in
`infra/provision/custom/` and are composed after the core package.

The materialized core/custom packages are used together. Managed core content is
updated by infrastructure synchronization; custom content is preserved and runs
as the project-owned extension.

Existing implementations:
- custom, django, laravel, next, nuxt, rails, sveltekit, vue

To add another:
1. Add Rust module under src/frameworks/<name>.rs; register in frameworks.rs.
2. Add the built-in Python package under bonesinfra/frameworks/<name>/ with
   `__init__.py`, `manifest.py`, `runtime.py`, and templates.
3. Add deployment assets and `.env.build` defaults under the embedded framework assets.
4. Add the name to `BUILTIN_FRAMEWORKS` in `project.py` and preserve
   materialization into `infra/provision/core` plus `infra/provision/custom`.

Canonical example:
frameworks/laravel/ (both Rust and Python sides)

Do not:
- Special-case framework behavior in setup, runtime, or init commands
- Bypass the Rust framework dispatch in frameworks.rs
- Put framework provisioning logic anywhere other than the framework's runtime.py
- Treat project custom content as managed core content
```

```text
### LanguageRuntime

Represents: A programming language runtime that can be installed on the deployment server.

Use when:
Adding support for a new language (e.g. Go, Elixir).

Contract:
LanguageRuntime ABC (services/languages/base.py)
├── config_key          # .env key for version selection
├── default_version
├── version_pattern
├── install(ctx)
└── install_version(ctx) -> str

Framework runtime.py modules select and invoke the appropriate language runtime.

Existing implementations:
- PHPRuntime, PythonRuntime, NodeRuntime, RubyRuntime

To add another:
Subclass LanguageRuntime in services/languages/<name>.py; export a singleton.

Canonical example:
services/languages/php.py

Do not:
- Install language runtimes directly from framework runtime.py code
- Create a second language-installation mechanism
- Reimplement version selection logic per framework
```

```text
### RuntimeService

Represents: A server-side service BonesDeploy can provision (database, cache, ...).

Use when:
Adding PostgreSQL-, Redis-, MongoDB-like infrastructure.

Contract:
RuntimeService ABC (services/runtime/base.py)
├── provision(ctx)                # install, create user/db, seed shared/.env
├── manifest_artifacts(ctx)       # declare paths for manifest inspection
└── manifest_services(ctx)        # declare systemd units for manifest inspection

Registered in the SERVICES dict in services/runtime/__init__.py.
Activated by the service names parsed from the root `.env`.

Existing implementations:
- PostgresService, RedisService, MariaDBService, MysqlService, MongodbService, ValkeyService

To add another:
Subclass RuntimeService in services/runtime/<name>.py; add entry to SERVICES dict.

Canonical example:
services/runtime/postgres.py

Do not:
- Provision databases directly from framework runtime.py
- Create another service registry
- Reimplement shared provisioning behavior (user creation, credential generation)
```

```text
### Patch

Represents: A version-gated infrastructure migration step.

Use when:
Adding a migration that must run during `bonesdeploy update` for a specific version.

Contract:
Patch (patches/registry.py)
├── identifier            # e.g. "0003-project-infra"
├── introduced_in         # semver
└── local_apply           # runs on the workstation

The current registry contains only `0003-project-infra`, introduced in `0.8.0`.
Its remote scope writes the remote completion marker; it does not perform a
second remote migration.

Completion is tracked per-project, per-scope via marker files.
Local markers: ~/.local/share/bonesdeploy/patches/<project>/<id>
Remote markers: /var/lib/bonesdeploy/patches/<site>/<id>

Existing implementations:
- 0003-project-infra (local project layout migration; remote marker handling)

To add another:
Add a Patch to registry.py with an introduced_in version; implement apply functions.

Canonical example:
patches/registry.py

Do not:
- Scatter one-off version checks or migration scripts elsewhere
- Run migrations outside the patch registry
- Skip the marker-file idempotency mechanism
```

### Deployment

```text
### SiteMutation

Represents: The serialization guard for all remote site-mutating operations.

Use when:
Any bonesremote command that changes per-site state.

Contract:
SiteMutation (crates/bonesremote/src/release/site_mutation.rs)
├── site: String          # site identity
├── config: Bones         # validated snapshot (confused-deputy check: project_name == site)
└── _lock: DeploymentLock # per-site advisory file lock (flock)

Constructors:
- acquire(site)                  # standard: lock, then load config
- adopt(site, cfg, lock)         # cancellation: adopt config loaded before terminating a deployment

Existing implementations:
- Single implementation; consumed by deploy, rollback, kill, drop-failed, prune, and service restart

To add another:
Do not. Use the existing constructor that matches your context.

Canonical example:
crates/bonesremote/src/release/site_mutation.rs

Do not:
- Create independent locking mechanisms
- Mutate site state without acquiring a SiteMutation
- Validate config separately from lock acquisition (the guard bundles them deliberately)
```

```text
### Deployment Lifecycle

Represents: The staged, persisted state machine for a single deployment.

Use when:
Modifying what happens during a deploy, or adding a deployment stage.

Contract:
Phases (release/state/record.rs):
Created → SourceExported → Built → Promoted → Prepared → Sealed
       → Activated → Verified → Completed
       → (CleanupPending on post-commit failure | Failed on pre-commit abort)

Orchestrator (commands/deploy/lifecycle.rs):
run_staged_deployment(mutation, revision)
  ├─ stage::run()         → Created
  ├─ checkout::run()      → SourceExported
  ├─ build::run()         → Built
  ├─ build::promote()     → Promoted
  ├─ wire_shared + prepare → Prepared
  ├─ build::finalize()    → Sealed
  ├─ preflight::validate  (nginx -t gate; no live mutation yet)
  ├─ activate::run()      → Activated   *** cut-over ***
  ├─ service::run()       → Verified
  └─ prune + cleanup      → Completed

Pre-activation failure: abort, drop failed release, clear state.
Post-activation failure: restore previous release, restart, clear state.
Post-commit maintenance failure: record as CleanupPending (non-blocking).

Each phase is persisted to SiteState. Crash detection via pid + process_start_ticks.

Existing implementations:
- Single orchestrator; phases are not independently runnable commands in normal use.

To add another:
Add/modify a stage module under release/lifecycle/<stage>.rs; thread the phase
transition through the orchestrator. Do not create a parallel deployment flow.

Canonical example:
release/lifecycle/build/mod.rs (a stage with sub-steps)

Do not:
- Create a second deployment pipeline
- Mutate live state before the Activated phase (just-in-time principle)
- Skip persisting phase transitions to SiteState
- Bypass the preflight gate before activation
```

```text
### SiteState

Represents: The single source of truth for a site's runtime-mutated state.

Use when:
Reading or writing deployment state (active deployment, staged release).

Contract:
SiteState (release/state/store.rs)
├── schema_version: u32
├── active: Option<DeploymentRecord>
└── staged_release: Option<String>

All writes are atomic (temp file, fsync, rename, directory fsync).
Malformed state triggers quarantine via `release recover`.
Legacy files (active-deployment.json, staged-release) are migrated on first read.

Existing implementations:
- Single store per site at `<sites_root>/<site>/deployment.json`

To add another:
Do not. Extend the SiteState struct if new state is needed.

Canonical example:
crates/bonesremote/src/release/state/store.rs

Do not:
- Read or write state files directly
- Introduce a second state file for a site
- Use non-atomic writes for state
```

### Integration Boundaries

These are thin wrappers around external systems. New code that needs to talk to
these systems should use the wrapper, not open a new channel.

```text
### SSH (Rust)
infra/ssh.rs — connect, connect_privileged, connect_as, run_cmd, stream_cmd
  Used by: bonesdeploy commands that invoke bonesremote on the server
  Do not: open raw SSH sessions outside this module

### SSH (Python)
pyinfra/runner.py — connect_all, run_ops; operations.py — mkdir, render
  Used by: bonesinfra provisioning commands
  Do not: use a different SSH library or bypass the pyinfra runner

### Git
infra/git.rs — ensure_git_repository, remotes, URL parsing
  Used by: init, deploy, doctor
  Do not: shell out to git directly

### GPG
commands/secrets/gpg.rs — isolated keyring, key generation, encrypt/decrypt
  Used by: secrets init/edit/push
  Do not: import external GPG state or use a different keyring location
```

## 5. Architectural Conventions

**Atomic state writes.** All persisted state uses temp file → fsync → rename →
directory fsync. Never truncate in place.

**Path centralization.** All product-owned paths live in `bonesdeploy-core::paths`
(Rust) and `DeploymentPaths` (Python). Other code derives subpaths by joining
these constants. No hardcoded path strings elsewhere.

**Just-in-time mutation.** Pre-deploy stages validate and prepare isolated
candidate state. Activation is the live release cut-over; shared links and
candidate preparation must not be described as mutations of the active release.

**Framework contract.** Rust owns framework questions, centralized validation,
defaults, permission defaults, and build-environment generation. Python
provisioning uses managed `infra/provision/core` and project-owned
`infra/provision/custom` packages composed together. Framework-specific config
goes through `Runtime.extra`.

**Binary communication.** `bonesdeploy` ↔ `bonesremote` via SSH command execution.
`bonesdeploy` ↔ `bonesinfra` via the `bonesinfra::run()` subprocess boundary;
BonesInfra reads the explicit root `.env` path.
`bonesremote` never calls `bonesinfra`.

**Script naming.** Deployment scripts use `NN_name.sh` and run in lexical order.
Other files in the script directories are ignored.

**Error handling.** All functions return `anyhow::Result<T>`. CLI entry points
print formatted error chains to stderr. Aborted deployments record the error in
the deployment record.

## 6. Dependency and Ownership Rules

- **`bonesdeploy-core`** is a leaf crate. Nothing in the workspace depends
  outward through it.
- **`bonesdeploy`** depends on `bonesdeploy-core` and `bonesinfra`.
- **`bonesremote`** depends on `bonesdeploy-core`. It does **not** depend on
  `bonesinfra`.
- **`bonesinfra`** depends on `bonesdeploy-core` (minimally, for cache path
  resolution).
- Config is authored by `bonesdeploy` in the root `.env`, then read by both
  binaries and the Python layer. The schema is owned by `bonesdeploy-core`.
- Path constants are owned by `bonesdeploy-core::paths` (Rust) and mirrored by
  `DeploymentPaths` (Python). The two must agree.
- Remote site state is owned exclusively by `bonesremote`. `bonesdeploy` never
  reads or writes it directly.
- Provisioning is owned by `bonesinfra`. Deployment execution is owned by
  `bonesremote`. The two never call each other.

## 7. Known Architectural Tensions

**Duplicate path representations.** `bonesdeploy-core::paths` (Rust) and
`DeploymentPaths` (Python) must be kept in sync manually. There is no automated
consistency check.

**Two server-side layers.** `bonesinfra` (provisioning) and `bonesremote`
(deployment) both operate on the server but in separate lifecycle phases and
privilege contexts. Some concerns (user creation, directory layout) appear in
both systems by design.

**Legacy state migration.** Older versions stored deployment state in separate
files. The `store` module migrates these to unified `SiteState` on first read
and deletes the old files. The migration is one-way.

**Cross-layer configuration and integration side doors.** Rust and Python have
separate readers for the same root `.env` contract. BonesInfra provisioning
receives the explicit `--env-file` path, while Git and SSH callers use their
existing integration boundaries.

**Framework and deployment side doors.** Framework identity, defaults, and
assets cross Rust and Python boundaries, while release commands can reach state
and mutation details directly. These are tracked for the Framework and
Deployment child changes.

**Thread-local test overrides.** `bonesremote` uses a `thread_local!` for
`SITES_ROOT_OVERRIDE` in tests rather than dependency injection. Tests using the
override must effectively be serial.

## 8. Reconnaissance Guide for Future Agents

1. Identify the responsibility involved in the requested change.
2. Consult §2 (Ownership Map) to find the canonical owner.
3. Consult §3 (Where Does New Behavior Belong?) to find the concept to snap into.
4. Search for sibling implementations and study one as a structural example.
5. Prefer extending an existing concept over creating a parallel abstraction.
6. Check both sides of the Rust/Python boundary — a change to paths or config
   may need corresponding updates in both `bonesdeploy-core` and `bonesinfra`.
7. Respect the dependency direction: core is a leaf; `bonesremote` does not
   depend on `bonesinfra`.
8. Follow the just-in-time mutation principle if the change touches the
   deployment lifecycle.
9. Create a new architectural concept only when no existing concept reasonably
   owns the behavior, and only after searching for sibling behavior that might
   already cover it.

For exhaustive type inventories, function signatures, and traced flows, see
[`docs/architecture/reference.md`](docs/architecture/reference.md).
