# Deploy Preflight Ordering

<!-- Merged from feature/deploy-preflight-ordering -->

## Idea

### Request

Change `bonesdeploy deploy` so it proves that the configured committed
revision can be exported and fully packaged locally before it checks or pushes
production secrets or synchronizes any remote control-plane state. Keep the
completed `PackagedArtifact` available until remote secret handling, control-
plane synchronization, and artifact upload finish.

### Problem

`crates/bonesdeploy/src/commands/deploy.rs` currently checks whether production
secrets exist and may push them before `build::package` resolves the configured
branch, exports the committed tree, and performs the native or Compose build.
When local export or packaging fails, the command can therefore mutate
production secrets before proving that the requested revision is deployable.
The current orchestration also does not provide a deliberate public boundary
for crate-root integration tests to prove the ordering of local and remote
effects.

### Definitions

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

### Desired outcome

After configuration is loaded and the deploy destination message is printed,
`build::package` is the first operation in the deploy workflow. If committed
revision export or packaging fails, the command returns the build error without
opening an SSH session or performing a secret, control-plane, or artifact
operation. When packaging succeeds, the workflow checks and performs the
existing production secret behavior, synchronizes the control plane, and then
uploads the retained artifact in that order.

### Scope

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

### Constraints

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

### Exclusions

- Transport timeout changes.
- Transactional control-plane promotion.
- Changes to automatic first secret push behavior.
- Remote rollback behavior.
- Changes to local Git export, native build, Compose build, or artifact
  packaging internals.


## Plan

### Current Behavior

`commands::deploy::run` loads `.env`, prints the destination, checks remote
production secret presence, and automatically calls `secrets::push` when the
remote file is missing. It then calls `build::package`, which resolves the
configured branch, exports the committed revision, runs the selected native or
Compose build, and returns a `PackagedArtifact`. Finally, `deploy_artifact`
encodes the manifest, opens SSH, synchronizes `RemoteDeploymentConfig`, opens
the artifact, and streams it to `bonesremote deploy`.

`PackagedArtifact` owns a temporary artifact file and its manifest, so keeping
the value alive keeps the uploadable file available. `production_secrets_exist`
and `secrets::push` each perform their own SSH-backed remote operation.
`infra::sync_control_plane` and the artifact upload share the deployment SSH
session and currently occur in that order.

The crate exposes the `commands` module through the library target and already
uses `crates/bonesdeploy/tests/` for integration tests. The Rust convention
requires tests to use deliberate public APIs and forbids test-only modules or
seams in production source files.

### Intended Behavior

After `run` loads configuration and prints the deploy destination, it constructs
the production operations adapter and delegates to `DeployWorkflow`.
`DeployWorkflow` calls the adapter's package operation first and retains the
returned `PackagedArtifact`.

Only after local packaging succeeds does it check production secret presence
and perform the existing automatic first push when the remote secret is
missing. It then opens the deployment session, synchronizes the control-plane
configuration, and uploads the retained artifact. A local export or packaging
error returns before any SSH connection or remote action. A successful deploy
therefore observes production secret handling before control-plane sync, and
control-plane sync before artifact upload.

### Approach

Keep configuration loading and the user-facing destination message in
`commands::deploy::run`. Define the focused public `DeployOperations` trait in
`commands::deploy` with exactly these `&mut self` methods:

- synchronous `package(&Bones) -> Result<PackagedArtifact>`;
- `handle_production_secrets(&Bones) -> impl Future<Output = Result<()>> +
  Send`;
- `sync_control_plane(&Bones) -> impl Future<Output = Result<()>> + Send`; and
- `upload_artifact(&Bones, &PackagedArtifact) -> impl Future<Output =
  Result<()>> + Send`.

Use stable Rust RPITIT for the three remote methods; do not add
`async-trait`. Define `DeployWorkflow<O>` over that contract. Its one workflow
method calls package, then `handle_production_secrets`, then
`sync_control_plane`, then `upload_artifact`, retaining the package value
across all later calls. The workflow returns immediately from a package error,
so no later operation receives a fabricated or absent artifact.

