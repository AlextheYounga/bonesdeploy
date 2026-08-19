# Clarification

## Trigger

The earlier project-infrastructure record retained a framework-only project
snapshot and an installed BonesInfra fallback. That leaves the project unable to
run its committed provisioning implementation independently.

## Decision

Managed core is the complete BonesInfra distribution under
`infra/provision/core/`. The project-local distribution is the sole executable
source for normal provisioning and patches. Cache directories hold only
project-specific dependency environments and editable-install metadata.

`bonesdeploy update` installs a new binary, continues with that binary,
atomically replaces managed core, applies patches from that core, and then
refreshes deployment assets. Legacy `.bones/infra` content is project-owned and
moves to `infra/provision/custom/`; the new core is preserved.

## Supersedes

This supersedes the framework-only materialization, installed fallback, and
“no copied BonesInfra execution engine” decisions in
`refactor/project-infra` and
`feature/decentralization/06-managed-custom-infrastructure-clarity.md`.

## Effect on the record

- `01-idea.md` defines complete managed core and cache-only dependency
  environments.
- `02-plan.md` assigns atomic materialization and project-local execution to
  the Rust embedding boundary and records update ordering.
- `03-tasks.md` tracks implementation, migration, regression tests, and
  validation for the complete-core model.
