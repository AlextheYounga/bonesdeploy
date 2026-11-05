# Plan

## Current behavior

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

## Intended behavior

The embedded distribution is atomically copied as the complete managed core.
Every Python command runs the package at `infra/.framework/src/bonesinfra`
through a cached project-specific virtual environment. The local package is
mandatory; no installed-source or selected-framework fallback exists. Core and
custom provisioning retain their explicit composition order.

Update first installs the new local binary and re-execs it for continuation.
The continuation materializes core before applying local and remote patches,
then synchronizes deployment assets. The legacy patch keeps pre-materialized
core and moves legacy project-owned infrastructure into custom provisioning.

## Approach

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

## Responsibilities and boundaries

The `bonesinfra` Rust crate owns embedded-source extraction, atomic managed-core
replacement, project cache identity, venv preparation, and Python process
execution. The Python package owns local core/custom composition and patch
migrations. `bonesdeploy init` owns selecting framework/deployment assets and
requests core materialization. `bonesdeploy update` owns installation,
continuation, operation ordering, and deployment asset synchronization.

## Affected areas

- `crates/bonesinfra/src/lib.rs` and its integration tests.
- `crates/bonesinfra/python/src/bonesinfra/project.py`, CLI registration,
  patch migration code, and Python tests.
- `crates/bonesdeploy/src/commands/init/scaffold.rs` and update modules.
- Rust init and update synchronization tests.
- `CONTEXT.md`, Python context, README, architecture documents, and prior
  decentralization/project-infra planning records.

## Decisions

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

## Risks

Atomic replacement must not delete custom provisioning when replacing its
sibling. Editable environments must be refreshed when core changes and must
not accidentally import a global package. Re-exec must retain update options
and terminate the old process. Migration must preserve encrypted bytes and
leave both old and new trees intact on a collision.

## Validation

- Python tests prove mandatory local core loading, framework selection,
  core-before-custom composition, and legacy migration behavior.
- Rust tests prove full-core init layout, atomic core replacement with custom
  preservation, and update synchronization no longer handles core.
- Exercise project-local execution by placing a distinct marker in materialized
  core and observing the invoked command load it.
- Run `ruff check .`, `ruff format .`, `uv run pytest`, affected Rust tests,
  workspace tests excluding e2e, `cargo clippy`, `cargo fmt`, and `shfmt -w .`.
