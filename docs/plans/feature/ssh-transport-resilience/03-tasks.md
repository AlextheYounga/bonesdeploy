# Tasks

## Implementation

- [ ] Define public `TransportPolicy` and `SshTransport` types in `infra::ssh`; completion is that `SshTransport` owns both the `openssh::Session` and policy, production constructors use the fixed 30-second connect/30-minute command/2-hour transfer/64 KiB defaults, and one explicit policy-taking constructor is available as meaningful transport API.
- [ ] Change every SSH operation helper to use `SshTransport` rather than a direct `Session`; completion is that the transport policy is available at every connection, command, upload, download, and streaming boundary without a generic trait or fake-only production API.
- [ ] Apply the 30-second connect, 30-minute command, and 2-hour transfer deadlines through `SshTransport`; completion is that child creation, command execution, transfer, draining, and wait are covered by the relevant class deadline and all production callers use the fixed default policy.
- [ ] Centralize concurrent stdout/stderr draining with directly joined fallible futures, bounded tails, and live line output; completion is that both streams are drained concurrently, tails are capped, reader failures are returned, and no spawned reader-task join failure path remains.
- [ ] Make all timeout and local stream-failure paths disconnect the child channel while preserving the primary failure; completion is that no helper returns from those paths without attempting child cleanup.
- [ ] Classify timeout, transport, and nonzero remote-exit failures distinctly; completion is that each public helper returns the correct category and remote exits include bounded stdout/stderr diagnostics without command or input content.
- [ ] Migrate `infra/mod.rs` and all listed command callers from direct `Session` values to `SshTransport`; completion is that no current Rust SSH caller passes an `openssh::Session` to an SSH helper.
- [ ] Add `crates/bonesdeploy/tests/ssh_transport.rs` with its own ephemeral local `sshd` fixture and millisecond policy values for connect, command, and transfer classes; completion is deterministic coverage of blocked connect, blocked command, blocked transfer, nonzero exit, abrupt stream or transport closure, concurrent draining, cleanup, and diagnostic redaction without waiting for production deadlines or injecting a reader-task panic.

## Validation

- [ ] Run the focused SSH transport integration test target; completion is evidence that short-policy blocked connect, command, and transfer boundaries return within their configured deadlines and all required error distinctions, stream-failure propagation, and cleanup observations pass.
- [ ] Run the affected `bonesdeploy` crate test suite; completion is that existing SSH helper, CLI, and infrastructure tests remain green after caller migration.
- [ ] Run `cargo fmt --check`; completion is no formatting diff.
- [ ] Run `cargo clippy --workspace --all-targets --all-features`; completion is no warning or error introduced by the change.
- [ ] Run `shfmt -d .`; completion is no shell-formatting diff.
- [ ] Review the final diff and changed-file list; completion is that implementation is limited to the focused SSH transport boundary, listed callers, and integration tests, with no e2e execution and no deployment transaction changes.

## Completion

- [ ] Confirm diagnostics retain only bounded remote stdout/stderr tails and never log command text, stdin, artifact bytes, or secrets.
- [ ] Confirm the three authoritative planning documents match the implemented transport API, caller migration, test fixture, and validation evidence.

## Completion notes

Implementation and validation have not started. Completion notes will record
only material deviations, validation evidence, discoveries, and deliberately
unfinished work after implementation.
