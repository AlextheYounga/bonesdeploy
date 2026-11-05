# Local Artifact Builds

<!-- Merged from feature/local-artifact-builds -->

## Idea

### Request

Make local artifact builds the only BonesDeploy application build path in the
next breaking release. The local BonesDeploy CLI builds the committed
application in Docker and uploads the completed native release or Compose image
artifact. BonesRemote does not retain a server-side application build path.

Synchronize only the release and backend-specific runtime values BonesRemote
still consumes. Local build configuration and arbitrary framework settings do
not belong in the remote control-plane snapshot.

Support only Debian 12 or newer and Ubuntu 24.04 or newer as production host
operating systems. Existing installations and configuration formats do not
constrain this breaking release; migration support can be designed separately.

### Problem

Building applications on production hosts consumes scarce CPU, memory, storage,
and I/O, and requires build users, rootless container state, caches, and builder
images on every server. Retaining both local and remote build modes would keep
that operational and security complexity even though the project does not need
compatibility at this stage.

The first local implementation established the artifact protocol and shared
release lifecycle, but deliberately retained remote builds and used rootless
Podman for compatibility. Docker is more widely available on developer machines
and is an acceptable local trust dependency when the build receives no host or
production secrets and no privileged host interfaces.

Production platform support is also broader than the environments the project
intends to validate. A narrow Debian and Ubuntu contract makes provisioning,
diagnostics, service management, and operational documentation explicit.

### Definitions

**Local artifact build:** The sole application build path. BonesDeploy exports
the exact configured branch commit and builds it with local Docker. Native
projects run numbered build scripts and package the resulting runtime tree.
Compose projects build and collect the exact service images plus the committed
release tree and generated release override. Neither path builds the working
tree or executes application builds on production.

**Build environment:** The fixed, non-secret metadata required by the build
contract plus values explicitly declared in the committed `.env.build` file,
such as `NODE_VERSION`. Every `.env.build` value is public build configuration,
not a secret. The build environment excludes the ambient host environment, root
`.env`, production variables, credentials, tokens, SSH agents, and Docker
socket.

**Release artifact:** A versioned, checksummed transport containing everything
production needs to activate one immutable release. A native artifact contains
the complete runnable filesystem tree and prepare scripts. A Compose artifact
contains the committed release tree, generated release-specific override, image
inventory, and exact service images. Artifacts exclude Git metadata, environment
files, production shared state, server control-plane state, local build caches,
and build-only content not needed by the selected runtime.

**Production host:** The machine running BonesRemote and application services.
A supported production host runs Debian 12 or newer or Ubuntu 24.04 or newer.
Other Linux distributions and older releases are unsupported.

**Prepare:** The existing server-side phase that runs numbered prepare scripts
against the promoted candidate as the runtime user after shared paths are wired.
Database migrations and environment-dependent runtime preparation remain
prepare work. Dependency installation, compilation, and other application build
work do not.

**Legacy remote build:** The server-side Git export, build-user, rootless-Podman,
and build-cache pipeline. It is removed rather than retained as a selectable mode
or automatic fallback.

### Desired Outcome

`bonesdeploy deploy` always builds the configured committed revision locally in
Docker before opening the deployment transaction. BonesRemote safely imports
the resulting native tree or Compose images and uses the existing prepare,
preflight, activation, restart, rollback, pruning, and cleanup lifecycle without
consulting an application repository or building application code.

Build or packaging failure leaves production untouched. Upload, validation,
prepare, or activation failure preserves existing cleanup and rollback
semantics. No failure can execute a build on production.

Fresh server provisioning and diagnostics accept only supported Debian and
Ubuntu versions. Production hosts contain no application source repository,
application builder image, per-site build user, build cache, or rootless Podman
state. Compose hosts retain Docker Engine and the Compose plugin only to load and
run prepared images.

### Scope

- Remove native build-mode configuration, selection, and compatibility behavior.
- Replace the local rootless-Podman runner and diagnostics with Docker.
- Keep the pinned Debian builder image, Linux `x86_64` target, clean Git revision
  provenance, complete-context artifact, and strict artifact protocol.
- Restrict build inputs to fixed non-secret contract metadata and explicitly
  declared, committed `.env.build` values.
- Remove native server checkout/build execution and its build users, caches,
  rootless container resources, image-store requirements, diagnostics, and tests.
- Build Docker Compose service images locally, upload them with their committed
  release tree, and prohibit production Compose pulls and builds.
- Remove production application bare repositories, Git-push readiness, branch
  resolution, and source materialization for every runtime backend.
- Produce complete runnable framework artifacts and remove build-only dependency
  trees, caches, and scripts that the selected production runtime does not use.
- Retain one BonesRemote artifact receipt and release lifecycle for native and
  Compose releases.
- Replace the complete local runtime model in config sync with a narrow native
  or Docker remote runtime descriptor, and remove unused remote prepare inputs.
- Enforce and document Debian 12+ and Ubuntu 24.04+ production support in
  provisioning and diagnostics.
- Update focused, regression, ignored E2E, architecture, security, and user
  documentation for the breaking contract.

### Constraints

- Docker is the sole supported local build engine. Builds use Linux containers
  and the project-owned builder image pinned by digest.
- Builds target Linux `x86_64`. Non-`x86_64` build machines must provide working
  Docker emulation and pass an execution probe before the build starts.
- Docker containers are never privileged and never receive the Docker socket,
  host home directory, SSH configuration, credential stores, production state,
  or ambient host environment.
- The local root `.env`, decrypted production environment, backup settings,
  ambient host variables, and other secret variables are not build inputs.
- `.env.build` remains the explicit project-owned input for non-secret build-tool
  settings. It must not contain passwords, tokens, private keys, or credentials.
- Local Git remains the source-selection and revision-provenance boundary. The
  artifact records the exact locally exported commit; production does not
  require a second repository or pushed branch.
- Artifact input remains untrusted at the privileged boundary. BonesRemote
  retains framing, size, digest, path, entry-type, symlink, file-count, and
  extracted-size enforcement before promotion.
- BonesInfra provisioning requests and production `shared/.env` secrets remain
  separate from the narrow BonesRemote deployment configuration contract.
- Prepare remains server-side with its current runtime identity and shared-state
  access, but may not install application dependencies or compile the release.
- Compose images are built for `linux/amd64`, receive immutable release-specific
  tags, and are started remotely with pulls and builds disabled.
- Production support is limited to Debian 12+ and Ubuntu 24.04+; unsupported
  distributions or versions fail clearly rather than continuing best-effort.
