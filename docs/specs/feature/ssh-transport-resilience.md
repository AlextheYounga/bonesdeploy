# SSH Transport Resilience

<!-- Merged from feature/ssh-transport-resilience -->

## Idea

### Request

Make the Rust SSH transport resilient to indefinitely hanging connections and
remote operations. Ensure streaming helpers preserve read failures and expose
useful remote exit diagnostics without logging stdin, artifacts, or secrets.
Establish a deliberate `SshTransport` boundary that owns the SSH session and
transport policy, and make the behavior concretely testable through public
transport APIs.

### Problem

`crates/bonesdeploy/src/infra/ssh.rs` currently has no deadline on `connect_as`
or on the public command and transfer helpers. A stalled SSH connection,
remote process, upload, download, or local stream can therefore leave a CLI
command waiting indefinitely. `stream_cmd` and `stream_cmd_with_stdin` discard
line-reader failures and spawned-task join failures, while their nonzero-exit
errors omit remote stdout and stderr.

The other helpers either wait without a deadline or report failures without a
consistent distinction between a transport failure, a timeout, and a remote
command that exited unsuccessfully. The current direct `Session` parameters
also leave policy ownership and test control outside the SSH boundary.

### Definitions

**SSH transport policy:** A public value object containing the connect deadline,
command deadline, transfer deadline, remote-output tail limit, and command
output limit used by
`SshTransport`. Production constructors use its fixed default values. An
explicit policy is available to meaningful transport callers, including
crate-root integration tests that need short deadlines. The policy is not
project configuration.

**`SshTransport`:** The public Rust SSH boundary that owns an
`openssh::Session` and its `TransportPolicy`. Its constructors establish the
session, and its methods perform remote command and transfer operations.
Callers pass this boundary rather than an `openssh::Session` directly.

**SSH operation:** Any public `SshTransport` method that spawns, writes to,
reads from, drains, waits for, or transfers data through a remote SSH child
channel. It includes command execution, streaming, upload, and download.

**Remote output tail:** The bounded suffix retained from remote stdout or
stderr for diagnostics. It contains remote process output only; it never
contains the command string, local stdin, or artifact bytes.

**Transport failure:** A failure in SSH channel creation, local stream I/O,
reader execution, child waiting, or session communication that is not a
remote nonzero exit and is not a deadline expiry.

**Timeout:** Expiration of the fixed SSH connect deadline or the applicable
command or transfer deadline before that public boundary completes. A timeout
is reported separately from a transport failure and a nonzero remote exit.

### Desired outcome

Every public Rust SSH connection and operation boundary completes with a
bounded result. `SshTransport` keeps the session and policy together. A
timed-out or locally failed child channel is disconnected, streaming reader
failures are returned, and concurrent stdout/stderr draining retains bounded
diagnostic tails. Errors distinguish timeouts, transport failures, and
nonzero remote exits; nonzero exits include bounded remote stdout/stderr tails
when present.

### Scope

This change includes:

- public `TransportPolicy` and `SshTransport` types in `infra::ssh`;
- production constructors using the fixed default policy and an explicit
  policy-taking transport constructor;
- applying the policy to connection, command, streaming, upload, and download
  operations;
- directly joined concurrent stdout/stderr draining with bounded tails;
- cleanup of child channels after timeouts and local stream failures;
- distinct error reporting with bounded remote exit diagnostics;
- migration of all current direct `Session` callers to `SshTransport`; and
- integration-level tests through an ephemeral local `sshd` and the public
  transport boundary.

### Constraints

The fixed production connect deadline is 30 seconds. The fixed production
command deadline is 30 minutes for run, stream, and stdin command helpers. The
fixed production transfer deadline is 2 hours for downloads and artifact
uploads through `stream_cmd_with_reader`. Each retained stdout and stderr tail
and each returned successful command stdout value is capped at 64 KiB. A
command exceeding the returned-output limit fails explicitly rather than
returning a partial successful result. These values are product-owned defaults and are not read
from `Bones` or another project configuration source.

Use one absolute `tokio::time::Instant` deadline at each public transport
boundary. Disconnect the remote child channel on timeout and on local stream
failure with independently bounded cleanup. Drain stdout and
stderr concurrently with directly joined fallible futures so one full stream
cannot block the other and reader-task join failures do not exist. Preserve
the existing public helper responsibilities and `anyhow::Result` boundary.

