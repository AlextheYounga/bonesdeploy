# Project Owned Infrastructure

<!-- Merged from feature/decentralization -->

## Project-Local Infrastructure Refactor

### Request

Move project-owned deployment infrastructure into the application repository
under `infra/`, remove the separate `.bones` configuration repository and its
transport, and make an application Git revision the unit of deployment.

Remove the general-purpose `bones.toml` configuration model rather than
relocating it. Keep only information that has a real consumer in an existing
channel: convention, local `.env`, committed `.env.build`, committed
`infra/` files or infrastructure code, or existing remote machine state.

Deployments remain explicitly user-triggered:

```text
git push
bonesdeploy deploy
```

### Problem

The current project configuration is a symlink from `.bones` to a separate
Git repository under the user's configuration directory. `bonesdeploy push`
publishes that repository to a second remote repository, where BonesRemote
imports it as mutable site state. Local commands, provisioning, and remote
release stages then depend on that separately synchronized copy.

This duplicates Git history, separates application code from the instructions
that build and deploy it, and allows a deployment to combine an application
revision with infrastructure from another revision. It also treats storage
location as a secret boundary even though the encrypted secret mechanism is
the actual protection required.

### Definitions

**Project identity** is the explicit site name supplied to BonesDeploy and
BonesRemote, such as `checkmyslop`. It is used to derive the canonical bare
repository, site root, runtime identity, build identity, and systemd target. It
is not inferred from an incidental directory name and is not duplicated in a
remote project configuration file.

**Project infrastructure** is committed, non-machine-specific behavior and
assets under `infra/`, including deployment scripts, templates, and project
infrastructure Python modules. It is part of normal application Git history.

**Managed provisioning** is the project-facing provisioning implementation
under `infra/.framework/`. BonesDeploy supplies and maintains it, and an
explicit BonesDeploy update may refresh it. It is visible and committed in the
application repository, but users should customize `custom/` instead.

**Custom provisioning** is project-owned provisioning code under
`infra/custom/`. BonesDeploy updates preserve it and never silently
overwrite it. Core provisioning runs before custom provisioning through
ordinary explicit Python composition; no general plugin registry is required.

**Provisioning input** is information consumed only by the local
`bonesdeploy` process while it invokes BonesInfra to change a remote machine.
Connection, domain, TLS, and similar values are local `.env` inputs and are
not required by remote deployment.

**Deployment revision** is the full commit SHA resolved by BonesRemote from
the requested branch or revision in the application bare repository. The
revision supplies the application source, `infra/` deployment instructions,
and `.env.build` values used for that deployment.

**Machine state** is durable state created or maintained on the server:
repositories, releases, shared runtime data, locks, deployment records, logs,
caches, identities, service configuration, and runtime secrets. It is not a
second copy of project infrastructure.

**Encrypted secret material** is the existing GPG-encrypted project secret
file. It may be committed under `infra/secrets/`; plaintext runtime secrets
remain protected server-side in shared state and are never supplied to builds.

**Local key material** is the machine-local GPG keyring, including private
keys, trust data, and other decryption authority. It is never stored in the
application repository, `infra/`, or remote Git repositories. The refactor
preserves the BonesDeploy local data/configuration roots needed to store this
keyring and other non-project application state.

**Managed shared state** is the server-side `shared/` directory used by
releases. BonesDeploy always creates and wires `shared/.env`; it may create
framework-declared directories referenced by local environment values. It does
not create application data files such as SQLite databases.

### Desired outcome

A project contains ordinary committed infrastructure content:

```text
project/
├── infra/
│   ├── provision/
│   │   ├── core/
│   │   └── custom/
│   ├── deployment/
│   ├── templates/
│   ├── secrets/
│   └── infrastructure code and assets
├── .env
├── .env.build
└── application source
```

There is no `.bones` symlink, nested infrastructure Git repository,
per-project configuration copy under `~/.config/bonesdeploy/projects`,
infrastructure remote repository, infrastructure-specific Git hook, or separate
`bonesdeploy push` step.

For `bonesdeploy deploy`, BonesRemote resolves one immutable deployment
revision, reads the deployment instructions and scalar build inputs from that
revision, and runs the existing staged release lifecycle. It does not reload
project configuration from mutable remote site state.

The build context contains only source and build-safe inputs. It excludes
`infra/secrets/`, plaintext runtime secrets, decryption keys, `shared/.env`,
and other runtime credentials. The existing GPG workflow remains in place;
this refactor changes its project path only where required by `infra/`.

### Scope