- E2E tests are authored for human execution and are not run by an agent unless
  explicitly requested.

### Exclusions

- Automatic fallback to a server-side build or another local container engine.
- Compatibility guarantees, in-place migrations, or automatic repair for
  installations created by earlier releases.
- Private dependency credentials or any mechanism that exposes secrets to
  arbitrary application build scripts; `.env.build` is not a secret channel.
- Registry publishing or registry-mediated deployment; Compose images travel in
  the release artifact over the existing SSH transport.
- Production support for Arch Linux, macOS, Windows, or Linux distributions
  other than the specified Debian and Ubuntu releases.
- A broader native build-host support promise; Docker Desktop, WSL2, and native
  Windows CLI portability require their own validation and decisions.
- Artifact registries, signing, provenance services, SBOM generation, or
  reproducible-build certification.


## Plan

### Current Behavior

Native deployment resolves and exports the configured local branch commit,
builds numbered scripts in local Docker, packages the complete resulting tree,
synchronizes deployment configuration, and streams a framed artifact to
BonesRemote. The Docker runner uses a pinned Linux `x86_64` image, scoped cache,
explicit public build environment, per-script timeout, detached container,
repeated Docker preflight, and ownership-restoring cleanup.

BonesRemote still resolves the configured branch in a production bare
repository before accepting a native artifact and rejects an artifact whose
revision does not match that separately pushed ref. Docker Compose deployment
still takes the ordinary no-artifact path: BonesRemote exports the pushed source
revision, runs Compose `config`, `pull`, and `build` on production, then promotes
and activates the release.

Both paths converge on `DeploymentLifecycleCoordinator` for staging, candidate
materialization, shared-state wiring, prepare, sealing, preflight, activation,
restart, rollback, pruning, cancellation, and cleanup. The persisted phases
still use source-export and build terminology even when those operations already
happened locally.

BonesInfra provisions no native build users, caches, Podman state, or builder
image store. It still provisions a bare repository for every site and carries
repository and branch fields through paths, contexts, manifests, deletion, and
diagnostics. It provisions the production runtime, services, security policy,
backups, and operational infrastructure required after a release arrives.

Framework artifacts are complete only inconsistently. Laravel installs Composer
dependencies locally, while Django installs application requirements during
remote prepare. Node-based artifacts retain build-time dependency trees and
caches even when their production runtime needs only compiled or standalone
output.

### Intended Behavior

All application builds happen locally. `bonesdeploy deploy` resolves and exports
the configured branch commit once, constructs the selected runtime's complete
artifact with local Docker, synchronizes deployment configuration, and streams
one framed deployment to BonesRemote. Production never checks out application
source, resolves an application Git ref, downloads application dependencies,
pulls Compose images, or builds application code or images.

Deployment configuration contains only release retention and the selected
backend's remotely consumed values. Native configuration carries the web root
and optional Ruby version needed by remote Rails migrations. Docker
configuration carries the optional ingress port used by diagnostics and the
Compose startup timeout. Framework identity, Node and Python build versions,
permission defaults, and arbitrary runtime extras remain local or in the
separate BonesInfra provisioning request.

Native artifacts contain a complete runnable release tree. Framework build
scripts install dependencies and compile all deterministic output locally, then
remove build-only dependency trees, caches, and numbered build scripts that the
runtime does not need. Remote prepare performs only production-state operations
such as shared-path setup, migrations, production-secret-based configuration,
and runtime validation.

Compose artifacts contain the committed release tree, a generated BonesDeploy
override, an image inventory, and a bounded Docker image archive. BonesDeploy
runs Compose validation, pull, and build locally for `linux/amd64` with the same
stable project name used remotely. It discovers every resulting service image,
assigns an immutable tag derived from site, service, and revision, and writes an
override selecting those tags. Production validates and loads the archive and
starts the stack with `--no-build --pull never`.

The production release directory retains the original Compose files for
operational inspection and bind-mounted release content. The generated override
is always passed last and is owned by the release protocol, not the project.
Rollback activates the previous release directory and its immutable image tags.
Failed-release cleanup and release pruning remove tags owned solely by the
discarded release while allowing Docker to retain shared layers still referenced
by another release.

Local Git is authoritative for source selection. The full commit hash remains in
the artifact manifest, release name, deployment state, status, and logs as audit
metadata. Production does not maintain an application repository or require a
first push. Compose and native sites use the same artifact-only BonesRemote
deploy entry point.

The local Docker runner provides a clean compatible build environment rather
than production tenant isolation. It retains the pinned image and platform,
explicit environment, absence of production secrets and ambient variables,
narrow project/cache inputs, timeout, streamed output, and reliable cleanup. Its
command lifecycle is simplified where direct foreground Docker execution can
replace detached containers, redundant probes, copied deployment trees, and
ownership-repair machinery without weakening those guarantees.

BonesRemote treats all artifact bytes as untrusted privileged input. It retains
strict framing, identity, digest, path, entry-type, symlink, conflict, file-count,
compressed-size, and expanded-size enforcement. The manifest identifies artifact
kind and enforceable payload contents. It does not claim that a client-supplied
builder image identifier proves build provenance.

BonesInfra provisions the production machine and execution environment only. It
retains supported-system validation, server hardening, deploy/runtime identities,
BonesRemote and constrained sudo, release/shared/configuration paths, language
runtimes, Nginx, systemd, AppArmor, Docker Engine and Compose runtime, backups,
SSL, tunnels, manifests, deletion, and patches. It removes application bare
repositories and repository-derived configuration, readiness, and inventory.

### Approach

Extend the artifact protocol with an explicit native or Compose artifact kind
and kind-specific validated metadata. Keep the common framed manifest, declared
length, SHA-256, and safe filesystem extraction. Add a bounded Compose image
payload and image inventory whose service names and release tags are validated
against the site and revision before invoking Docker.

Refactor local source export into the common start of every deployment. Native
projects execute their numbered build scripts and package the pruned result.
Compose projects execute Docker Compose against the exported context with
controlled `linux/amd64` defaults and `.env.build`, collect both locally built
and pulled service images, add immutable release tags, generate the protected
override, and save those exact tags into the artifact. Production `.env` remains
absent from every local build.

Make framework build and prepare scripts obey the complete-artifact boundary.
Move Django dependency installation into local build output. Preserve remote
migrations and environment-dependent framework work. Add framework-specific
pruning after successful builds, including removal of Laravel root
`node_modules`, Next/Nuxt/SvelteKit/Vue build caches and unnecessary dependency
trees according to their runtime output, and numbered native build scripts.

