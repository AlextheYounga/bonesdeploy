# Project-Local Infrastructure Refactor Plan

## Current behavior

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
call `load_site_config`; checkout resolves a revision and exports the complete
bare-repository tree, while build and prepare scripts are read from persisted
site state. Runtime settings and shared paths are also reloaded from that
state. The deployment record already stores the resolved full revision SHA.

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

## Intended behavior

`bonesdeploy init` establishes project identity and local provisioning inputs
without creating `.bones`, a nested repository, or a remote config repository.
It scaffolds committed `infra/` content and `.env.build`, and updates the
application repository ignore rules so generated infrastructure and encrypted
secret paths are handled deliberately. Local `.env` supplies connection and
provisioning-only values. Existing machine-local BonesDeploy data, including
the GPG keyring under the data root, is preserved. Project-facing provisioning
is scaffolded under `infra/provision/core/` and `infra/provision/custom/`.
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

## Approach

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
7. Separate project-facing provisioning into managed `infra/provision/core/`
   and project-owned `infra/provision/custom/`. Scaffold both, compose them
   explicitly in that order, and make update refresh only unmodified managed
   files. If a managed file was modified, update reports the conflict and
   refuses to overwrite it; it does not perform a three-way merge.
8. Remove the generic `runtime.shared` model. Always create and wire
   `shared/.env`; let framework modules identify directory-valued environment
   variables whose directories should be created. Do not create arbitrary
   files or infer file types from path-like values.

## Responsibilities and boundaries

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
- Update flow: refreshes BonesDeploy-managed `infra/provision/core/` files only
  when their project copies remain unmodified. It preserves
  `infra/provision/custom/` and reports modified-core conflicts without
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

## Affected areas

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

## Decisions

- `infra/` is the sole project-owned infrastructure directory and is ordinary
  application Git content.
- Project-facing provisioning is split into visible managed
  `infra/provision/core/` and project-owned `infra/provision/custom/`.
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

## Risks

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

## Validation

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
- Test that initialization creates core and custom provisioning directories,
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
