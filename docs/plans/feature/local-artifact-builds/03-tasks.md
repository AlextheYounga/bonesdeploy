# Tasks

The completed tasks below record the artifact foundation committed in
`0a52c666` and the Docker-only native follow-up committed through `66ded8e8`.
Clarifications `04` and `05` superseded the original opt-in Podman direction.
Clarification `06-unified-local-artifact-lifecycle-clarity.md` now supersedes
production Git revision comparison, Compose source builds, and deferred
framework pruning. Its pending follow-up is the authoritative remaining
execution state.
Clarification `07-narrow-remote-config-clarity.md` narrows the completed
artifact lifecycle's control-plane sync to values BonesRemote still consumes.

## Narrow Remote Configuration Follow-Up

### Implementation

- [ ] Replace the complete `Runtime` in `RemoteDeploymentConfig` with a tagged
  native or Docker descriptor that carries only backend-specific remote values.
- [ ] Update BonesRemote lifecycle, runtime, status, and doctor consumers to use
  the narrow descriptor while deriving only the minimal internal site config.
- [ ] Remove Python version and arbitrary runtime-extra handling from remote
  prepare and control-plane persistence; retain the Rails Ruby migration input.
- [ ] Remove obsolete tests, imports, and direct dependencies exposed by the
  narrowed control-plane contract, and update architecture documentation where
  it describes config sync.

### Validation

- [ ] Add focused serialization tests proving native and Docker config sync omit
  local build, framework, permission, secret, and unrelated backend values and
  reject the old broad snapshot shape.
- [ ] Run focused core, BonesDeploy, and BonesRemote tests, then all non-E2E Rust
  and Python tests without executing E2E scenarios.
- [ ] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, Python Ruff checks and
  formatting, generated-wheel validation when applicable, and `git diff --check`.

### Completion

- [ ] Review the final diff for stale remote-build configuration, accidental
  provisioning or secret coupling, compatibility code, dead dependencies, and
  documentation drift; record validation evidence and any deliberate remainder.

## Implementation

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

## Validation

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

## Completion

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

## Completion notes

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

## Breaking Local-Only Follow-Up

### Implementation

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

### Validation

- [x] Run focused tests for Docker command safety, fixed public build metadata,
  unconditional artifact deployment, remote-build removal, production OS
  version boundaries, and mode-free provisioning transport.
- [x] Compile but do not execute ignored E2E scenarios covering first deploy,
  second release, and failed-activation rollback across the native framework
  matrix.
- [x] Run all non-E2E Rust and Python tests, Clippy with warnings denied,
  Rustfmt, shfmt, Ruff checks and formatting, generated-wheel validation, and
  `git diff --check`; resolve every warning or failure.

### Completion

- [x] Review the final diff for remaining Podman, remote native build,
  build-mode, accidental `.env.build` secret semantics, unsupported
  production-host, secret exposure, broad Docker access, and stale compatibility
  behavior.
- [x] Record implementation deviations, exact validation evidence, and any
  deliberately deferred Compose, migration, build-host portability, artifact
  pruning, credential, signing, or resumable-upload work.

## Breaking Follow-Up Completion Notes

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

## Unified Local Artifact Lifecycle Follow-Up

### Implementation

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

### Validation

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

### Completion

- [x] Review the final diff for any production application checkout, dependency
  installation, image pull/build, mutable Compose image tag, build-only artifact
  content, repository provisioning, first-push guidance, unverifiable provenance
  claim, accidental secret input, unsafe artifact receipt, or stale lifecycle
  terminology.
- [x] Record exact artifact-size limits, framework pruning outcomes, Compose image
  rollback/pruning evidence, implementation deviations, validation results, and
  deliberately deferred migration, registry, credential, signing, SBOM, and
  resumable-upload work.

## Unified Follow-Up Completion Notes

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
