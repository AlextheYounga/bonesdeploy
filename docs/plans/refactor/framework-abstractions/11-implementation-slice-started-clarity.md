# Clarification

## Trigger

The user explicitly authorized implementation in the
`refactor/framework-abstractions` worktree after reviewing the framework
abstraction records.

## Decision

Implementation begins with a bounded bootstrap slice before the remaining child
changes:

- resolve the committed `manifest.rs` conflict using the post-decentralization
  `.env` implementation;
- correct `docs/ARCHITECTURE.md` and `docs/architecture/reference.md`;
- implement the concrete Rust `Framework` identity and dispatch boundary;
- migrate its known callers and preserve Python provisioning ownership;
- keep the remaining configuration, integration, provisioning, update, deployment,
  doctor, and final-audit boundaries as separate future slices.

The Framework slice does not change the persisted `Runtime` schema, Python
materialization, Git/SSH ownership, or remote deployment behavior.

## Supersedes

This narrows the parent exclusion that implementation was entirely outside the
current worktree. It does not merge the remaining child changes into the parent.

## Effect on the record

- `01-idea.md` now identifies the bounded bootstrap and Framework slice as the
  authorized initial implementation scope.
- `02-plan.md` records the completed bootstrap/documentation/framework sequence and
  leaves later boundaries in their existing dependency order.
- `03-tasks.md` records completed initial-slice tasks, validation, and deliberately
  unfinished child boundaries.