Tests belong in the owning crate's `tests/` directory and must exercise public
transport behavior or an observable CLI/process boundary. Integration tests
use millisecond explicit policy values for all three deadline classes and must
not wait for production deadlines. Do not add `#[cfg(test)]` modules,
arbitrary public test seams, or e2e tests.

### Exclusions

Do not introduce a broad generic transport trait or a fake-only production API.
Do not change deployment transaction semantics, remote cancellation semantics,
the Python provisioning SSH runner, SSH host-key policy, command construction,
or project configuration. Do not log command strings, stdin, uploaded
artifacts, or secrets. Session close after a successful operation remains
outside this change and is not converted into a new success criterion.

Operations that exceed their fixed deadline intentionally fail. The 2-hour
transfer deadline is selected to accommodate supported artifact uploads up to
2 GiB and site exports while still bounding a hung transfer.


## Plan

### Current behavior

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

### Intended behavior

`infra::ssh` will expose `SshTransport`, which owns an `openssh::Session` and a
`TransportPolicy`. Production constructors (`connect`, `connect_privileged`,
and `connect_as`) use the fixed default policy: a 30-second connection
deadline, a 30-minute command deadline, a 2-hour transfer deadline, a 64 KiB
per-stream diagnostic tail cap, and a 64 KiB returned-command-output cap. An explicit policy-taking constructor is
part of this focused transport API so crate-root integration tests can use
millisecond deadlines without waiting for production values.

Every current command caller uses `SshTransport`; no command caller passes an
`openssh::Session` to an SSH helper. Each public SSH operation establishes one
absolute deadline before child creation; it covers concurrent stream draining,
output copying, and child wait. Run, stream, and stdin command helpers use the
30-minute command deadline. `download_cmd` and `stream_cmd_with_reader` use
the 2-hour transfer deadline for complete download or artifact-transfer
boundaries.

When an operation times out, the child channel is disconnected with separately
bounded cleanup and the result
identifies a timeout. When local stream I/O, reader execution, child waiting,
or SSH communication fails, the child channel is disconnected and the result
identifies a transport failure. A remote nonzero exit is reported as a
remote-exit failure and includes bounded stdout and stderr tails. The command
string, stdin bytes, artifact bytes, and secrets are excluded from diagnostics.

Streaming helpers continue to print live stdout and stderr, while non-streaming
helpers retain output without printing it. Their shared
draining path also records at most 64 KiB per stream for a later remote-exit
diagnostic. Both streams are drained concurrently by directly joined fallible
futures. Reader failures are returned; spawned reader tasks and task-join
failures are no longer part of the design.

### Approach

Add public `TransportPolicy` and `SshTransport` types inside `infra::ssh`.
`SshTransport` stores the `openssh::Session` and policy together. Keep
`connect`, `connect_privileged`, and `connect_as` as production constructors
that use `TransportPolicy::default()`, and add one explicit constructor that
accepts a `TransportPolicy` for deliberate transport callers and integration
tests. The policy constructor is not a fake-only seam and does not introduce a
generic transport trait.

Refactor every public operation method so one absolute policy deadline covers
the complete child lifecycle. All failure paths attempt child disconnect through
a separate bounded cleanup deadline before returning. Keep the existing operation
responsibilities and `anyhow::Result` error boundary while moving their
receiver from `&Session` to `&SshTransport`.

Implement one shared concurrent stream-draining path for streaming commands
with directly joined fallible futures. It will print each received line,
retain bounded stdout/stderr tails, and return both reader results. This
eliminates spawned task-join failure handling rather than adding panic
injection to tests. Non-streaming helpers retain bounded command stdout and
diagnostic tails without printing remote output. Command output exceeding 64 KiB
fails explicitly rather than returning partial output.

### Responsibilities and boundaries

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

### Affected areas

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

### Decisions

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

### Risks

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

### Validation

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


## Tasks

### Implementation

