# Project Local BonesInfra Core

<!-- Merged from refactor/project-local-bonesinfra-core -->

## Idea

### Request

Make `infra/.framework/` the complete executable BonesInfra source for a
project. `bonesdeploy update` must install the new binary, continue with that
binary, replace managed core, run local and remote patches from that core, and
then refresh deployment assets.

### Problem

BonesInfra currently extracts its complete Python distribution into
`~/.cache/bonesdeploy/bonesinfra` and executes it there. Project
`infra/.framework/` receives only a selected framework snapshot, whose
modules import the hidden cached implementation. This makes a committed project
tree incomplete, lets update run patches before its new code is materialized,
and contradicts the project-local infrastructure boundary.

### Definitions

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

### Desired outcome

Fresh initialization writes a complete, runnable BonesInfra distribution to
`infra/.framework/`. Provisioning, manifest, and patch commands execute
that project-local package. Updating atomically replaces managed core before
running patches, preserves custom provisioning, and refreshes deployment assets
after patching. A legacy `.bones` migration retains project-owned provisioning,
deployment files, and encrypted secrets without replacing new managed core.

### Scope

- Complete managed-core materialization, project-local command execution, and
  project-scoped dependency environments.
- Initialization and update ordering, including continuation under an installed
  new binary.
- Local patch migration, obsolete framework-only synchronization removal, and
  focused Rust and Python regression tests.
- Documentation and prior planning-record corrections.

### Constraints

- `infra/.framework/` is the only executable BonesInfra source for normal
  project commands and patches.
- `~/.cache/bonesdeploy` contains dependencies and editable-install metadata,
  not an authoritative BonesInfra source checkout.
- Framework replacement is atomic and must preserve `infra/custom/`.
- Patches run only after the new core is materialized.
- Do not run end-to-end tests or create commits.

### Exclusions

- A general plugin framework or additional provisioning ownership layers.
- A three-way merge for modified managed core files; managed core is replaced
  as an explicit update operation.
- Changes to unrelated deployment or remote release behavior.


## Plan

### Current behavior

`crates/bonesinfra/src/lib.rs` embeds the full Python distribution, extracts it
to `~/.cache/bonesdeploy/bonesinfra`, creates its venv there, and runs
`python -m bonesinfra`. `project.materialize()` copies only a selected
framework package into `infra/.framework/` and creates custom stubs.
`project.py` loads that snapshot when present but otherwise imports framework
modules from the installed package.

Initialization invokes the Python `project materialize` CLI. Update installs a
new local binary, applies patches using the current cached package, then copies
framework files through `update::sync`. The current migration patch predates
this layout and moves legacy `.bones/infra` content to the wrong project path.

### Intended behavior

The embedded distribution is atomically copied as the complete managed core.
Every Python command runs the package at `infra/.framework/src/bonesinfra`
through a cached project-specific virtual environment. The local package is
mandatory; no installed-source or selected-framework fallback exists. Core and
custom provisioning retain their explicit composition order.

Update first installs the new local binary and re-execs it for continuation.
The continuation materializes core before applying local and remote patches,
then synchronizes deployment assets. The legacy patch keeps pre-materialized
core and moves legacy project-owned infrastructure into custom provisioning.

### Approach

Refactor the Rust embedding boundary into two operations: atomically
materialize embedded files at a project core path, and prepare a cached venv
that editable-installs that core. Derive the cache directory from the canonical
project root so environments cannot reuse another project's source. Execute
the venv Python with the project core on `PYTHONPATH`, preserving package
imports while making the source location explicit.

Remove Python framework-only materialization and built-in fallback. Keep the
canonical frameworks in the embedded distribution as materialization content;
the locally executed package selects and loads its own framework modules and
then composes custom modules. Replace init's Python materialize command with
the Rust materialization boundary.

Make update continuation explicit with an internal command flag that prevents
recursion. The original process installs a newer binary and invokes it with the
same update options. The continued process materializes the new core, applies
patches from it, updates remote components, and refreshes deployment assets.
Remove managed-core behavior from deployment asset synchronization.

### Responsibilities and boundaries

The `bonesinfra` Rust crate owns embedded-source extraction, atomic managed-core
replacement, project cache identity, venv preparation, and Python process
execution. The Python package owns local core/custom composition and patch
migrations. `bonesdeploy init` owns selecting framework/deployment assets and
requests core materialization. `bonesdeploy update` owns installation,
continuation, operation ordering, and deployment asset synchronization.

### Affected areas