Add `ProductionDeployOperations` as the sole CLI adapter. Its package method
delegates to `build::package`; its secret method delegates to
`production_secrets_exist` and the existing automatic `secrets::push` path;
its control-plane method opens the deployment SSH session, delegates
configuration synchronization to `infra::sync_control_plane`, and stores the
successful `openssh::Session` privately. Its upload method takes that stored
session, delegates framed artifact streaming to the existing SSH operation,
then closes the session after upload as the current implementation does. A
sync error or adapter drop relies on existing `openssh::Session` drop behavior
and introduces no transaction, rollback, or promotion semantics. Its secret
method retains the current separate sessions used by
`production_secrets_exist` and `secrets::push`. The adapter does not expose a
transport session through `DeployOperations`; the workflow owns only ordering
and artifact lifetime.

Add crate-root integration tests with a `RecordingDeployOperations`
implementation of the same contract. The failure fixture makes the real
package operation fail during committed export or packaging; it returns
`Err` and no `PackagedArtifact`, and the test asserts that no later recording
event or remote action occurs. The success fixture calls real `build::package`
against an isolated committed source, records each operation, and in its
upload operation asserts that the supplied artifact path exists and is the
real temporary packaged artifact before recording completion. The resulting
event list must be exactly `package`, `handle_production_secrets`,
`sync_control_plane`, `upload_artifact`.

### Responsibilities and Boundaries

- `commands::deploy::run` loads `.env`, prints the destination, constructs
  `ProductionDeployOperations`, and delegates; it does not own sequencing.
- `DeployOperations` names exactly the four `&mut self` methods and effects
  required by the workflow; it exposes no transport session, build runner, or
  secret payload.
- `DeployWorkflow<O>` owns operation ordering and the lifetime of the
  `PackagedArtifact`.
- `ProductionDeployOperations` is the production adapter and delegates to
  current build, secret, infrastructure, and SSH functions while privately
  retaining the deployment session between control-plane sync and upload.
- `build::package` remains responsible for committed revision resolution,
  source export, native/Compose builds, and artifact creation.
- `commands::secrets` retains production secret existence checking and the
  existing automatic first push behavior.
- `deploy_artifact` and `infra` retain control-plane synchronization and
  framed artifact upload behavior.
- `crates/bonesdeploy/tests/` owns the recording implementation, isolated
  fixtures, and assertions about observable external effects; it does not
  duplicate workflow logic or add production test hooks.

### Affected Areas

- `crates/bonesdeploy/src/commands/deploy.rs`: define `DeployOperations`,
  `DeployWorkflow<O>`, and `ProductionDeployOperations`; keep `run` thin and
  route the CLI through the production adapter.
- `crates/bonesdeploy/tests/`: add focused integration coverage for preflight
  failure isolation and successful remote-operation ordering.
- `CONTEXT.md`: update the primary deploy flow because its current statement
  that deployment starts with remote config synchronization does not describe
  the local-package-first workflow.
- `README.md`: no change, because the current README describes local artifact
  builds and remote receipt but does not state this operation ordering.

### Decisions

- Local packaging is the first post-message deploy operation because it is the
  only step that proves the configured committed revision can be exported and
  fully packaged before production mutation.
- `DeployOperations` is the only testable seam because it names the four real
  workflow effects and avoids test-only cfg hooks or a broad DI framework.
- `DeployWorkflow<O>` is generic over the focused contract so integration tests
  can provide a recording implementation while the CLI uses the production
  adapter.
- `PackagedArtifact` remains in scope until upload completes because its
  temporary file is the source of the streamed deployment payload.
- Secret handling remains before control-plane synchronization, and
  synchronization remains before artifact upload, preserving the required
  successful-deploy order and the current remote command responsibilities.
- The automatic first secret push is preserved exactly; only its position after
  successful local packaging changes.

### Risks

- Moving packaging ahead of the secret check can expose local build failures
  earlier and change when users see the remote secret error; this is intended,
  and focused failure tests must prove no remote action precedes that error.
- Dropping or shadowing the artifact before the asynchronous upload would cause
  a missing temporary file; the success test and code review must verify the
  artifact remains owned by the orchestration scope through upload completion.
- A contract with hidden transport or build methods would broaden the seam and
  make its responsibility unclear; the four-operation contract must remain
  exact and the adapter must contain production delegation details.
- A remote method that consumes or replaces the stored session would regress
  current connection reuse; the adapter must retain the session after a
  successful sync and close it only after upload, while relying on normal
  session drop behavior for sync failure or adapter drop.
- Adding an `async-trait` dependency would broaden the change without solving a
  current requirement; stable RPITIT is the settled contract mechanism for the
  three remote methods.
- A recording test that manufactures an artifact could pass while upload
  lifetime is broken; the success test must call real packaging and verify the
  temporary artifact exists when upload receives it.