- [x] Define public `TransportPolicy` and `SshTransport` types in `infra::ssh`; completion is that `SshTransport` owns both the `openssh::Session` and policy, production constructors use the fixed 30-second connect/30-minute command/2-hour transfer/64 KiB defaults, and one explicit policy-taking constructor is available as meaningful transport API.
- [x] Change every SSH operation helper to use `SshTransport` rather than a direct `Session`; completion is that the transport policy is available at every connection, command, upload, download, and streaming boundary without a generic trait or fake-only production API.
- [x] Apply the 30-second connect, 30-minute command, and 2-hour transfer deadlines through `SshTransport`; completion is that child creation, command execution, transfer, draining, and wait are covered by the relevant class deadline and all production callers use the fixed default policy.
- [x] Centralize concurrent stdout/stderr draining with directly joined fallible futures, bounded tails, and live line output; completion is that both streams are drained concurrently, tails are capped, reader failures are returned, and no spawned reader-task join failure path remains.
- [x] Make all timeout and local stream-failure paths disconnect the child channel while preserving the primary failure; completion is that no helper returns from those paths without attempting child cleanup.
- [x] Classify timeout, transport, and nonzero remote-exit failures distinctly; completion is that each public helper returns the correct category and remote exits include bounded stdout/stderr diagnostics without command or input content.
- [x] Migrate `infra/mod.rs` and all listed command callers from direct `Session` values to `SshTransport`; completion is that no current Rust SSH caller passes an `openssh::Session` to an SSH helper.
- [x] Add `crates/bonesdeploy/tests/ssh_transport.rs` with its own ephemeral local `sshd` fixture and millisecond policy values for connect, command, and transfer classes; completion is deterministic coverage of blocked connect, blocked command, blocked transfer, nonzero exit, abrupt stream or transport closure, concurrent draining, cleanup, and diagnostic redaction without waiting for production deadlines or injecting a reader-task panic.

### Validation

- [x] Run the focused SSH transport integration test target; completion is evidence that short-policy blocked connect, command, and transfer boundaries return within their configured deadlines and all required error distinctions, stream-failure propagation, and cleanup observations pass.
- [x] Run the affected `bonesdeploy` crate test suite; completion is that existing SSH helper, CLI, and infrastructure tests remain green after caller migration.
- [x] Run `cargo fmt --check`; completion is no formatting diff.
- [x] Run `cargo clippy --workspace --all-targets --all-features`; completion is no warning or error introduced by the change.
- [x] Run `shfmt -d .`; completion is no shell-formatting diff.
- [x] Review the final diff and changed-file list; completion is that implementation is limited to the focused SSH transport boundary, listed callers, and integration tests, with no e2e execution and no deployment transaction changes.

### Completion

- [x] Confirm diagnostics retain only bounded remote stdout/stderr tails and never log command text, stdin, artifact bytes, or secrets.
- [x] Confirm the three authoritative planning documents match the implemented transport API, caller migration, test fixture, and validation evidence.

### Completion notes

Implemented `SshTransport` with fixed production deadlines and explicit policy
construction, then migrated all Rust SSH callers to its methods. Review
clarification `04-command-output-and-cleanup-clarity.md` added a 64 KiB returned
command-output limit, one absolute deadline across each child lifecycle, and a
five-second independent cleanup bound. The public integration fixture starts an
isolated local `sshd`; it covers stalled handshakes, command and transfer
timeouts, reusable transport cleanup, abrupt closure, exact command-output
limits, bounded diagnostics and redaction, streaming, and local reader
failures. `cargo test -p bonesdeploy --test ssh_transport`, `cargo fmt --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`, and
`shfmt -d .` passed. E2E tests were not run. No unfinished work remains.


## Command Output And Cleanup Clarity

### Trigger

Implementation review found that retaining all successful `run_cmd` output
could allocate without bound, and that cleanup and deadline handling did not
yet cover the complete child lifecycle as one operation boundary.

### Decision

`TransportPolicy` includes a public command-output limit. The fixed production
limit is 64 KiB, matching the existing per-stream remote diagnostic-tail limit.
`run_cmd` and `run_cmd_with_stdin_output` retain and return complete stdout only
when it is at or below this limit. Once stdout exceeds the limit, they fail with
an explicit SSH transport error and never return a partial successful result.

Each operation establishes one absolute deadline before spawning its child. The
same deadline applies to spawn, local I/O, concurrent draining, and child wait.
Failure cleanup calls `child.disconnect()` under a separate fixed five-second
cleanup deadline so cleanup cannot hide or extend the primary result
indefinitely. When `openssh::Child::wait` itself owns the child at deadline
expiry, dropping that timed-out wait future drops the child, which the openssh
API documents as terminating the local SSH process for the remote channel; no
additional child handle exists for `disconnect()`.

Only streaming helpers print remote output live. Non-streaming helpers retain
stdout and stderr solely for return values and bounded diagnostics.

### Supersedes

This clarifies the earlier implicit preservation of unbounded successful command
stdout and strengthens the plan's complete-boundary cleanup requirement with
the ownership behavior of `openssh::Child::wait`.

### Required Authoritative Updates

- `01-idea.md`
- `02-plan.md`
- `03-tasks.md`