- Remove the `.bones` local and remote configuration repositories and their
  synchronization commands, hooks, and site import/receive transport.
- Audit every current `Bones` and `bones.toml` consumer and eliminate or move
  each value to convention, local `.env`, `.env.build`, committed `infra/`, or
  machine state.
- Make `init`, setup, doctor, manifest, runtime provisioning, services,
  SSL, secrets, deploy, rollback, status, and update use the new boundaries.
  - Make project-facing provisioning visible in `infra/.framework/` and
  `infra/custom/`, with explicit ownership and update behavior.
- Make BonesRemote obtain deployment-specific instructions from the requested
  application Git revision while preserving the existing release safety,
  locking, cancellation, and activation behavior.
- Remove deploy-on-push behavior and its application post-receive trigger.
- Provide a safe migration from an existing `.bones` tree to `infra/` without
  creating commits or exposing plaintext secrets.
- Keep project-local infrastructure code and assets in `infra/` and retain
  the existing BonesInfra framework behavior behind that boundary.

### Constraints

- The application Git revision is the only project revision deployed.
- Remote deployment may use only the target revision, existing machine state,
  and explicit command arguments after local control ends.
- `.env.build` is the single committed scalar channel for values needed after
  control moves to the remote machine; `BONES_*` names remain reserved for
  derived/internal values.
- Local provisioning values remain uncommitted `.env` inputs.
- Runtime secrets never enter the build environment, and plaintext secrets are
  never committed.
- `shared/.env` is the only shared file created and release-wired by
  BonesDeploy. Other application files are created by the application.
- GPG private keys and other decryption authority remain machine-local. Do not
  delete or relocate the local BonesDeploy data/configuration roots solely
  because the old per-project configuration repository is removed.
- Project identity remains explicit and is validated at trust boundaries.
- `infra/.framework/` is BonesDeploy-managed and updateable. An update must
  detect modified managed files and refuse to overwrite them silently.
- `infra/custom/` is project-owned and preserved by every
  BonesDeploy update.
- Existing rollback behavior remains conceptually unchanged and gains no new
  metadata unless an existing feature demonstrably requires it.
- Preserve current runtime permission behavior; `[runtime.permissions]` is a
  separate follow-up.
- Do not run e2e tests during ordinary implementation validation.
- No backwards compatibility with old `.bones`-based project layouts. Commands
  fail with a clear message if they encounter the old structure; `bonesdeploy
  update` is the single versioned bridge.
- BonesDeploy creates only framework-declared shared directories from local
  environment values. It does not infer or create arbitrary shared files;
  application data files remain application-owned.

### Exclusions

- Redesigning GPG encryption or introducing git-crypt, SOPS, age, or another
  secret system.
- Leaving the BonesInfra execution engine or its authoritative source outside
  the project's managed core.
- Introducing a general plugin framework or broad ownership subdivisions where
  no managed/project-owned boundary exists.
- Automatically pushing application changes from `bonesdeploy deploy`.
- Redesigning deploy-on-push; it is removed for now.
- Adding a replacement `deploy.toml`, `infra.toml`, or equivalent declarative
  project configuration layer.
- Stripping `infra/` from final runtime release artifacts; that is a later
  hardening change.
- Reworking unrelated deployment behavior or making runtime permissions
  functional as part of this refactor.
- Inferring arbitrary shared files from path-like environment values or adding
  a generic shared-path permission model.


## Project-Local Infrastructure Refactor Plan

### Current behavior

The shared `bonesdeploy-core` crate models the project as `config::Bones`,
loaded from `.bones/bones.toml`. `App`, `Runtime`, `Services`, and `Build`
contain connection details, project identity, deployment branch, DNS/TLS,
runtime selection, shared paths, services, release retention, and build
limits. `bonesdeploy` commands load that file directly for init, doctor,
setup, manifest, remote provisioning, secrets, deploy, rollback, status,
config, and update.

`init` creates a configuration directory under the user's XDG config root,
scaffolds it, exposes it through `.bones`, initializes a nested Git repository,
and materializes project infrastructure below `.bones/infra`. `push_state`
autocommits and pushes that nested repository to the root-owned remote config
repository. `setup` and `deploy` invoke that synchronization before remote
operations. The local GPG keyring is separate machine-local state. The
encrypted project secret file currently lives under `.bones/secrets`; the
private keys are not in that directory or in the project repository.