Replace BonesRemote's source/artifact input split with one artifact input.
Receive and validate the payload, materialize the release tree, load and verify
Compose images when present, promote the candidate, wire shared state, run
native prepare where applicable, seal, preflight, activate, and restart. Remove
server repository resolution, archive export, Compose pull/build, and source
deployment routing. Start Compose with builds and pulls disabled.

Replace the control-plane snapshot's complete local `Runtime` value with one
serde-tagged remote runtime enum. Convert it back into the minimal site runtime
state required by the existing release lifecycle at the BonesRemote boundary.
Remove the unused Python-version prepare environment and tests that preserve
arbitrary runtime extras. Reject old broad snapshots under the breaking-release
compatibility policy rather than retaining a legacy parser.

Rename persisted phases and coordinator operations around remote facts rather
than historical source/build operations. Preserve compatibility only with state
written by the current breaking-release branch where concrete recovery requires
it; do not retain obsolete application build behavior.

Remove repository fields and resources from canonical remote/provisioning
transports and BonesInfra. Site setup creates control-plane, project, release,
shared, placeholder, backup, and runtime resources without a bare repository.
Local initialization no longer creates a deployment Git remote, and setup and
doctor no longer instruct users to make a first push.

Keep the existing deploy SSH principal and narrowly anchored sudo policy, but
rename Git-specific terminology where it describes artifact transport rather
than source hosting. Rebuild the packaged BonesInfra wheel after Python changes.

### Responsibilities And Boundaries

`bonesdeploy-core` owns artifact framing, enforceable manifest types, target
platform, explicit build environment vocabulary, and shared validation. It does
not model server application repositories or unverifiable build attestation. It
also owns the narrow backend-specific remote deployment descriptor.

`bonesdeploy` owns local committed-source selection, native and Compose build
execution, build cache, framework pruning, Compose image collection and immutable
tagging, artifact packaging, progress, failures, and SSH upload.

`bonesremote` owns privileged artifact receipt, safe extraction, Compose image
loading, site locking, lifecycle state, promotion, production preparation,
activation, rollback, image/release pruning, cancellation, and cleanup. It never
builds application source or pulls application images.

BonesInfra owns production operating-system policy and resources required to run
imported releases. It installs runtime interpreters and services but does not
install application dependencies into a release or provision source/build
infrastructure.

Framework build scripts own application dependency installation, compilation,
runtime artifact layout, and removal of build-only content. Prepare scripts own
production-state transitions and environment-dependent runtime preparation.

### Affected Areas

- Artifact and configuration contracts in `bonesdeploy-core`, including artifact
  kind, Compose image inventory, repository-field removal, narrow remote runtime
  configuration, and lifecycle terms.
- BonesDeploy init, deploy, doctor, local Docker execution, Git export, native
  packaging, Compose build/image export, progress, and tests.
- Framework build/prepare assets for complete runtime output and pruning.
- BonesRemote deploy CLI and coordinator, artifact receipt, Compose image load,
  runtime start, rollback, failed-release cleanup, pruning, doctor, state, and
  tests.
- BonesInfra request/context/path models, site directories, manifests, deletion,
  runtime provisioning, packages, tests, and generated wheel.
- E2E setup and scenarios for artifact-only native and Compose first deploy,
  subsequent deploy, failed activation, rollback, and pruning.
- README, context, architecture, security, framework, and operational docs.

### Decisions

- Native and Docker Compose releases build locally and deploy only through the
  artifact protocol.
- Production application bare repositories and first-push workflows are removed.
- Local committed Git state is the sole source-selection boundary; revision is
  retained as audit metadata rather than compared with a production ref.
- Compose service images travel over SSH in the deployment artifact. A registry
  is not part of this change.
- Compose images use immutable release-specific tags, and remote Compose start
  disables both builds and pulls so rollback remains release-correct.
- `.env.build` is public build configuration for native tools and Compose build
  interpolation. Production `.env` and ambient host variables remain excluded.
- Complete artifacts include application dependencies. Remote prepare cannot
  install or compile them.
- BonesRemote config sync carries only values with current remote consumers;
  BonesInfra provisioning and production secrets remain separate contracts.
- Framework-specific artifact pruning is required, not deferred.
- Local Docker is a compatibility/reproducibility dependency, not a production
  isolation boundary. Secret exclusion and narrow tool-owned inputs remain
  mandatory.
- Artifact receipt remains a strict privileged trust boundary.
- BonesInfra remains the runtime and machine provisioner; friendly public CLI
  commands and operational inspection remain in scope and are not cleanup
  targets.
- Production support remains Debian 12+ or Ubuntu 24.04+ on `x86_64`.
- Previous-installation migration, registries, private build credentials,
  signing, SBOMs, and resumable upload remain separate changes.

### Risks

- Compose image archives can be substantially larger than native filesystem
  artifacts. Limits must remain explicit while accommodating a realistic
  multi-service release, and interrupted uploads must leave removable state.
- Compose files can vary topology through interpolation and profiles. Public
  build inputs must determine the complete image topology; production secrets
  cannot select an image that was absent from the artifact.
- Mutable Compose tags would silently break rollback. Generated overrides,
  remote start commands, pruning, and tests must prove release-specific image
  identity end to end.
- Loading images mutates Docker state before activation. Abort and pruning paths
  must remove release-owned tags without deleting layers or images used by the
  active or previous release.
- Framework pruning can remove runtime-required dependencies. Each framework
  needs observable runtime-output tests rather than a universal exclusion rule.
- Locally built native extensions must be compatible with the supported runtime
  and production ABI. Builder runtimes and production runtime versions must stay
  aligned.
- Simplifying Docker execution can regress timeout, output, cache, ownership, or
  cleanup behavior. Tests must preserve outcomes rather than the detached
  container implementation.
- Removing repository state touches provisioning, deletion, doctor, setup,
  configuration, and recovery assumptions across Rust and Python.
- Narrowing the persisted control-plane snapshot can expose files written by a
  pre-breaking-release binary. This is intentional under the no-compatibility
  policy; every deployment synchronizes the new descriptor before invoking the
  artifact lifecycle.
- Omitting a remotely consumed backend value would break prepare, preflight,
  Compose startup, or diagnostics. Focused consumer and serialization tests must
  cover every retained field and prove unrelated fields are absent.

### Validation

- Core protocol tests round-trip native and Compose manifests and reject wrong
  kinds, sites, revisions, lengths, digests, image inventories, tags, paths,
  entry types, conflicts, symlinks, and resource-limit violations.
- Native build tests prove complete dependency output, framework-specific
  pruning, explicit build variables, secret exclusion, target compatibility,
  timeout, output, cache, ownership, and cleanup.
