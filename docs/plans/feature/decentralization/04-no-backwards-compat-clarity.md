# Clarification

## Trigger

The user directed: "we should not be trying to do any backwards compatibility for existing workspaces using this tool." Commands must not detect, adapt to, or silently support old `.bones`-based project layouts.

## Decision

This refactor is a hard cut. Once `bonesdeploy` is updated, it will not read or adapt to the old `.bones` symlink, `bones.toml`, nested Git, or config-repository layout. Commands that encounter the old layout fail with a clear message directing the user to run the explicit migration. No dual-mode code paths, fallback loading, or format auto-detection survive.

The migration tool is the single deliberate bridge: it copies project-owned
files from `.bones` into `infra/` and removes the old workspace structure. It
does not delete or relocate machine-local BonesDeploy state, including GPG
keyrings. After migration, the tool works against `infra/`. Before migration,
it refuses to deploy, setup, or provision.

## Supersedes

Adds an explicit constraint without changing an earlier decision. The previous plan already described migration as deliberate and commands as adopting new boundaries, but did not explicitly rule out auto-detection or dual-mode fallback behavior.

## Effect on the record

- `01-idea.md`: Added no-backwards-compatibility to Constraints.
- `02-plan.md`: Specified that commands encountering old layout fail with a migration message rather than detecting or adapting. Migration section clarified as the single bridge.
- `03-tasks.md`: Added task for clear failure messages on old layout; migration task emphasizes no auto-detection.