On the server, site import/receive validates and replaces a persisted site
dataset containing `bones.toml` and deployment files. `SiteMutation` loads
that dataset and couples it to the site lock. Remote release stages repeatedly
use site-derived identity and paths plus a small `RemoteDeploymentConfig`
received over SSH stdin; checkout resolves a revision and exports the complete
bare-repository tree, while build and prepare scripts use the in-memory
descriptor. Runtime settings are never reloaded from `shared/.env`, which is
application runtime input only. The deployment record already stores the
resolved full revision SHA.

BonesInfra's `DeployContext.from_files` parses `bones.toml`, derives machine
paths and identities, and feeds setup, runtime, services, SSL, helpers, and
manifest operations. Its project infrastructure loader already recognizes an
`infra/` directory relative to the supplied configuration path and can load
project-local Python entrypoints. The current architecture does not yet
separate BonesDeploy-managed provisioning from project-owned provisioning or
provide an update path for managed project-local files.

`.env.build` is already parsed without shell evaluation, rejects duplicate,
invalid, and reserved `BONES_*` names, and is included in build environment
construction. The current build implementation additionally derives scalar
environment variables from persisted `Bones` configuration. Build containers
are not intended to receive runtime secrets or the control-plane state.

### Intended behavior

`bonesdeploy init` establishes project identity and local provisioning inputs
without creating `.bones`, a nested repository, or a remote config repository.
It scaffolds committed `infra/` content and `.env.build`, and updates the
application repository ignore rules so generated infrastructure and encrypted
secret paths are handled deliberately. Local `.env` supplies connection and
provisioning-only values. Existing machine-local BonesDeploy data, including
the GPG keyring under the data root, is preserved. Project-facing provisioning
is scaffolded under `infra/.framework/` and `infra/custom/`.
Core runs first and custom runs second through explicit composition.

Local provisioning commands load explicit project identity and `.env`, derive
canonical paths from conventions, and pass a typed provisioning context to
BonesInfra. Setup writes only machine state to the host. `bonesdeploy deploy`
does not publish local configuration, push application code, or automatically
push changes; it connects to the configured host and invokes BonesRemote for
the user-selected or configured application revision.

Setup creates the server-side `shared/` directory and `shared/.env` for every
framework. Deployment always wires `shared/.env` into the active release as
`.env`. Framework provisioning may create known directories whose locations are
provided by local environment values, but application data files such as
`database.sqlite` are left for the application to create.

BonesRemote resolves the requested branch or revision to a full commit SHA
before staging. A deployment-scoped specification contains the site identity,
bare repository, resolved SHA, and the revision's deployment inputs. Git
lookups for `infra/` scripts, infrastructure entrypoints, and deployment
scalar values use that SHA. Lifecycle stages receive the same specification
or its parsed components rather than reloading mutable site configuration.
Machine paths, users, services, locks, release state, and runtime secrets
continue to come from existing conventions and server state.

The source export/build boundary remains secret-safe. Build execution receives
application source, `.env.build`, and explicitly build-safe inputs, but not
runtime plaintext, decryption keys, or encrypted secret material that is not
needed by the build. Deployment scripts and prepare scripts are streamed or
materialized only through the controlled lifecycle. Existing release locking,
atomic state, cancellation, rollback, activation, and cleanup semantics stay
in force.

Deploy-on-push and both its local/remote trigger plumbing are deleted. A
legacy project migration copies the old project-owned files into `infra/`,
moves encrypted secret material without decrypting it, removes the old
symlink/repository only after the destination is verified, leaves machine-local
GPG keyrings untouched, and leaves Git commit creation to the user.

### Approach

1. Audit all `Bones` fields and consumers. Classify each consumer as
   `CONVENTION`, `LOCAL`, `PYINFRA`, `BUILD`, `BONESREMOTE`, `MACHINE STATE`,
   or `OBSOLETE`. Delete fields and APIs with no remaining consumer rather
   than mechanically turning every field into an environment variable.
2. Replace `.bones` path constants and local config loading with project-root
   paths for `infra/`, `.env`, `.env.build`, and encrypted secrets. Commands
   that encounter the old `.bones` layout must fail with a clear message
    directing the user to `bonesdeploy update`, not attempt auto-detection or
   dual-mode adaptation. Keep the explicit project identity in the local
   provisioning environment/command boundary, not in a replacement project
   TOML file. Preserve the machine-local data/configuration roots used for the
   GPG keyring and other non-project application state; remove only their
   obsolete per-project configuration repository contents.
3. Refactor init, setup, provisioning, doctor, manifest, secrets, status,
   rollback, update, and CLI help around those paths. Remove push-state,
   config-repository setup, config commands, and deploy-on-push hooks where
   their only purpose was the old transport.
