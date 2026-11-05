# Clarification

## Trigger

The user instructed that keeping all update patch behavior in a single
`patches.rs` file is not a good long-term plan. The patch implementation
should live in a `crates/bonesdeploy/src/commands/update/patches/` folder,
with each patch grouped in its own module.

## Decision

Restructure the `bonesdeploy` side into a directory module at
`crates/bonesdeploy/src/commands/update/patches/` containing:

- `mod.rs` — the patch registry, version ordering and selection, marker
  handling, and local/remote dispatch.
- `config_repo.rs` — the `0001-config-repo` patch: ensure the `.bones`
  directory is a Git repository and its `origin` remote points at the
  configured config repository.
- `root_config_repo.rs` — the `0002-root-config-repo` patch: ensure
  `origin` in `.bones` targets the root-owned config repository.
- `git.rs` — shared Git command and config-repository URL helpers used by
  the patch modules.

Each patch's migration behavior belongs to exactly one module, so a future
patch adds a module instead of growing a single file.

The remote side keeps its single `bonesremote patch apply` command module
because both remote patch identifiers map to one shared configuration
repository migration implemented there.

## Supersedes

The `02-plan.md` affected-areas entry naming
`crates/bonesdeploy/src/commands/update/patches.rs` as the single home of the
local patch implementation.

## Effect on the record

- `01-idea.md`: unchanged; the file layout is an implementation detail below
  the idea's scope.
- `02-plan.md`: "Affected areas" now names the `patches/` folder and its
  four modules; "Responsibilities and boundaries" now assigns each patch
  identifier to its own module.
- `03-tasks.md`: added a task to split the single file into the folder
  module layout, marked with the other completed restructuring work.