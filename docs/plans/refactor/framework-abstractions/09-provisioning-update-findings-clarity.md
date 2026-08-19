# Clarification

## Trigger

The provisioning and update audit found concrete existing contracts and duplicate
paths that were compressed out of the parent record.

## Decision

The provisioning child change will preserve and strengthen the existing
`LanguageRuntime`, `RuntimeService`, Manifest, project materialization, and pyinfra
runner concepts.

The update child change will make the Python `Patch` registry the sole migration
mechanism and give managed infrastructure synchronization one owner.

Concrete findings are:

- Python framework `runtime.py` files repeat the same setup, shared-directory,
  application, custom-hook, and service-start workflow;
- built-in `custom.py` hooks overlap with materialized `infra/custom`
  composition;
- direct pyinfra global `host` access escapes the runner boundary;
- infrastructure paths are scattered as module constants instead of consistently
  using `DeploymentPaths`;
- `RuntimeService.get_service()` exits the process on lookup failure;
- the live patch is `0003-project-infra`, introduced in version `0.8.0`, and its
  remote scope writes markers only;
- `crates/bonesdeploy/src/commands/migrate.rs` is a dead Rust duplicate of the
  live Python migration;
- update synchronization copies managed `infra/.framework` and deployment
  content, refuses modified-core conflicts, and preserves custom content.

The child plans must preserve language/service extension points, managed-core versus
custom ownership, patch markers, idempotency, and revision consistency.

## Supersedes

This adds the concrete provisioning and migration findings that were compressed out
of the parent summary and supersedes the generic provisioning/update descriptions.

## Effect on the record

- `02-plan.md` identifies the repeated workflow, custom-hook overlap, runner/path
  bypasses, live patch, dead migration, and managed sync behavior.
- `03-tasks.md` requires separate child plans to address provisioning composition
  and update/migration ownership.
