# Idea

## Request

Change `bonesdeploy deploy` so it proves that the configured committed
revision can be exported and fully packaged locally before it checks or pushes
production secrets or synchronizes any remote control-plane state. Keep the
completed `PackagedArtifact` available until remote secret handling, control-
plane synchronization, and artifact upload finish.

## Problem

`crates/bonesdeploy/src/commands/deploy.rs` currently checks whether production
secrets exist and may push them before `build::package` resolves the configured
branch, exports the committed tree, and performs the native or Compose build.
When local export or packaging fails, the command can therefore mutate
production secrets before proving that the requested revision is deployable.
The current orchestration also does not provide a deliberate public boundary
for crate-root integration tests to prove the ordering of local and remote
effects.

## Definitions

**Committed revision:** The exact revision resolved from the configured branch
by `build::package`; it is the source exported and packaged for this deploy.

**Local packaging success:** Successful completion of `build::package`,
including committed-revision resolution and export, the configured native or
Compose build, and creation of the uploadable `PackagedArtifact`.

**Production secret handling:** The remote existence check performed by
`production_secrets_exist` and, when the remote file is absent, the existing
automatic `secrets::push` behavior. This change does not alter whether the
first missing secret file is pushed automatically.

**Remote control-plane action:** Any SSH-backed production operation after
local packaging, including production secret handling, control-plane
configuration synchronization, and artifact deployment upload.

**`DeployOperations`:** The focused public Rust trait in `commands::deploy`
with exactly four `&mut self` methods. `package` is synchronous and returns
`Result<PackagedArtifact>`. `handle_production_secrets`, `sync_control_plane`,
and `upload_artifact` return `impl Future<Output = Result<()>> + Send` using
stable return-position `impl Trait` in traits (RPITIT), with no new
`async-trait` dependency. The production implementation delegates those
methods to the current `build`, `secrets`, `infra`, and `ssh` functions. A
recording implementation is used only by crate-root integration tests.

**Production SSH session:** `ProductionDeployOperations` stores the
deployment `openssh::Session` privately after successful
`sync_control_plane`. `upload_artifact` reuses that session and closes it after
upload, matching the current shared-session behavior. Production secret
handling keeps its current separate SSH sessions. If control-plane sync fails
or the adapter is dropped before upload, existing `openssh::Session` drop
behavior handles the session; this change adds no transaction or rollback
semantics.

**`DeployWorkflow<O>`:** The public generic orchestration boundary that owns
the deploy order and the lifetime of the `PackagedArtifact` while invoking a
`DeployOperations` implementation. It is a focused workflow abstraction, not
a generalized transport, build, or dependency-injection framework.

## Desired outcome

After configuration is loaded and the deploy destination message is printed,
`build::package` is the first operation in the deploy workflow. If committed
revision export or packaging fails, the command returns the build error without
opening an SSH session or performing a secret, control-plane, or artifact
operation. When packaging succeeds, the workflow checks and performs the
existing production secret behavior, synchronizes the control plane, and then
uploads the retained artifact in that order.

## Scope

- Reorder `bonesdeploy deploy` orchestration around local packaging as the
  preflight boundary.
- Retain the resulting `PackagedArtifact` through remote secret handling,
  control-plane synchronization, and artifact upload.
- Add the focused public `DeployOperations` contract and generic
  `DeployWorkflow<O>` boundary required for crate-root integration tests,
  following `docs/conventions/rust.md`.
- Add integration coverage for build/export failure isolation and successful
  secrets-before-config-sync-before-artifact-upload ordering.
- Correct `CONTEXT.md` because its documented deploy ordering currently starts
  with remote control-plane synchronization; no README change is required
  because it does not document this ordering.

## Constraints

- `build::package` remains the existing owner of Git resolution/export and
  native/Compose build behavior; local build internals are unchanged.
- The CLI continues to load configuration and print its deploy message before
  entering the orchestration boundary.
- Existing secret-check, automatic first secret push, control-plane sync, and
  artifact-upload operations retain their behavior apart from their ordering.
- `DeployOperations` contains only package, production-secret handling,
  control-plane synchronization, and artifact upload; it is not a general
  dependency-injection framework.
- The production adapter is the only CLI implementation and delegates to the
  current build, secrets, infrastructure, and SSH functions.
- Rust tests belong in the `crates/bonesdeploy/tests/` crate-root integration
  test directory; production source files must not gain test-only seams.
- No e2e tests may be run for this change.

## Exclusions

- Transport timeout changes.
- Transactional control-plane promotion.
- Changes to automatic first secret push behavior.
- Remote rollback behavior.
- Changes to local Git export, native build, Compose build, or artifact
  packaging internals.
