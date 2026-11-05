# Command Output And Cleanup Clarity

## Trigger

Implementation review found that retaining all successful `run_cmd` output
could allocate without bound, and that cleanup and deadline handling did not
yet cover the complete child lifecycle as one operation boundary.

## Decision

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

## Supersedes

This clarifies the earlier implicit preservation of unbounded successful command
stdout and strengthens the plan's complete-boundary cleanup requirement with
the ownership behavior of `openssh::Child::wait`.

## Required Authoritative Updates

- `01-idea.md`
- `02-plan.md`
- `03-tasks.md`
