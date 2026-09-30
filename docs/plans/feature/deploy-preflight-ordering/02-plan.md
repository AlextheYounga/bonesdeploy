# Plan

## Current Behavior

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

## Intended Behavior

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

## Approach

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

## Responsibilities and Boundaries

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

## Affected Areas

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

## Decisions

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

## Risks

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

## Validation

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
