# Tasks

## Implementation

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

## Validation

- [x] Run the focused `bonesdeploy` integration tests and confirm the failure
  test has no artifact and no later operation, while the success test consumes
  a real temporary artifact and records the exact four-operation order.
- [x] Run the relevant non-e2e workspace tests and confirm existing build,
  secrets, infrastructure-command, and artifact behavior remains unchanged.
- [x] Run `cargo clippy` and resolve every warning or error.
- [x] Run `cargo fmt` and confirm the Rust tree is formatted.
- [x] Run `shfmt -w .` and confirm shell files remain formatted.
- [x] Do not execute e2e tests.

## Completion

- [x] Inspect the final diff for accidental changes to the four-operation
  contract, build internals, transport behavior, automatic first secret push
  semantics, or remote rollback behavior.
- [x] Update `CONTEXT.md` so its primary deploy-flow ordering starts with local
  packaging, and leave `README.md` unchanged because it has no direct ordering
  claim.
- [x] Run `git diff --check` and confirm only the requested implementation,
  tests, and necessary documentation correction remain.

## Completion notes

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