4. Give BonesRemote one deployment-scoped revision/configuration boundary.
   Resolve the full SHA once, validate project identity, read revision-owned
   deployment files through Git, and pass the resulting values through the
   existing lifecycle. Keep machine-state loading limited to derived paths,
   runtime state, release state, and service state.
5. Update BonesInfra's context and project-infrastructure loading so setup
   consumes local `.env` plus committed `infra/` code/assets, while remote
   deployment does not require a persisted project configuration dataset.
6. Restore the versioned update-patch mechanism, add the `0.8.0` local layout
   transition, and remove obsolete transport paths, hooks, tests, templates,
   and documentation. `bonesdeploy update` is the single deliberate bridge
   from old to new layout; no other command detects, adapts to, or
    silently supports old `.bones`-based workspaces. Preserve encrypted secret
    bytes, leave local key material untouched, and fail closed when a migration
    encounters unsafe plaintext or ambiguous paths.
7. Separate project-facing provisioning into managed `infra/.framework/`
   and project-owned `infra/custom/`. Scaffold both, compose them
   explicitly in that order, and make update refresh only unmodified managed
   files. If a managed file was modified, update reports the conflict and
   refuses to overwrite it; it does not perform a three-way merge.
8. Remove the generic `runtime.shared` model. Always create and wire
   `shared/.env`; let framework modules identify directory-valued environment
   variables whose directories should be created. Do not create arbitrary
   files or infer file types from path-like values.

### Responsibilities and boundaries

- `bonesdeploy-core`: project identity validation, canonical machine-path
  derivation, `.env`/`.env.build` parsing, and small shared deployment value
  types. It must not own a persisted project TOML schema.
- `bonesdeploy` CLI commands: translate command arguments and local
  environment into provisioning/deployment requests; do not synchronize a
  second repository or contain remote release policy.
- `bonesinfra`: local provisioning operations and project-local infrastructure
  loading. It owns host setup inputs and templates, not remote release state.
  It loads managed core provisioning before project custom provisioning through
  explicit composition.
- Update flow: refreshes BonesDeploy-managed `infra/.framework/` files only
  when their project copies remain unmodified. It preserves
  `infra/custom/` and reports modified-framework conflicts without
  overwriting files.
- `bonesremote` revision boundary: resolve and validate the immutable Git
  revision, load revision-owned deployment behavior, and pass one snapshot
  through the deployment lifecycle.
- `bonesremote` release modules: retain stage, build, promote, prepare, seal,
  activate, rollback, lock, cancellation, and cleanup behavior. They consume
  the revision snapshot and machine state; they do not discover project policy
  from a mutable site config copy.
- Secrets module: preserve GPG encryption and protected remote shared-state
  delivery, with encrypted project data rooted in `infra/` locally. Keep the
  GPG keyring and private keys in the machine-local BonesDeploy data root; they
  must never pass through Git or build execution.
- Shared directory provisioning: create `shared/` and `shared/.env` for every
  framework, wire `.env` into releases, and create only framework-declared
  directories. Application-owned files are not created by BonesDeploy.
- Update patches: perform the explicit, reviewable filesystem transformation
  from old to new layout. The ordered Python registry owns version gates and
  per-project completion markers; only the `0.8.0` local patch copies the old
  layout. No ordinary command reads, adapts to, or silently supports `.bones`.

### Affected areas

- `crates/bonesdeploy-core/src/config.rs`, `app.rs`, `env_build.rs`, and
  `paths.rs`: remove the general-purpose TOML model and centralize new local
  input/path rules.
- `crates/bonesdeploy/src/commands/init/`, `config.rs`, `deploy.rs`,
  `setup.rs`, `push_state.rs`, `doctor.rs`, `manifest.rs`, `remote/`,
  `secrets/`, `rollback.rs`, `releases.rs`, `status.rs`, `update/`, and CLI
  argument/dispatch modules: adopt the new local and explicit-deploy flows.
- `crates/bonesremote/src/commands/site.rs`, `commands/hook.rs`,
  `release/lifecycle/`, `release/site_mutation.rs`, release state, and path
  consumers: remove imported site datasets and carry the revision snapshot.
- `crates/bonesinfra/python/src/bonesinfra/config/`, `project.py`, CLI
  commands, templates, and tests: consume local provisioning inputs and
  project-local `infra/` assets without a replacement TOML layer, including
  the managed-core/custom provisioning boundary.
- Embedded scaffolding, framework assets, shell hooks, tests, `CONTEXT.md`,
  `README.md`, and relevant skill documentation.