- Compose tests prove local `config`, pull, and build use the exported commit,
  controlled environment, stable project name, and `linux/amd64`; every service
  image is inventoried, immutably tagged, saved, loaded, and selected by the
  generated override.
- Remote Compose tests prove deployment and service restart use `--no-build
  --pull never`, perform no network pull or build, preserve release-specific
  rollback, and remove only pruned or failed release tags.
- Lifecycle tests prove both artifact kinds share receipt, locking, promotion,
  activation, failure rollback, cancellation, pruning, and cleanup without
  source-export or production-build phases.
- BonesInfra tests prove no backend provisions or declares a bare repository,
  branch ref, build user, builder cache, builder image, or application build
  operation while all runtime, service, security, backup, SSL, manifest, and
  deletion responsibilities remain intact.
- CLI and documentation tests prove initialization, setup, doctor, and deploy no
  longer create a deployment Git remote or require a first push, while public
  operational commands remain available.
- Remote configuration tests prove native and Docker descriptors carry exactly
  their backend-specific values, omit local build and provisioning data, derive
  the required internal site runtime, and reject old broad snapshot shapes.
- Remote prepare tests prove Rails still receives its Ruby version while no
  Python build version or arbitrary runtime extra is projected remotely.
- Ignored E2E definitions cover native and Compose first deploy, second release,
  failed activation, rollback, and old-release/image pruning. They compile but
  are not agent-executed.
- Run focused Rust and Python tests, full non-E2E suites, Clippy with warnings
  denied, Rustfmt, shfmt, Ruff checks and formatting, generated-wheel validation,
  `git diff --check`, and final architecture/security review.


## Tasks

The completed tasks below record the artifact foundation committed in
`0a52c666` and the Docker-only native follow-up committed through `66ded8e8`.
Clarifications `04` and `05` superseded the original opt-in Podman direction.
Clarification `06-unified-local-artifact-lifecycle-clarity.md` now supersedes
production Git revision comparison, Compose source builds, and deferred
framework pruning. Its pending follow-up is the authoritative remaining
execution state.
Clarification `07-narrow-remote-config-clarity.md` narrows the completed
artifact lifecycle's control-plane sync to values BonesRemote still consumes.
Clarification `08-prepare-build-boundary-clarity.md` restores the complete
Rails and Django artifact contract after distribution-runtime changes moved
dependency installation into remote prepare.

### Prepare Build Boundary Regression Correction

#### Implementation

- [ ] Retain the locally installed production Rails bundle and remove all
  Bundler installation from remote Rails prepare.
- [ ] Install and retain Django production dependencies and release launchers
  during the local build, and remove virtualenv creation and pip installation
  from remote Django prepare.
- [ ] Update focused regression tests and current documentation to enforce that
  prepare performs validation and production-state work without building.

#### Validation

- [ ] Run focused BonesDeploy and BonesRemote tests proving complete dependency
  output is packaged and neither prepare script invokes a package installer.
- [ ] Run all required non-E2E Rust, Python, lint, formatting, shell formatting,
  generated-artifact, and diff checks without warnings or failures.

#### Completion

- [ ] Review the final diff for package-manager execution in prepare, removed
  runtime dependencies, stale distribution-runtime documentation, and unrelated
  changes; record validation evidence below.

### Narrow Remote Configuration Follow-Up

#### Implementation

- [x] Replace the complete `Runtime` in `RemoteDeploymentConfig` with a tagged
  native or Docker descriptor that carries only backend-specific remote values.
- [x] Update BonesRemote lifecycle, runtime, status, and doctor consumers to use
  the narrow descriptor while deriving only the minimal internal site config.
- [x] Remove Python version and arbitrary runtime-extra handling from remote
  prepare and control-plane persistence; retain the Rails Ruby migration input.
- [x] Remove obsolete tests, imports, and direct dependencies exposed by the
  narrowed control-plane contract, and update architecture documentation where
  it describes config sync.

#### Validation

- [x] Add focused serialization tests proving native and Docker config sync omit
  local build, framework, permission, secret, and unrelated backend values and
  reject the old broad snapshot shape.
- [x] Run focused core, BonesDeploy, and BonesRemote tests, then all non-E2E Rust
  and Python tests without executing E2E scenarios.
- [x] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, Python Ruff checks and
  formatting, generated-wheel validation when applicable, and `git diff --check`.

#### Completion

- [x] Review the final diff for stale remote-build configuration, accidental
  provisioning or secret coupling, compatibility code, dead dependencies, and
  documentation drift; record validation evidence and any deliberate remainder.

Completion validation passed with `cargo test --workspace --exclude e2e
--no-fail-fast`, `cargo clippy --workspace --all-targets --all-features --exclude
e2e`, `cargo fmt --all -- --check`, `shfmt -w .`, `ruff check .`, `ruff format
--check .`, `uv run pytest`, and `git diff --check`. The focused transport suite
also covers exact backend-specific JSON, old broad snapshot rejection, Docker
limits, safe native web roots, identity derivation, and retaining only the Rails
Ruby prepare input. No E2E scenarios were executed.

### Implementation

- [x] Add canonical `BuildMode` configuration and strict local-artifact manifest
  and framing types to `bonesdeploy-core`; default omitted mode to remote, carry
  it through deployment and provisioning transports, reject local mode with the
  Docker backend, and cover backward-compatible serialization.
- [x] Pin the Debian builder image by digest for Rust and BonesInfra, define the
  shared Linux `x86_64` container contract and local cache paths, and update
  image-store tests and documentation that currently name the mutable tag.
- [x] Move numbered build-script discovery and sanitized build-environment
  projection into the shared core contract, update BonesRemote to consume it,
  and retain all existing denylist and `.env.build` behavior tests.
- [x] Add local Git helpers that resolve the configured local branch commit and
  export that exact commit into a private temporary context without working-tree,
  ignored, untracked, or Git metadata content.
- [x] Add the focused local rootless-Podman runner with pinned target/image
  preflight, protected environment file, `/workspace/source`,
  `/workspace/deployment`, scoped `/workspace/cache`, ordered scripts,
  per-script timeout, streamed output, and unconditional container cleanup.
- [x] Add complete-context artifact packaging with private temporary storage,
  gzip tar encoding, preserved executable files and relative symlinks, declared
  size, and SHA-256 over the exact transport bytes.
- [x] Add asynchronous SSH reader upload with concurrent stdout/stderr draining,
  then route local-mode `bonesdeploy deploy` through local build and packaging
  before config sync and framed artifact upload; never invoke remote fallback.