- `crates/bonesinfra/src/lib.rs` and its integration tests.
- `crates/bonesinfra/python/src/bonesinfra/project.py`, CLI registration,
  patch migration code, and Python tests.
- `crates/bonesdeploy/src/commands/init/scaffold.rs` and update modules.
- Rust init and update synchronization tests.
- `CONTEXT.md`, Python context, README, architecture documents, and prior
  decentralization/project-infra planning records.

### Decisions

1. Managed core is replaced as a complete tree rather than file-by-file. This
   removes stale managed files and gives each update a coherent package.
2. Cache environments are keyed by canonical project root and contain no copied
   BonesInfra source. Editable installation keeps dependencies cached while
   executing the project's managed core.
3. Local core is mandatory. A missing or invalid project core fails before any
   provisioning connection and never falls back to cached embedded code.
4. The update process re-execs only after a local binary install, using an
   internal continuation flag to prevent recursive installation.
5. Legacy `.bones/infra` is project-owned content and migrates into custom
   provisioning while pre-materialized core is retained. Colliding custom,
   deployment, or secrets destinations fail without writes.

### Risks

Atomic replacement must not delete custom provisioning when replacing its
sibling. Editable environments must be refreshed when core changes and must
not accidentally import a global package. Re-exec must retain update options
and terminate the old process. Migration must preserve encrypted bytes and
leave both old and new trees intact on a collision.

### Validation

- Python tests prove mandatory local core loading, framework selection,
  core-before-custom composition, and legacy migration behavior.
- Rust tests prove full-core init layout, atomic core replacement with custom
  preservation, and update synchronization no longer handles core.
- Exercise project-local execution by placing a distinct marker in materialized
  core and observing the invoked command load it.
- Run `ruff check .`, `ruff format .`, `uv run pytest`, affected Rust tests,
  workspace tests excluding e2e, `cargo clippy`, `cargo fmt`, and `shfmt -w .`.


## Tasks

### Implementation

- [x] Replace cache-source extraction with atomic complete-core materialization
  and project-scoped dependency environment preparation.
- [x] Require local core in the Python loader, preserve core/custom composition,
  and remove the framework-only materialization CLI.
- [x] Materialize complete core during init and remove framework-core update
  synchronization.
- [x] Re-exec update after local installation, materialize core before patches,
  and retain deployment asset refresh ordering.
- [x] Migrate legacy project-owned `.bones` content into custom provisioning
  while preserving pre-materialized core and rejecting destination collisions.
- [x] Correct affected architecture, context, README, and superseded plan text.

### Validation

- [x] Add and run focused Python tests for mandatory local core, composition,
  and legacy migration.
- [x] Add and run focused Rust tests for complete materialization, preservation
  of custom provisioning, init layout, and deployment-only synchronization.
- [x] Run package, workspace, formatting, lint, and shell formatting checks
  without e2e tests.

### Completion

- [x] Review the final diff for executable cached source, built-in fallback,
  and obsolete framework-only synchronization.

### Completion notes

- Replaced cache checkout extraction with project-local complete distribution
  materialization. Cached environments use the package version from
  `pyproject.toml` and a readable project-path cache location.
- Validation passed: `ruff check .`, `ruff format .`, `uv run pytest`,
  `cargo test --workspace --exclude e2e`, `cargo clippy --workspace --exclude
  e2e --all-targets --all-features -- -D warnings`, `cargo fmt`, and
  `shfmt -w .`.


## Clarification

### Trigger

The earlier project-infrastructure record retained a framework-only project
snapshot and an installed BonesInfra fallback. That leaves the project unable to
run its committed provisioning implementation independently.

### Decision

Managed core is the complete BonesInfra distribution under
`infra/.framework/`. The project-local distribution is the sole executable
source for normal provisioning and patches. Cache directories hold only
project-specific dependency environments and editable-install metadata.

`bonesdeploy update` installs a new binary, continues with that binary,
atomically replaces the managed framework, applies patches from that framework, and then
refreshes deployment assets. Legacy `.bones/infra` content is project-owned and
moves to `infra/custom/`; the new framework is preserved.

### Supersedes

This supersedes the framework-only materialization, installed fallback, and
“no copied BonesInfra execution engine” decisions in
`refactor/project-infra` and
`feature/decentralization/06-managed-custom-infrastructure-clarity.md`.

### Effect on the record

- `01-idea.md` defines the complete managed framework and cache-only dependency
  environments.
- `02-plan.md` assigns atomic materialization and project-local execution to
  the Rust embedding boundary and records update ordering.
- `03-tasks.md` tracks implementation, migration, regression tests, and
  validation for the complete-core model.