- Update-patch migration tests and focused Rust/Python tests at each changed
  public command or parser boundary.

### Decisions

- `infra/` is the sole project-owned infrastructure directory and is ordinary
  application Git content.
- Project-facing provisioning is split into visible managed
  `infra/.framework/` and project-owned `infra/custom/`.
  Managed core is updateable; custom provisioning is never silently replaced.
- `runtime.shared` is removed. The only universally managed shared file is
  `shared/.env`; other shared directories are framework-declared and
  application data files are application-owned.
- Core provisioning executes before custom provisioning using ordinary explicit
  Python composition. The design does not add a general plugin registry.
- No replacement general-purpose project configuration file is introduced.
  Each value must use convention, local `.env`, committed `.env.build`,
  committed infrastructure code/assets, or machine state.
- Project identity is explicit input and remains separate from incidental
  filesystem names and remote persisted configuration.
- The full Git SHA is resolved once per deployment and is the consistency
  boundary for application source and deployment behavior. Existing release
  metadata is reused; no new history database is added.
- Deployments stay explicit and never push application changes automatically.
- Deploy-on-push is removed instead of being adapted to the new architecture.
- Existing GPG encryption remains the secret mechanism. Encrypted bytes may
  travel with Git; build execution has no decryption authority.
- GPG private keys, keyrings, trust data, and other decryption authority remain
  machine-local. Removing the old per-project config repository does not mean
  deleting `~/.config/bonesdeploy`, `~/.local/share/bonesdeploy`, or their
  equivalent XDG roots.
- Runtime release contents are not stripped of `infra/` in this change.

### Risks

- A missed `Bones` consumer could retain a hidden dependency on removed site
  state or silently lose a required value. The field audit and compile/test
  failures are required gates.
- Mixing revision-owned files with machine state could make a release
  internally inconsistent. The revision specification must be created before
  staging and reused through every lifecycle stage.
- Exporting `infra/` or secret files into the build context could expose
  credentials. Build-context tests must assert exclusion of encrypted and
  plaintext runtime secret paths.
- Migration could overwrite user files or decrypt secrets accidentally. It
  must validate destinations, preserve encrypted bytes, leave machine-local
  keyrings untouched, refuse unsafe source layouts, and never auto-commit.
- Removing the old remote dataset can break rollback or doctor if those paths
  remain implicit. Those commands must use conventions and existing machine
  state only, with focused regression coverage.
- An update could destroy project changes in managed core or leave fixes
  unapplied. It must compare the project file with the previously supplied
  managed version, refresh only unmodified files, and report modified-file
  conflicts without overwriting them.

### Validation

- Unit-test `.env` and `.env.build` parsing, identity validation, path
  derivation, revision file lookup, and build-context filtering.
- Test CLI-visible init, migration, deploy argument handling, and removal of
  the old push/config transport through isolated temporary repositories and
  environments.
- Test BonesRemote deployment setup with a bare repository containing two
  revisions whose infrastructure differs, proving one deployment uses one
  revision consistently.
- Test that encrypted secret material and runtime plaintext are absent from
  build inputs and that secret delivery still writes protected shared state.
- Test that initialization creates the managed framework and custom directories,
  core executes before custom, updates preserve custom files, and modified
  managed files produce a conflict without being overwritten.
- Test that setup creates `shared/.env`, deployment wires it into releases, and
  framework provisioning creates declared directories without creating
  application data files.
- Run affected Rust and Python tests, `cargo fmt`, `cargo clippy`, `shfmt -w .`,
  and Python lint/format checks required by the crate instructions. Do not run
  e2e tests.
- Review the final diff for stale `.bones`, config-repository, import/receive,
  post-receive, and deploy-on-push references, then verify documentation
  describes only the new architecture.


## Project-Local Infrastructure Refactor Tasks

### Implementation

- [ ] Complete the `Bones` consumer audit and record each surviving value's
      destination as convention, local `.env`, `.env.build`, committed
      `infra/`, machine state, or obsolete.
- [ ] Replace shared `.bones` and `bones.toml` path/config APIs with explicit
      project identity, local `.env`, `.env.build`, `infra/`, and derived
      machine-path boundaries; remove fields classified obsolete.
- [x] Refactor `bonesdeploy init` and framework scaffolding to create ordinary
      committed `infra/` content without symlinks, nested Git, or config-root
      per-project copies, while preserving `.env.build`, framework asset
      behavior, and machine-local BonesDeploy data including the GPG keyring.
