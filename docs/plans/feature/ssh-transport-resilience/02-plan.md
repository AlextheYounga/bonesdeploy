# Plan

## Current behavior

`crates/bonesdeploy/src/infra/ssh.rs` owns the Rust SSH integration used by
the `bonesdeploy` CLI. `connect` and `connect_privileged` parse configuration
and delegate to `connect_as`, which returns an `openssh::Session` without a
deadline. Command helpers accept `&Session` directly and create `bash -c`
children through that session.

`run_cmd` uses `output`, the stdin helpers write and wait for output, and
`download_cmd` copies remote stdout while a task drains stderr. None of these
public operation boundaries has an overall timeout. `stream_cmd` and
`stream_cmd_with_stdin` drain stdout and stderr in spawned line-reader tasks,
but ignore line-reader errors and discard the results of `tokio::join!`; their
nonzero-exit errors contain only a generic command failure.
`stream_cmd_with_reader` does return reader and join failures, but has no
operation timeout and reports only the command text on a nonzero exit. The
existing `remote_command_failure` helper includes unbounded stdout/stderr and
the command string.

The crate has Tokio time support and its Rust integration tests are under
`crates/bonesdeploy/tests`. The repository convention forbids source test
modules and arbitrary test-only public seams.

## Intended behavior

`infra::ssh` will expose `SshTransport`, which owns an `openssh::Session` and a
`TransportPolicy`. Production constructors (`connect`, `connect_privileged`,
and `connect_as`) use the fixed default policy: a 30-second connection
deadline, a 30-minute command deadline, a 2-hour transfer deadline, and a
64 KiB per-stream diagnostic tail cap. An explicit policy-taking constructor is
part of this focused transport API so crate-root integration tests can use
millisecond deadlines without waiting for production values.

Every current command caller uses `SshTransport`; no command caller passes an
`openssh::Session` to an SSH helper. Each public SSH operation, including child
creation, concurrent stream draining, output copying, and child wait, has the
policy deadline for its class. Run, stream, and stdin command helpers use the
30-minute command deadline. `download_cmd` and `stream_cmd_with_reader` use
the 2-hour transfer deadline for complete download or artifact-transfer
boundaries.

When an operation times out, the child channel is disconnected and the result
identifies a timeout. When local stream I/O, reader execution, child waiting,
or SSH communication fails, the child channel is disconnected and the result
identifies a transport failure. A remote nonzero exit is reported as a
remote-exit failure and includes bounded stdout and stderr tails. The command
string, stdin bytes, artifact bytes, and secrets are excluded from diagnostics.

Streaming helpers continue to print live stdout and stderr, while their shared
draining path also records at most 64 KiB per stream for a later remote-exit
diagnostic. Both streams are drained concurrently by directly joined fallible
futures. Reader failures are returned; spawned reader tasks and task-join
failures are no longer part of the design.

## Approach

Add public `TransportPolicy` and `SshTransport` types inside `infra::ssh`.
`SshTransport` stores the `openssh::Session` and policy together. Keep
`connect`, `connect_privileged`, and `connect_as` as production constructors
that use `TransportPolicy::default()`, and add one explicit constructor that
accepts a `TransportPolicy` for deliberate transport callers and integration
tests. The policy constructor is not a fake-only seam and does not introduce a
generic transport trait.

Refactor every public operation method so the complete child lifecycle is
executed inside the policy's operation timeout and all failure paths
disconnect the child before returning. Keep the existing operation
responsibilities and `anyhow::Result` error boundary while moving their
receiver from `&Session` to `&SshTransport`.

Implement one shared concurrent stream-draining path for streaming commands
with directly joined fallible futures. It will print each received line,
retain bounded stdout/stderr tails, and return both reader results. This
eliminates spawned task-join failure handling rather than adding panic
injection to tests. Non-streaming helpers will use the same policy and bounded
diagnostic formatting for remote nonzero exits.

## Responsibilities and boundaries

`crates/bonesdeploy/src/infra/ssh.rs` owns `SshTransport`,
`TransportPolicy`, timeout application, child cleanup, stream draining, tail
retention, deadline-class selection, and SSH error classification. The existing
command modules remain
orchestration callers and do not gain individual timeout or cleanup logic.

`crates/bonesdeploy/tests/ssh_transport.rs` owns integration fixtures and
assertions at the public SSH transport boundary. The fixture uses an
ephemeral local `sshd` process with deterministic commands that block, close
their stream or transport abruptly, and exit nonzero. Tests construct
`SshTransport` with millisecond-scale values for all three deadline classes,
allowing them to observe bounded completion, cleanup, propagated stream
failures, and bounded diagnostics without waiting for production deadlines and
without exposing private production helpers.