- [x] Extend the narrowly allowlisted BonesRemote deploy CLI and sudoers policy
  with the exact site-scoped artifact form, leaving all other optional,
  reordered, or trailing argument forms denied.
- [x] Add bounded BonesRemote artifact receipt and safe context materialization,
  including strict manifest/revision/target/image checks; length and digest
  enforcement; regular file, directory, and deferred safe-symlink extraction;
  and rejection of traversal, conflicts, hard links, special files, and resource
  exhaustion.
- [x] Refactor `DeploymentLifecycleCoordinator` around remote-source and
  local-artifact inputs so both record coherent deployment phases and share the
  existing promotion, shared wiring, prepare, sealing, preflight, activation,
  rollback, pruning, abort, cancellation, and cleanup behavior.
- [x] Make BonesInfra site provisioning, manifest declarations, deletion, and
  BonesRemote doctor checks build-mode-aware: local native sites omit per-site
  build-user, linger, slice, rootless storage, cache, and Podman readiness while
  remote and legacy sites retain them.
- [x] Add `--build-mode local|remote` to initialization and generated managed
  configuration with remote as the opt-in-phase default; update local doctor to
  diagnose Git, Podman, target-image, and cache prerequisites for local mode.
- [x] Update ignored E2E scenario definitions to exercise local artifacts for
  Laravel, Next server/static, Nuxt server/static, SvelteKit, Vue, Django, and
  Rails through first deployment, second release, and activation rollback,
  without running those scenarios during agent implementation.
- [x] Update README, context, architecture, and security documentation for clean
  pushed-revision provenance, local Podman trust, complete-context transfer,
  mode-specific server resources, unchanged prepare behavior, and explicit
  remote builds without silent fallback.

### Validation

- [x] Run focused core, local CLI, BonesRemote, and BonesInfra tests proving
  configuration compatibility, clean Git export, build-command parity, protocol
  framing, safe artifact round trips and rejection cases, pushed-revision
  enforcement, common lifecycle behavior, and mode-aware provisioning.
- [x] Run the full non-E2E Rust and Python test suites and resolve every failure;
  do not execute ignored E2E scenarios unless the user explicitly requests it.
- [x] Run `cargo clippy --workspace --exclude e2e`, `cargo fmt`, `shfmt -w .`,
  `ruff check .`, and `ruff format .`, then resolve every warning or error.
- [x] Review the ignored framework E2E definitions and provide the exact human
  validation matrix required before local mode can become the default for newly
  initialized projects.

### Completion

- [x] Review the final diff for accidental secret or working-tree inclusion,
  mutable builder references, unsafe archive extraction, unbounded resource
  use, broad sudo permissions, direct writes into releases, duplicate lifecycle
  behavior, silent fallback, and regressions to legacy remote mode.
- [x] Remove obsolete duplicated build-contract code and update all affected
  documentation so the repository has one current description of each build
  mode and trust boundary.
- [x] Record implementation deviations, validation evidence, and deliberately
  deferred default-switch, Compose, pruning, Django wheelhouse, signing, and
  resumable-upload work below.

### Completion notes

Implemented as planned with one defensive addition discovered during review:
the complete exported Git context removes a tracked root `.env` before build
scripts run, and the shared build environment excludes all backup settings so a
Borg passphrase cannot reach either build mode.

Validation completed successfully:

- `cargo test --workspace --exclude e2e`
- `cargo clippy --workspace --exclude e2e --all-targets -- -D warnings`
- `cargo check -p e2e --tests`
- `cargo clippy -p e2e --tests --no-deps -- -D warnings`
- `cargo fmt --all -- --check`
- `shfmt -w .`
- `ruff check .`
- `ruff format .`
- `uv run pytest` (`520 passed`)
- `git diff --check`

Ignored E2E definitions cover Laravel, Next server/static, Nuxt server/static,
SvelteKit, Vue, Django, and Rails through first deployment, second release, and
post-activation rollback. They were compiled but not executed, as required;
the human-run matrix is recorded in `e2e/README.md`.

The default switch, Compose artifacts, framework-specific pruning, Django
dependency packaging, signing/provenance services, and resumable uploads were
deferred at this milestone. Clarification `06` brings Compose artifacts,
framework pruning, and Django dependency packaging into the pending follow-up;
the other items remain deferred.

### Breaking Local-Only Follow-Up

#### Implementation

- [x] Remove `BuildMode`, `--build-mode`, mode transport fields, remote defaults,
  mixed-mode validation, and native remote-build selection from Rust and Python
  configuration boundaries.
- [x] Replace the local Podman runner and doctor checks with one Docker runner
  that verifies a Linux engine and `linux/amd64` execution, uses the pinned
  builder digest, passes no secret or ambient environment, and retains scoped
  cache, timeout, output streaming, and unconditional cleanup behavior.
- [x] Retain committed `.env.build` as the explicit non-secret project build
  input alongside fixed public workspace metadata; preserve reserved-name
  validation and prove ambient host, runtime, backup, and credential variables
  are absent.
- [x] Route every native deploy through local build, packaging, and artifact
  upload; remove the ordinary native source-deploy form and any fallback path.
- [x] Remove BonesRemote native checkout/build coordinator inputs, build-user and
  Podman readiness, native build cancellation branches, and build-only modules
  while preserving revision verification and the shared post-receipt lifecycle.
- [x] Remove BonesInfra native build users, linger, build slices, rootless
  storage, caches, Podman verification, and native builder image-store state;
  preserve runtime ownership and regenerate the packaged wheel.
- [x] Enforce Debian 12+ and Ubuntu 24.04+ production support consistently in
  provisioning and remote diagnostics, rejecting older, malformed, and other
  distribution identities before mutation.
- [x] Update ignored native framework E2E definitions and all user,
  architecture, security, support, and upgrade documentation for Docker-only
  local artifacts and the breaking production-host contract.

#### Validation

- [x] Run focused tests for Docker command safety, fixed public build metadata,
  unconditional artifact deployment, remote-build removal, production OS
  version boundaries, and mode-free provisioning transport.
- [x] Compile but do not execute ignored E2E scenarios covering first deploy,
  second release, and failed-activation rollback across the native framework
  matrix.
- [x] Run all non-E2E Rust and Python tests, Clippy with warnings denied,
  Rustfmt, shfmt, Ruff checks and formatting, generated-wheel validation, and
  `git diff --check`; resolve every warning or failure.

#### Completion

- [x] Review the final diff for remaining Podman, remote native build,
  build-mode, accidental `.env.build` secret semantics, unsupported
  production-host, secret exposure, broad Docker access, and stale compatibility
  behavior.