- [x] Scaffold project-facing provisioning under
      `infra/.framework/` and `infra/custom/`, mark the framework as
      BonesDeploy-managed, and compose core before custom through explicit
      Python execution.
- [x] Make all commands that depend on project loading fail clearly when they
       encounter an old `.bones` layout, directing the user to `bonesdeploy
       update`. No ordinary command detects, adapts to, or silently supports
       the old layout.
- [x] Refactor local provisioning, doctor, manifest, runtime, services, SSL,
      helpers, status, rollback, update, and CLI config/help flows to consume
      the new local inputs and machine conventions.
- [x] Remove `bonesdeploy push`, config-repository setup, site import/receive,
      infrastructure hooks, and deploy-on-push behavior from both binaries and
      their embedded assets.
- [x] Move the existing encrypted project secret path into `infra/secrets/`,
      preserve encrypted bytes and protected remote shared-state delivery, and
      ensure GPG private keys, keyrings, plaintext, and other decryption
      authority never enter Git or build execution.
- [x] Add the BonesRemote immutable revision boundary: resolve the full SHA,
      validate site identity, read revision-owned deployment files, and pass
      one snapshot through stage, checkout, build, prepare, seal, activate,
      rollback, and cleanup without mutable site configuration reloads.
- [x] Update BonesInfra context loading and project infrastructure execution
      to use local `.env` plus committed `infra/` behavior without introducing
      a replacement TOML configuration layer.
- [x] Remove `Runtime.shared`, `SharedPath`, and `SharedPathType`; make setup
      create `shared/.env` for every framework and make deployment wire that
      file into each release.
- [x] Add framework-owned lists of directory-valued environment variables and
      create only those directories during provisioning. Do not create
      application data files such as SQLite databases.
- [x] Implement the explicit BonesDeploy update path for managed provisioning:
      refresh unmodified `infra/.framework/` files, preserve all custom
      files, and report modified-core conflicts without silently overwriting
      them or performing a three-way merge.
- [x] Restore the versioned Python update-patch registry and implement the
       `0.8.0` local patch that safely transitions `.bones` to `infra/`,
       preserves files and encrypted secret bytes, leaves machine-local GPG
       state untouched, refuses unsafe layouts, and does not create Git
       commits. `bonesdeploy update` is the single bridge; no ordinary command
       supports the old layout.
- [ ] Remove obsolete templates, fixtures, hooks, and code exposed by the
      refactor, keeping unrelated runtime permission behavior unchanged.

### Validation

- [ ] Add focused parser, identity, path, revision-consistency, and
      build-secret-boundary tests at the owning Rust/Python modules.
- [x] Add isolated CLI tests for fresh init, old-project migration, explicit
      deploy behavior, and rejection of obsolete transport/config commands.
- [ ] Add BonesRemote tests proving differing Git revisions cannot mix source,
      deployment scripts, infrastructure code, or scalar build inputs.
- [ ] Add regression tests proving encrypted secrets remain encrypted in Git,
      runtime plaintext is excluded from build inputs, secret delivery retains
      protected ownership and mode, and migration leaves local GPG keyrings
      untouched.
- [x] Add shared-state tests proving `.env` is created and wired universally,
      declared framework directories are created, and application data files
      are not created by BonesDeploy.
- [x] Run affected Rust and Python tests and inspect their observable results.
- [x] Run `cargo clippy` and address all warnings/errors.
- [x] Run `cargo fmt` and verify no formatting diff remains.
- [x] Run `shfmt -w .` and verify shell files remain valid.
- [x] Run the Python lint/format checks required by
      `crates/bonesinfra/python/AGENTS.md` without running e2e tests.

### Completion

- [x] Update `CONTEXT.md`, `README.md`, and related BonesInfra documentation
      so they describe project-local `infra/`, local provisioning inputs, and
      explicit deployment only.
- [ ] Search for stale `.bones`, `bones.toml`, config-repository,
      import/receive, and deploy-on-push references and remove or intentionally
      document every remaining internal compatibility reference.
- [ ] Review the final diff against `01-idea.md` and `02-plan.md`; record only
      meaningful deviations, validation results, discoveries, and deliberately
      unfinished excluded work in completion notes.

### Completion notes

- Shared-state implementation is complete: setup owns `shared/.env`, release
  wiring is unconditional, and framework runtimes declare directory-only
  shared paths.
- Removed `runtime.shared` configuration is rejected instead of silently
  ignored through runtime's flattened extension values.
- Focused shared-directory and release-wiring coverage exists; setup creation
  and application-data-file non-creation assertions are covered by the
  framework provisioning tests.