## Affected areas

Expected implementation files:

- `crates/bonesdeploy/src/infra/ssh.rs` for the transport boundary and all SSH helper behavior;
- `crates/bonesdeploy/src/infra/mod.rs` for the transport receiver boundary;
- `crates/bonesdeploy/src/commands/deploy.rs`;
- `crates/bonesdeploy/src/commands/rollback.rs`;
- `crates/bonesdeploy/src/commands/secrets/mod.rs`;
- `crates/bonesdeploy/src/commands/server/doctor.rs`;
- `crates/bonesdeploy/src/commands/site/delete.rs`;
- `crates/bonesdeploy/src/commands/site/doctor.rs`;
- `crates/bonesdeploy/src/commands/site/export.rs`;
- `crates/bonesdeploy/src/commands/site/releases.rs`;
- `crates/bonesdeploy/src/commands/site/setup.rs`;
- `crates/bonesdeploy/src/commands/site/status.rs`;
- `crates/bonesdeploy/src/commands/skill.rs`;
- `crates/bonesdeploy/src/commands/update/release.rs`; and
- `crates/bonesdeploy/tests/ssh_transport.rs` for the ephemeral local `sshd` fixture and public-boundary integration tests.

Planning files updated by this change:

- `docs/plans/feature/ssh-transport-resilience/01-idea.md`;
- `docs/plans/feature/ssh-transport-resilience/02-plan.md`; and
- `docs/plans/feature/ssh-transport-resilience/03-tasks.md`.

No configuration schema, Python SSH runner, remote binary, deployment
transaction code, or e2e package is affected.

## Decisions

The deadlines are fixed product policy rather than `Bones` configuration so a
project cannot disable the safety bound or require configuration migration.

`SshTransport` is the deliberate architecture boundary because it keeps the
session and policy inseparable while giving production and integration callers
the same transport behavior. No generic trait or fake-only production object
is introduced.

The selected deadline wraps the complete public transport method boundary rather
than individual reads or waits, so a sequence of repeatedly progressing I/O
cannot extend execution indefinitely.

Child disconnection is mandatory on timeout and local stream failure because
dropping local futures alone does not establish that the remote process has
stopped using the channel.

The shared stream drain retains only 64 KiB per remote stream. This preserves
useful exit diagnostics while bounding memory and avoiding accidental capture
of local input or deployment artifacts.

Remote exit, timeout, and transport failures use distinct error categories so
callers and CLI output can tell whether the remote command rejected the work,
the connection stopped responding, or the local SSH boundary failed.

Directly joined fallible futures replace spawned reader tasks. This preserves
stream errors without discarding task results and removes reader-task panic
injection from the test contract.

## Risks

Operations exceeding their fixed deadline fail intentionally. The 30-minute
command deadline can terminate an unusually slow legitimate remote command.
The 2-hour transfer deadline is selected to accommodate supported artifact
uploads up to 2 GiB and site exports while still bounding a hung transfer.

Disconnect cleanup can itself fail or hang at the SSH library boundary. The
primary timeout or local stream failure must remain the reported error, with
cleanup treated as best-effort and bounded by the surrounding operation
handling.

Changing streaming from discarded task results to directly joined fallible
futures can expose previously hidden remote or local failures to CLI callers.
This is required to avoid reporting successful transport when output draining
failed.

Tail truncation can omit the beginning of a large diagnostic. The retained
tail is explicitly bounded and must indicate truncation when the cap is
reached.

## Validation

Add integration tests that construct the public policy-taking `SshTransport`
API and invoke its operations against an ephemeral local `sshd`. Use
millisecond-scale connect, command, and transfer deadlines in the test policy.
Assert:

- a connection that does not complete returns a timeout within the short test
  connect deadline rather than hanging;
- a remote command, stream, or stdin command that does not complete returns a
  command timeout within the short test command deadline and disconnects its
  child channel;
- a download or artifact upload that does not complete returns a transfer
  timeout within the short test transfer deadline and disconnects its child
  channel;
- a nonzero remote exit is distinct from timeout and transport errors and
  includes stdout/stderr tails capped at 64 KiB per stream;
- stdout/stderr are drained concurrently, and abrupt stream or SSH transport
  closure is reported as a transport failure;
- stdin and artifact contents do not appear in returned diagnostics; and
- no test waits for production deadlines or injects a reader-task panic.

Run the affected integration tests, the affected crate's ordinary test suite,
`cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features`,
and `shfmt -d .`. Review the final diff to verify that only the SSH transport
boundary, migrated callers, its integration tests, and the three requested
planning documents changed. Do not run e2e tests.