- Updating the sequence without correcting `CONTEXT.md` would leave the
  documented deploy flow stale; the final diff must check that claim.

### Validation

- The build/export failure integration test returns the local error and records
  no SSH connection, production secret check/push, control-plane sync, or
  artifact upload.
- The successful integration test records the exact four-event order and
  proves artifact upload receives a real temporary package produced by the
  package operation.
- Existing focused `bonesdeploy` tests and the relevant non-e2e workspace tests
  pass without changing build, secret, transport, or remote lifecycle behavior.
- `cargo clippy`, `cargo fmt`, and `shfmt -w .` complete without warnings or
  formatting errors; e2e tests are not run.
- The final diff contains only the intended implementation, integration tests,
  and any necessary correction to the documented deploy ordering.


## Tasks

### Implementation

- [x] Define the public `DeployOperations` contract in
  `crates/bonesdeploy/src/commands/deploy.rs` as exactly four `&mut self`
  methods: synchronous `package(&Bones) -> Result<PackagedArtifact>` and
  `handle_production_secrets(&Bones)`, `sync_control_plane(&Bones)`, and
  `upload_artifact(&Bones, &PackagedArtifact)` each returning
  `impl Future<Output = Result<()>> + Send` through stable RPITIT; do not add
  `async-trait`.
- [x] Implement generic public `DeployWorkflow<O>` ordering so package runs
  first, its `PackagedArtifact` remains alive through all later operations, and
  package failure returns without invoking any later operation.
- [x] Implement `ProductionDeployOperations` as the CLI's sole adapter,
  delegating package, secret handling, control-plane sync, and artifact upload
  to the existing build, secrets, infra, and SSH functions. Retain the
  deployment `openssh::Session` privately after successful control-plane sync,
  reuse it for upload, and close it after upload; keep secret handling on its
  existing separate sessions without exposing transport types through the
  contract.
- [x] Preserve existing `openssh::Session` drop behavior when control-plane
  sync fails or the production adapter is dropped before upload; add no
  transaction, rollback, or control-plane promotion semantics.
- [x] Keep `commands::deploy::run` thin: load configuration, print the
  destination message, construct the production adapter, and invoke the
  workflow.
- [x] Add crate-root integration fixtures with a `RecordingDeployOperations`
  implementation and an isolated committed source/build setup; do not add
  test-only cfg hooks or production test seams.
- [x] Add a failure test whose real package operation returns an error before
  producing a `PackagedArtifact`, and prove the recording log and remote-action
  log contain no secret, control-plane, or upload event.
- [x] Add a success test whose recording package operation calls real
  `build::package`, whose upload operation verifies the supplied temporary
  artifact exists, and whose event log is exactly `package`,
  `handle_production_secrets`, `sync_control_plane`, `upload_artifact`.

### Validation

- [x] Run the focused `bonesdeploy` integration tests and confirm the failure
  test has no artifact and no later operation, while the success test consumes
  a real temporary artifact and records the exact four-operation order.
- [x] Run the relevant non-e2e workspace tests and confirm existing build,
  secrets, infrastructure-command, and artifact behavior remains unchanged.
- [x] Run `cargo clippy` and resolve every warning or error.
- [x] Run `cargo fmt` and confirm the Rust tree is formatted.
- [x] Run `shfmt -w .` and confirm shell files remain formatted.
- [x] Do not execute e2e tests.

### Completion

- [x] Inspect the final diff for accidental changes to the four-operation
  contract, build internals, transport behavior, automatic first secret push
  semantics, or remote rollback behavior.
- [x] Update `CONTEXT.md` so its primary deploy-flow ordering starts with local
  packaging, and leave `README.md` unchanged because it has no direct ordering
  claim.
- [x] Run `git diff --check` and confirm only the requested implementation,
  tests, and necessary documentation correction remain.

### Completion notes

Implemented the four-operation deploy workflow and production adapter. Added
crate-root integration tests that run real packaging in isolated committed Git
repositories, covering package-failure isolation and successful operation order
with a live temporary artifact. Validation completion is recorded after checks
finish.

Validation completed successfully: `cargo test -p bonesdeploy` (including the
focused workflow integration target), `cargo clippy`, `cargo fmt`, `shfmt -w .`,
and `git diff --check`. No e2e tests were executed. Final inspection found only
the requested deploy workflow, integration coverage, documentation correction,
and this task record.