- [x] Record implementation deviations, exact validation evidence, and any
  deliberately deferred Compose, migration, build-host portability, artifact
  pruning, credential, signing, or resumable-upload work.

### Breaking Follow-Up Completion Notes

Implemented the breaking local-only native build contract. Native projects now
always export the configured commit, build it in local Docker, and upload a
framed artifact. Build mode configuration and native production build users,
Podman state, caches, image stores, checkout, and build execution were removed.
The separate Compose runtime keeps its existing source export and server-side
Compose `config`, `pull`, and `build` path; it cannot act as a native fallback.

Docker executes the pinned `linux/amd64` builder with no privileged mode, Docker
socket, host home, runtime environment, or ambient variable injection. The
committed `.env.build` remains explicit public build configuration. It must be a
regular file, is consumed before packaging, and is excluded from the release
artifact together with the root `.env`. Rootful Docker output ownership is
restored from inside the container before forced cleanup on success and failure.

Provisioning and remote diagnostics accept only Debian 12+ or Ubuntu 24.04+ on
`x86_64`. The packaged BonesInfra wheel was regenerated after removing native
server build resources.

Validation completed successfully:

- `cargo test --workspace --exclude e2e`
- `cargo clippy --workspace --exclude e2e --all-targets -- -D warnings`
- `cargo check -p e2e --tests`
- `cargo clippy -p e2e --tests --no-deps -- -D warnings`
- `cargo fmt --all -- --check`
- `shfmt -d .` after `shfmt -w .`
- `uv run ruff check .`
- `uv run ruff format --check .`
- `uv run pytest` (`504 passed`)
- `git diff --check`
- Two independent final reviews reported no remaining correctness or security
  findings after follow-up fixes.

Ignored E2E scenarios were compiled but not executed. At this milestone Compose
artifact transport and artifact pruning were deferred; clarification `06` brings
both into the pending follow-up. Migration support, broader build-host
portability, private dependency credentials, signing, and resumable uploads
remain deliberately deferred.

### Unified Local Artifact Lifecycle Follow-Up

#### Implementation

- [x] Extend the shared artifact contract with explicit native-tree and Compose
  image artifact kinds, enforceable kind-specific metadata, validated immutable
  Compose image tags, and realistic bounded payload limits without treating
  client-declared builder identity as attestation.
- [x] Refactor deploy so every backend starts from one exact local commit export;
  remove deployment Git remote creation, first-push guidance, remote repository
  paths, pushed-branch readiness, and server revision comparison while retaining
  the revision in release identity, state, status, and logs.
- [x] Simplify the native Docker runner around compatibility and reproducibility,
  preserving the pinned `linux/amd64` environment, explicit public inputs,
  secret exclusion, cache, timeout, streamed output, ownership, and reliable
  cleanup without retaining unnecessary detached-container or probe machinery.
- [x] Make every native framework build produce a complete runnable artifact:
  move Django application dependency installation local, verify runtime output,
  and remove framework-specific build-only dependency trees, caches, temporary
  files, and numbered build scripts only where the production runtime does not
  need them.
- [x] Add local Compose validation, pull, and build against the exported context
  using `.env.build`, the stable site project name, and `linux/amd64`; discover
  every service image, assign an immutable site/service/revision tag, generate
  the protected release override and inventory, and save the exact images into
  the artifact.
- [x] Replace BonesRemote source and artifact deploy routing with one artifact
  input; safely materialize either kind, validate and load Compose images, and
  preserve the common release transaction without repository export or any
  production application pull/build operation.
- [x] Start Compose releases with the generated override passed last and with
  `--no-build --pull never`; preserve release-correct restart and rollback, and
  remove release-specific image tags during failed-release cleanup and release
  pruning without removing images still referenced by retained releases.
- [x] Rename coordinator operations and persisted phases that currently report
  remote source export or build so state reflects receipt, materialization,
  production preparation, sealing, activation, verification, and cleanup.
- [x] Remove application repository creation, paths, branch context, manifest
  entries, deletion inventory, package assumptions, and diagnostics from
  BonesInfra while retaining the deploy transport identity and all machine,
  runtime, service, placeholder, security, backup, SSL, tunnel, manifest,
  deletion, and patch responsibilities.
- [x] Regenerate the embedded BonesInfra wheel and update user, framework,
  architecture, security, support, and operational documentation for the single
  local-build/artifact lifecycle and runtime-only BonesInfra boundary.

#### Validation

- [x] Add focused native artifact tests proving complete runtime dependencies,
  safe framework pruning, explicit public build inputs, no production or ambient
  secrets, target compatibility, timeout, output, cache, and cleanup.
- [x] Add focused Compose tests proving local-only `config`, pull, and build;
  complete image discovery; immutable tagging; override generation; bounded
  image transport; remote load; and `--no-build --pull never` activation.
- [x] Add release tests proving native and Compose first deploy, subsequent
  deploy, failed activation, rollback, cancellation, and pruning use the correct
  release tree and image identities without a production repository or build.
- [x] Add BonesInfra and CLI tests proving setup, doctor, manifest, and deletion
  contain no application bare repository or first-push state while preserving
  all runtime and operational resources and friendly public commands.
- [x] Compile but do not execute ignored native and Compose E2E scenarios for
  first deploy, second release, failed activation, rollback, and release/image
  pruning.
- [x] Run all non-E2E Rust and Python tests, Clippy with warnings denied,
  Rustfmt, shfmt, Ruff checks and formatting, generated-wheel validation, and
  `git diff --check`; resolve every warning or failure.

#### Completion

- [x] Review the final diff for any production application checkout, dependency
  installation, image pull/build, mutable Compose image tag, build-only artifact
  content, repository provisioning, first-push guidance, unverifiable provenance
  claim, accidental secret input, unsafe artifact receipt, or stale lifecycle
  terminology.
- [x] Record exact artifact-size limits, framework pruning outcomes, Compose image
  rollback/pruning evidence, implementation deviations, validation results, and
  deliberately deferred migration, registry, credential, signing, SBOM, and
  resumable-upload work.

### Unified Follow-Up Completion Notes

Native and Compose deployments now use artifact protocol version 2 and the same
artifact-only BonesRemote entry point. The manifest distinguishes native trees
from Compose image releases, validates at most 128 site/service/revision-derived
image identities, and caps the compressed transport at 2 GiB. Receipt additionally
caps one file at 2 GiB, the expanded tree at 4 GiB, paths at 4 KiB, and entries at
100,000 while retaining strict digest, type, conflict, and symlink checks.

