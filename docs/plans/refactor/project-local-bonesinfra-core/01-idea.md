# Idea

## Request

Make `infra/.framework/` the complete executable BonesInfra source for a
project. `bonesdeploy update` must install the new binary, continue with that
binary, replace managed core, run local and remote patches from that core, and
then refresh deployment assets.

## Problem

BonesInfra currently extracts its complete Python distribution into
`~/.cache/bonesdeploy/bonesinfra` and executes it there. Project
`infra/.framework/` receives only a selected framework snapshot, whose
modules import the hidden cached implementation. This makes a committed project
tree incomplete, lets update run patches before its new code is materialized,
and contradicts the project-local infrastructure boundary.

## Definitions

**Managed core** is the complete BonesInfra Python distribution under
`infra/.framework/`, including its packaging metadata, executable package,
frameworks, patches, and assets. It is replaced as one managed tree during an
explicit update.

**Custom provisioning** is project-owned code under
`infra/custom/`. The managed framework imports and composes it after framework
provisioning. Core replacement never changes this directory.

**Dependency environment** is a project-specific cached Python virtual
environment containing third-party dependencies and an editable installation
of that project's managed core. It is not an executable source checkout and
does not provide a fallback provisioning implementation.

## Desired outcome

Fresh initialization writes a complete, runnable BonesInfra distribution to
`infra/.framework/`. Provisioning, manifest, and patch commands execute
that project-local package. Updating atomically replaces managed core before
running patches, preserves custom provisioning, and refreshes deployment assets
after patching. A legacy `.bones` migration retains project-owned provisioning,
deployment files, and encrypted secrets without replacing new managed core.

## Scope

- Complete managed-core materialization, project-local command execution, and
  project-scoped dependency environments.
- Initialization and update ordering, including continuation under an installed
  new binary.
- Local patch migration, obsolete framework-only synchronization removal, and
  focused Rust and Python regression tests.
- Documentation and prior planning-record corrections.

## Constraints

- `infra/.framework/` is the only executable BonesInfra source for normal
  project commands and patches.
- `~/.cache/bonesdeploy` contains dependencies and editable-install metadata,
  not an authoritative BonesInfra source checkout.
- Framework replacement is atomic and must preserve `infra/custom/`.
- Patches run only after the new core is materialized.
- Do not run end-to-end tests or create commits.

## Exclusions

- A general plugin framework or additional provisioning ownership layers.
- A three-way merge for modified managed core files; managed core is replaced
  as an explicit update operation.
- Changes to unrelated deployment or remote release behavior.
