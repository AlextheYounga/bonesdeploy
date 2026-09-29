# Tasks

The completed tasks below record the artifact foundation committed in
`0a52c666`. Clarification `04-local-only-docker-production-support-clarity.md`
supersedes its opt-in Podman and remote-compatibility direction. The pending
breaking-release tasks are the authoritative remaining execution state.

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
wheelhouse work, signing/provenance services, and resumable uploads remain
deliberately deferred.

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

Ignored E2E scenarios were compiled but not executed. Compose artifact
transport, migration support, broader native build-host portability, artifact
pruning, private dependency credentials, signing, and resumable uploads remain
deliberately deferred.