Compose runs `config`, `pull`, and `build` locally from the exact exported commit
with the stable site project name, `linux/amd64`, and explicit public
`.env.build` values. The artifact carries the release tree, generated override,
validated inventory, and exact saved images. BonesRemote loads and verifies those
images, passes the generated override last, and starts with `--no-build --pull
never`. Abort, cancellation, failed-release cleanup, and pruning remove only
unreferenced release tags; rollback selects the retained release override and
immutable image identities.

Framework artifacts retain only required runtime output. Laravel and Rails retain
server dependencies while dropping frontend dependencies and caches. Next retains
standalone output, Nuxt retains `.output`, SvelteKit retains adapter-node runtime
dependencies, and Vue retains `dist`. Django installs requirements into
`.python-packages` locally and ships wrappers that invoke the provisioned matching
production interpreter; prepare performs only validation, migrations, and static
collection. Numbered build scripts and framework-specific build caches are removed
after successful builds.

Validation completed successfully:

- `cargo test --workspace --exclude e2e`
- `cargo clippy --workspace --exclude e2e --all-targets -- -D warnings`
- `cargo check -p e2e --tests`
- `cargo clippy -p e2e --tests --no-deps -- -D warnings`
- `uv run pytest` (`504 passed`)
- `uv run ruff check .`
- `uv run ruff format --check .`
- `cargo fmt --all -- --check`
- `shfmt -w .`
- `git diff --check`

Ignored E2E scenarios cover native and Compose first deployment, second release,
failed activation rollback, and release/image pruning. They compile but were not
executed. Migration from previous installations, registries, private dependency
credentials, signing, SBOM generation, and resumable upload remain deliberately
deferred.


## Local-Only Docker And Production Support Clarification

### Trigger

After the opt-in local artifact implementation was completed, the project chose
to make the next release a deliberate breaking change rather than preserve two
native build systems for a small installed base. The local build machine is an
acceptable place to depend on Docker when build containers receive no host or
production secrets. The project also narrowed its supported production platform
to the server environments it intends to validate and operate.

### Decision

Local artifact builds are the only native build path. Build mode is no longer a
configuration concept, and BonesRemote does not retain an explicit or automatic
server-side native build fallback.

Docker is the sole local build engine. Podman support, rootless enforcement, and
a generic engine-selection abstraction are removed. Docker runs the pinned
Linux `x86_64` builder without privileged mode, host credential mounts, the
Docker socket, ambient host variables, production variables, root `.env`, or
`.env.build`. Fixed non-secret workspace metadata remains part of the build
contract.

Supported production hosts are Debian 12 or newer and Ubuntu 24.04 or newer.
Provisioning and diagnostics reject older releases and other distributions
explicitly. This production support policy does not itself promise native
Windows or broader local build-host support.

The release does not guarantee in-place upgrades from earlier BonesDeploy
installations. Migration and repair behavior can be designed later from concrete
user needs instead of preserving the obsolete server build architecture now.

### Supersedes

This decision supersedes the original plan's opt-in `local|remote` build mode,
remote compatibility default, rootless Podman local runner, mixed-mode server
provisioning, server builder image requirement, and deferred default switch.

It does not weaken artifact receipt validation, Git revision provenance,
server-side prepare behavior, activation rollback, or the rule that failed local
builds never fall back to production execution.

### Required Authoritative Updates

- `01-idea.md` now defines local Docker artifacts as the only native build path,
  the no-secret build environment, the breaking compatibility boundary, and the
  supported production systems.
- `02-plan.md` now describes removal of build modes, remote native builds,
  Podman, server build resources, and user-defined build variables, plus Docker
  execution and production-version enforcement.
- `03-tasks.md` retains the completed artifact-foundation history and adds the
  pending breaking-release implementation, validation, and completion work.


## Build Environment Boundary Clarification

### Trigger

The Docker clarification described `.env.build` as excluded from local builds.
That was broader than intended. `.env.build` exists specifically so a project
can declare a small set of build-tool inputs such as `NODE_VERSION` without
exposing its runtime or production environment.

### Decision

Local Docker builds retain committed `.env.build` values as explicit build
inputs. The existing parser and reserved-name validation remain part of the
shared build contract. Fixed container-controlled and derived public metadata
continues to be supplied separately and cannot be overridden by `.env.build`.

Every `.env.build` value is treated as public. The file must not contain
passwords, tokens, private keys, private dependency credentials, or other
secrets because arbitrary build scripts and supply-chain dependencies can read
and exfiltrate any build input.

The build does not inherit the ambient host environment and does not receive the
root runtime `.env`, decrypted production variables, backup credentials, SSH
agent, credential stores, host home directory, or Docker socket.

### Supersedes

This clarification supersedes only the statements in
`04-local-only-docker-production-support-clarity.md` that excluded `.env.build`
and user-defined build variables entirely. It does not change Docker as the sole
local engine, removal of native remote builds, the breaking compatibility
boundary, or Debian 12+/Ubuntu 24.04+ production support.

### Required Authoritative Updates

- `01-idea.md` now defines `.env.build` as explicit, committed, non-secret build
  configuration and distinguishes it from excluded runtime and host variables.
- `02-plan.md` now retains `.env.build` parsing, projection, reserved-name
  validation, and tests while continuing to exclude all ambient and sensitive
  environments.
- `03-tasks.md` now requires preservation and validation of the `.env.build`
  boundary instead of its removal.


## Unified Local Artifact Lifecycle Clarification

### Trigger

Moving native application builds to the local machine exposed two remaining
parts of the production-build model. Native deployment still requires a pushed
commit in a production bare repository solely to compare revision identifiers,
and Docker Compose deployment still exports source from that repository and
builds application images on production.

The local build container is also no longer a production tenant-isolation
boundary. Its purpose is to provide a clean, reproducible Linux `x86_64` build
environment compatible with production. The privileged boundary begins when
BonesRemote accepts deployment input on the production host.

### Decision

All application releases build locally before the deployment transaction.
Native projects produce a complete runtime filesystem artifact. Docker Compose
projects run Compose validation, pulls, and builds locally and produce an
artifact containing the committed release tree, a generated BonesDeploy Compose
override, an image inventory, and the exact service images required to run the
release.

Local Git is the sole source-selection boundary. BonesDeploy resolves and
exports the configured branch commit and records that commit as release audit
metadata. Production does not maintain application bare repositories, require a
first Git push, resolve remote branch refs, or compare the artifact revision to
a second repository controlled by the same deploy credential.