- The general-purpose `Bones` type remains as an internal compatibility-shaped
  value object for remote machine inputs; project-local persistence is now the
  flat root `.env`, and no command writes or loads a project `bones.toml`.
- The old `.bones` layout is transitioned only by the version-gated `0.8.0`
  update patch; ordinary commands reject it. E2E fixtures and historical plan
  references retain explanatory legacy examples and are not runtime
  compatibility paths.
- Restored the Python patch registry with the local `0003-project-infra` patch
  and marker-only remote completion. The patch's focused tests cover version
  selection, ciphertext-preserving migration, refusal without a marker, and
  remote marker creation.
- Validation passed: `uv run pytest` (379 tests), `cargo test -p bonesdeploy
  -p bonesinfra`, `cargo clippy`, `cargo fmt --check`, `ruff check .`, and
  `shfmt -d .`. E2E tests were not run.
- Full Python and Rust validation is recorded in the implementation session;
  e2e tests were intentionally not run.


## Clarification

### Trigger

The user directed: "we should not be trying to do any backwards compatibility for existing workspaces using this tool." Commands must not detect, adapt to, or silently support old `.bones`-based project layouts.

### Decision

This refactor is a hard cut. Once `bonesdeploy` is updated, it will not read or adapt to the old `.bones` symlink, `bones.toml`, nested Git, or config-repository layout. Commands that encounter the old layout fail with a clear message directing the user to run the explicit migration. No dual-mode code paths, fallback loading, or format auto-detection survive.

The migration tool is the single deliberate bridge: it copies project-owned
files from `.bones` into `infra/` and removes the old workspace structure. It
does not delete or relocate machine-local BonesDeploy state, including GPG
keyrings. After migration, the tool works against `infra/`. Before migration,
it refuses to deploy, setup, or provision.

### Supersedes

Adds an explicit constraint without changing an earlier decision. The previous plan already described migration as deliberate and commands as adopting new boundaries, but did not explicitly rule out auto-detection or dual-mode fallback behavior.

### Effect on the record

- `01-idea.md`: Added no-backwards-compatibility to Constraints.
- `02-plan.md`: Specified that commands encountering old layout fail with a migration message rather than detecting or adapting. Migration section clarified as the single bridge.
- `03-tasks.md`: Added task for clear failure messages on old layout; migration task emphasizes no auto-detection.


## Clarification

### Trigger

The prior wording about removing config-root copies could be read as deleting
the entire `~/.config/bonesdeploy` tree or moving GPG private keys into the
application repository. The user clarified that machine-local GPG state must
remain protected and available.

### Decision

The refactor removes only the obsolete per-project configuration repository,
its nested Git transport, and its project-local workspace representation. It
does not delete or relocate the BonesDeploy XDG data/configuration roots.

GPG private keys, keyrings, trust data, and other decryption authority remain
machine-local. Only encrypted project secret ciphertext may be placed under
committed `infra/secrets/`; plaintext secrets and private keys never enter Git
or build execution. Migration copies encrypted bytes without decrypting them
and leaves the local keyring untouched.

### Supersedes

Supersedes the broad interpretation of “remove config-root copies” and
“remove the old structure” from the preceding planning record. Those phrases
refer only to obsolete per-project configuration state, not machine-local
BonesDeploy state or GPG key material.

### Effect on the record

- `01-idea.md`: Defines local key material, narrows removal to per-project
  configuration state, and requires preservation of local BonesDeploy roots.
- `02-plan.md`: Separates encrypted project ciphertext from the machine-local
  GPG keyring and makes migration preserve local key material.
- `03-tasks.md`: Requires preserving the keyring during initialization,
  secret relocation, and migration, while forbidding private keys in Git or
  build inputs.


## Clarification

<!--
This document records a settled change to the existing planning record.

Never use a clarification file to ask a question or preserve undecided
alternatives.
-->

### Trigger

<!-- State the new information or instruction that required clarification. -->

### Decision

<!-- State the settled decision and why it was made. -->

### Supersedes

<!--
Identify the earlier definition, assumption, scope boundary, or decision
this replaces. If it replaces nothing, state that it adds detail without
changing an earlier decision.
-->

### Effect on the record

<!--
Summarize the corresponding updates made to 01-idea.md, 02-plan.md,
and 03-tasks.md.

The clarification is not complete until the authoritative files reflect
the new current truth.
-->
## Clarification

### Trigger

Moving BonesInfra project-facing provisioning into the application repository
needs an ownership boundary that keeps infrastructure visible and versioned
without permanently forking BonesDeploy-managed behavior.

