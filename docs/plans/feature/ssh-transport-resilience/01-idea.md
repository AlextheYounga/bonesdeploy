# Idea

## Request

Make the Rust SSH transport resilient to indefinitely hanging connections and
remote operations. Ensure streaming helpers preserve read failures and expose
useful remote exit diagnostics without logging stdin, artifacts, or secrets.
Establish a deliberate `SshTransport` boundary that owns the SSH session and
transport policy, and make the behavior concretely testable through public
transport APIs.

## Problem

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

## Definitions

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

## Desired outcome

Every public Rust SSH connection and operation boundary completes with a
bounded result. `SshTransport` keeps the session and policy together. A
timed-out or locally failed child channel is disconnected, streaming reader
failures are returned, and concurrent stdout/stderr draining retains bounded
diagnostic tails. Errors distinguish timeouts, transport failures, and
nonzero remote exits; nonzero exits include bounded remote stdout/stderr tails
when present.

## Scope

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

## Constraints

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

## Exclusions

Do not introduce a broad generic transport trait or a fake-only production API.
Do not change deployment transaction semantics, remote cancellation semantics,
the Python provisioning SSH runner, SSH host-key policy, command construction,
or project configuration. Do not log command strings, stdin, uploaded
artifacts, or secrets. Session close after a successful operation remains
outside this change and is not converted into a new success criterion.

Operations that exceed their fixed deadline intentionally fail. The 2-hour
transfer deadline is selected to accommodate supported artifact uploads up to
2 GiB and site exports while still bounding a hung transfer.