Compose builds use the same fixed `linux/amd64` target as native builds. They
use `.env.build` for explicit public build-time interpolation and do not receive
the production `.env` or ambient host variables from BonesDeploy. The project
Compose definition may not depend on production secrets to select services,
build contexts, images, or other build topology. Private dependency and registry
credentials remain unsupported.

Every Compose service image is tagged with an immutable release-specific name
derived from the site, service, and revision. BonesDeploy writes a generated
override that selects those tags. BonesRemote validates and loads the image
archive, materializes the release tree and override, and starts the stack with
builds and pulls disabled. Rollback therefore selects the previous release's
Compose override and already-loaded image tags rather than a mutable shared tag.
Failed-release cleanup and release pruning remove release-specific image tags;
Docker may retain layers still referenced by another release.

Framework build scripts must leave a complete runnable artifact. Dependency
installation, native-extension compilation, and asset compilation are local
build work. Remote prepare is limited to operations that require production
state, such as shared-path wiring, database migrations, production-secret-based
configuration, and runtime validation. Build-only content such as root
`node_modules`, framework caches, and numbered build scripts is removed when the
selected runtime does not require it.

The local Docker runner is a compatibility and reproducibility mechanism, not a
hardened sandbox. It retains the pinned target and image, explicit environment,
secret exclusion, narrow project/cache inputs, timeout, output, and reliable
cleanup. It does not preserve production-oriented user, namespace, or
container-lifecycle complexity that provides no practical local-build value.
BonesDeploy never adds privileged mode, the Docker socket, host home, SSH state,
credential stores, production state, or ambient host variables to a build.

BonesRemote continues to treat every artifact as untrusted privileged input. It
retains framing, digest, resource bounds, path, entry-type, symlink, conflict,
and expanded-size validation before promotion. Manifest fields describe facts
that BonesRemote can enforce; a client-supplied builder-image value is not build
attestation and is not a production trust proof.

BonesInfra remains responsible for production machine policy and runtime
provisioning: operating-system validation, server hardening, deploy and runtime
identities, BonesRemote, constrained sudo, release/shared/configuration paths,
language runtimes, Nginx, systemd, AppArmor, Docker Engine and Compose runtime,
backups, SSL, tunnels, manifests, deletion, and infrastructure patches. It no
longer provisions application source repositories or any application build
facility. The deploy identity remains as the narrow SSH transport principal even
though it no longer serves Git repositories.

### Supersedes

This clarification supersedes mandatory server-side pushed-revision comparison,
production bare repositories, first-push readiness, Compose source deployment,
production Compose builds and pulls, the exclusion of Compose artifacts,
deferred framework artifact pruning, and any definition of prepare that permits
application dependency installation.

It does not remove friendly public commands, production runtime provisioning,
placeholder behavior, strict artifact receipt, release locking, shared-state
wiring, production preparation, sealing, preflight, activation, restart,
rollback, cancellation, pruning, recovery, or operational inspection.

### Required Authoritative Updates

- `01-idea.md` defines one local-build/artifact lifecycle for native and Compose
  runtimes, local Git provenance, complete artifacts, and the stricter prepare
  boundary.
- `02-plan.md` describes native filesystem artifacts, Compose image artifacts,
  immutable image tagging, repository removal, local-runner simplification,
  BonesInfra's runtime-only boundary, and lifecycle state changes.
- `03-tasks.md` adds the implementation, validation, and completion work needed
  to apply this clarification.


## Narrow Remote Configuration Clarification

### Trigger

After application builds moved entirely to local Docker, the BonesDeploy config
sync still serialized the complete local `Runtime` model to BonesRemote. That
model includes framework selection, local build language settings, permission
defaults, and arbitrary framework values that BonesRemote does not consume.

### Decision

BonesDeploy sends a dedicated backend-specific remote runtime descriptor instead
of the local `Runtime` model. Native descriptors contain only the web root and
the optional Ruby version required by the server-side Rails migration script.
Docker descriptors contain only the optional Compose ingress port and Compose
startup timeout. The descriptor retains the release retention count and an
explicit backend discriminator.

Framework template, Node version, permission defaults, Python version, and
arbitrary runtime extras are not synchronized to BonesRemote. Python version is
removed from the remote prepare environment because no server-side prepare
script consumes it; Django uses it only during the local build.

The new descriptor is the only accepted control-plane snapshot format. This
breaking-release branch does not retain deserialization compatibility for the
old broad runtime snapshot. The artifact manifest remains unchanged because its
identity, kind, platform, format, length, digest, and Compose inventory fields
are enforceable transport facts rather than local build configuration.

Nearby control-plane code, tests, direct dependencies, and documentation made
obsolete by narrowing the descriptor are removed as part of the same change.
Provisioning requests sent to BonesInfra and production secrets written to
`shared/.env` remain separate contracts and are not narrowed by this decision.

### Supersedes

This clarification supersedes serializing `RemoteDeploymentConfig.runtime` as
the complete local `Runtime` model and retaining unused runtime values in the
persisted BonesRemote snapshot. It adds detail to the existing breaking-release
decision that previous configuration formats do not constrain this release.

### Required Authoritative Updates

- `01-idea.md` includes the narrow remote control-plane contract in scope and
  preserves the distinction between deployment, provisioning, and secrets.
- `02-plan.md` defines backend-specific descriptor ownership, consumers,
  cleanup, risks, and validation.
- `03-tasks.md` records implementation and validation of the narrowed contract.


## Prepare Build Boundary Clarification

### Trigger

Distribution-runtime changes made Rails and Django install application
dependencies during remote prepare. A Rails deployment compiled native gems on
a small production server and exhausted its resources. This also contradicted
the established complete-artifact contract, which defines dependency
installation and native-extension compilation as local build work.

### Decision

Rails and Django local builds produce complete runnable artifacts. Rails retains
its production `vendor/bundle`; Django retains its production dependency tree
and release-local launchers. Their prepare scripts may validate that required
artifact output exists, but may not invoke Bundler or pip, create a virtual
environment, download application dependencies, or compile native extensions.

Rails and Django prepare continue to own database migrations and other
production-state operations such as Django deployment checks and static-file
collection. Missing or incompatible packaged dependencies fail prepare before
activation without attempting a production build.

### Supersedes

This supersedes the later Ruby and Python distribution-runtime decisions that
moved application dependency installation into remote prepare. It restores and
does not otherwise change the complete-artifact and prepare definitions in
`01-idea.md` and `02-plan.md`.

### Required Authoritative Updates

`01-idea.md` and `02-plan.md` already state the required build and prepare
boundary. `03-tasks.md` records the regression correction and its validation.