### Decision

Project-facing provisioning is split into two committed directories:

- `infra/.framework/` contains BonesDeploy-supplied, managed framework code
  code. It remains visible to users and may be refreshed by an explicit
  BonesDeploy update.
- `infra/custom/` contains project-owned provisioning code. Updates
  preserve it and never silently overwrite it.

Core provisioning runs before custom provisioning through ordinary explicit
Python composition. Managed core is the complete copied BonesInfra execution
engine, so every project can execute its committed core without a hidden source
fallback. No general plugin registry is introduced.

An update refreshes managed core files only when their project copies are
unmodified. If a managed file was changed, update reports the conflict and
refuses to overwrite it. The update path does not perform a three-way merge.
There is no canonical per-project infrastructure tree under
`~/.config/bonesdeploy`; the application repository is authoritative.

### Supersedes

Superseded by `refactor/project-local-bonesinfra-core`: managed core is the
complete copied BonesInfra distribution rather than a framework-only snapshot.

### Effect on the record

- `01-idea.md`: Defines managed core and custom provisioning, adds the desired
  directory structure, and records update and composition constraints.
- `02-plan.md`: Adds current-state analysis, scaffolding, composition,
  update-conflict behavior, responsibilities, risks, and validation for the
  ownership boundary.
- `03-tasks.md`: Adds implementation tasks for core/custom scaffolding,
  explicit composition, and safe managed-core updates.


## Clarification

<!--
This document records a settled change to the existing planning record.

Never use a clarification file to ask a question or preserve undecided
alternatives.
-->

### Trigger

The shared-path proposal was simplified: application files such as Laravel's
`database.sqlite` are created by the application, while BonesDeploy should only
create directories needed by runtime configuration, with `.env` as the one
managed file.

### Decision

Remove the configurable `runtime.shared` object and its `SharedPath` types.
Every setup creates `shared/` and `shared/.env` by default; `.env` remains the
only shared file that BonesDeploy creates and wires into releases.

BonesDeploy may create known framework directories referenced by local
environment values, such as Laravel's `LARAVEL_STORAGE_PATH`,
`VIEW_COMPILED_PATH`, `CACHE_PATH`, and `UPLOADS_PATH`. Framework modules define
which environment variables represent directories. BonesDeploy does not create
application data files such as `database.sqlite`; the application owns those
files.

The deployment path always links `shared/.env` into the active release as
`.env`. No generic shared-path allowlist or file/directory inference is used.
The initial directory creation uses the existing runtime ownership and a
consistent writable directory mode; framework-specific permission policy is
not introduced as part of this clarification.

### Supersedes

Supersedes the earlier proposal to infer both files and directories from every
path-like `.env` value and to add framework-specific file permission objects.
It adds a narrower rule to the existing decision to remove `runtime.shared`.

### Effect on the record

- `01-idea.md`: Defines `shared/.env` as the only managed shared file and
  application-owned data files as excluded from automatic creation.
- `02-plan.md`: Replaces generic shared-path inference with framework-declared
  directory creation and unconditional `.env` wiring.
- `03-tasks.md`: Adds removal of the shared object, default `.env` wiring, and
  directory-only creation behavior.


## Clarification

### Trigger

The decentralization implementation removed the existing update-patch system
and added a standalone `bonesdeploy migrate` command. This left `update`
calling a deleted Python command and made updates fail.

### Decision

Restore the Python-owned ordered update-patch registry. The `0.8.0` local patch
is the only transition from the old `.bones` workspace to project-local
`infra/`. It copies only validated project-owned content, preserves encrypted
secret bytes, verifies the copied tree before removing the old workspace, and
does not create a Git commit. It writes its local completion marker only after
the transition succeeds.

`bonesdeploy update` is the single deliberate bridge. The standalone
`bonesdeploy migrate` command is removed. Ordinary commands continue to reject
old `.bones` layouts and direct users to `bonesdeploy update`. Remote patch
execution remains part of the patch system but this layout-only patch performs
no obsolete config-repository operation remotely.

### Supersedes

Supersedes the standalone migration-tool decision in
`04-no-backwards-compat-clarity.md` and the corresponding migration-command
references in the authoritative record.

### Effect on the record

- `01-idea.md`: Names `bonesdeploy update` as the single versioned bridge.
- `02-plan.md`: Restores update-patch ownership and assigns the layout
  transformation to the `0.8.0` local patch.
- `03-tasks.md`: Replaces the completed standalone migration task with the
  completed patch-registry and update-transition task.
