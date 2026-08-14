# Clarification

## Trigger

The current `patches/git.rs` name and location make generic Git process
operations look like patch-specific behavior. The patch orchestration also
places local migrations and remote SSH dispatch beside each other without a
clear local/remote boundary.

## Decision

Move generic Git-at-path operations into the existing
`crates/bonesdeploy/src/infra/git.rs` module, which owns Git infrastructure.
Keep config-repository URL construction with the local config-repository
patches, where its project configuration meaning is visible.

Organize update patches as explicit local and remote modules:

- `patches/mod.rs` owns the registry, version selection, marker handling, and
  top-level update orchestration.
- `patches/local/mod.rs` owns local patch dispatch and local config-repository
  URL construction.
- `patches/local/config_repo.rs` owns `0001-config-repo`.
- `patches/local/root_config_repo.rs` owns `0002-root-config-repo`.
- `patches/remote/mod.rs` owns only the SSH invocation of the remote
  `bonesremote patch apply` command.

Remote migration implementation remains in `bonesremote`, where remote
filesystem and root privilege operations belong.

## Supersedes

The prior clarification's decision to place shared Git helpers in
`crates/bonesdeploy/src/commands/update/patches/git.rs`, and the corresponding
plan that placed local and remote dispatch together without explicit module
boundaries.

## Effect on the record

- `01-idea.md`: unchanged; this clarifies implementation ownership only.
- `02-plan.md`: updated responsibilities, approach, and affected areas to name
  `infra/git.rs`, `patches/local/`, and `patches/remote/mod.rs`.
- `03-tasks.md`: added and completed the boundary cleanup task.
