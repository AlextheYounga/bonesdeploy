# Tasks

## Implementation

- [ ] Define the public `DeployOperations` contract in
  `crates/bonesdeploy/src/commands/deploy.rs` as exactly four `&mut self`
  methods: synchronous `package(&Bones) -> Result<PackagedArtifact>` and
  `handle_production_secrets(&Bones)`, `sync_control_plane(&Bones)`, and
  `upload_artifact(&Bones, &PackagedArtifact)` each returning
  `impl Future<Output = Result<()>> + Send` through stable RPITIT; do not add
  `async-trait`.
- [ ] Implement generic public `DeployWorkflow<O>` ordering so package runs
  first, its `PackagedArtifact` remains alive through all later operations, and
  package failure returns without invoking any later operation.
- [ ] Implement `ProductionDeployOperations` as the CLI's sole adapter,
  delegating package, secret handling, control-plane sync, and artifact upload
  to the existing build, secrets, infra, and SSH functions. Retain the
  deployment `openssh::Session` privately after successful control-plane sync,
  reuse it for upload, and close it after upload; keep secret handling on its
  existing separate sessions without exposing transport types through the
  contract.
- [ ] Preserve existing `openssh::Session` drop behavior when control-plane
  sync fails or the production adapter is dropped before upload; add no
  transaction, rollback, or control-plane promotion semantics.
- [ ] Keep `commands::deploy::run` thin: load configuration, print the
  destination message, construct the production adapter, and invoke the
  workflow.
- [ ] Add crate-root integration fixtures with a `RecordingDeployOperations`
  implementation and an isolated committed source/build setup; do not add
  test-only cfg hooks or production test seams.
- [ ] Add a failure test whose real package operation returns an error before
  producing a `PackagedArtifact`, and prove the recording log and remote-action
  log contain no secret, control-plane, or upload event.
- [ ] Add a success test whose recording package operation calls real
  `build::package`, whose upload operation verifies the supplied temporary
  artifact exists, and whose event log is exactly `package`,
  `handle_production_secrets`, `sync_control_plane`, `upload_artifact`.

## Validation

- [ ] Run the focused `bonesdeploy` integration tests and confirm the failure
  test has no artifact and no later operation, while the success test consumes
  a real temporary artifact and records the exact four-operation order.
- [ ] Run the relevant non-e2e workspace tests and confirm existing build,
  secrets, infrastructure-command, and artifact behavior remains unchanged.
- [ ] Run `cargo clippy` and resolve every warning or error.
- [ ] Run `cargo fmt` and confirm the Rust tree is formatted.
- [ ] Run `shfmt -w .` and confirm shell files remain formatted.
- [ ] Do not execute e2e tests.

## Completion

- [ ] Inspect the final diff for accidental changes to the four-operation
  contract, build internals, transport behavior, automatic first secret push
  semantics, or remote rollback behavior.
- [ ] Update `CONTEXT.md` so its primary deploy-flow ordering starts with local
  packaging, and leave `README.md` unchanged because it has no direct ordering
  claim.
- [ ] Run `git diff --check` and confirm only the requested implementation,
  tests, and necessary documentation correction remain.

## Completion notes

Implementation has not started. This planning record is complete and awaits
human review and approval before implementation.
