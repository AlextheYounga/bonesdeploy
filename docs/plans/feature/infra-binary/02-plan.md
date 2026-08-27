# Plan

## Current behavior

`crates/bonesinfra/src/lib.rs` embeds the Python tree, materializes it at
`infra/.framework/`, creates a cached virtualenv, and installs that directory
editable with pip. Commands run `python -m bonesinfra` with
`infra/.framework/src` on `PYTHONPATH`.

`bonesinfra.project` imports managed framework modules from that tree and loads
`infra/custom/` separately. Framework and shared templates are read beside the
managed Python modules. Initialization materializes the tree, while update
refreshes it before applying local patches.

## Intended behavior

The release/build workflow creates a versioned pure-Python wheel from the
BonesInfra Python package and commits it under the `bonesinfra` Rust crate. The
wheel is included in the crates.io package and embedded in the `bonesdeploy`
executable. Users building from the published crate do not need Python packaging
tools or network access to create the wheel.

Initialization copies the embedded wheel beneath `infra/` and copies the
complete managed template set into `infra/templates/`. It does not create
`infra/.framework/`.

The Rust wrapper validates the committed wheel, hashes it for cache identity,
installs it non-editably into the project-scoped virtualenv, and runs the
installed `bonesinfra` package. Python loads managed framework logic from that
installation and loads all managed templates from `infra/templates/`, while
continuing to compose `infra/custom/`.

Update first runs using the newly installed BonesDeploy executable after the
normal self-update handoff. That executable copies its embedded wheel and
template snapshot into the project. Managed templates are refreshed wholesale;
there is no manifest or merge behavior in v1. Existing `.framework` contents are
removed as part of migration. Files in `infra/custom/` and `infra/secrets/` are
not touched. Edits inside the old managed `.framework` tree are outside the
supported migration contract.

## Approach

The `bonesinfra` Rust crate is the artifact boundary. A release-maintained
generation step builds the Python wheel and stores it as a crate asset. Rust
embeds the committed wheel and the complete template inventory, and writes the
wheel plus template tree with the existing atomic replacement discipline.
Cargo packaging tests confirm that the wheel is included in the crates.io source
archive.

The cached environment stamp changes from the parsed `pyproject.toml` version to
the committed wheel content identity and Python compatibility metadata. A cache
miss installs the wheel into the existing venv with pip; dependency resolution
remains in the cache and is not committed. The committed wheel is the source of
truth for the managed BonesInfra code installed in the project environment.

The Python package gets a project-template-root helper based on the current
project working directory. Shared assets and every supported framework’s
templates are copied under `infra/templates/` in a defined relative layout.
Framework runtime modules use those project paths instead of paths derived from
`__file__`. The project loader keeps its existing custom module import and
composition behavior.

The update synchronizer refreshes managed templates wholesale. It does not
inspect local template hashes or write a manifest in v1. Individual file and
directory replacements remain safe, and rerunning update repairs an interrupted
refresh.

## Responsibilities and boundaries

`bonesinfra::` owns wheel embedding, artifact validation, cache installation,
atomic artifact replacement, and template materialization.

`commands/init/scaffold.rs` owns the initial wheel/template layout.
`commands/update/sync.rs` owns wholesale template refresh; `commands/update/mod.rs`
owns update ordering and the self-update handoff before the new wheel is used.

The Python project loader owns managed-package discovery and composition. Python
framework and shared service modules consume the project template paths but do
not own copying or update policy. `infra/custom/` remains the project-owned
extension boundary.

## Affected areas

`crates/bonesinfra/src/lib.rs` and tests; the Python project loader,
configuration/path helpers, template consumers, and tests; initialization and
update scaffolding/synchronization; `.gitignore` and path constants; wheel
generation/version/release packaging; Cargo package inclusion; fixture tests;
and `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`,
`docs/ARCHITECTURE.md`, `docs/architecture/reference.md`, and `README.md`.

## Decisions

1. Use a `py3-none-any` wheel instead of a native executable for cross-platform
   support.
2. Generate and commit the wheel in the Rust crate rather than build it during
   end-user Cargo compilation. This keeps crates.io builds independent of Python
   packaging tools and network availability.
3. Install the wheel into the existing cached virtualenv instead of importing
   directly from the ZIP archive.
4. Store all supported framework templates and shared template assets in the
   project so they are inspectable and editable.
5. Replace managed templates wholesale in v1. A merge manifest is deferred until
   preserving managed-template edits becomes a concrete requirement.
6. Use wheel content identity for cache invalidation rather than version alone.

## Risks

The wheel may omit a resource previously available beside the source tree, so
package-resource tests must cover every runtime path. Template path changes may
alter generated server configuration, so rendering tests must use the project
snapshot. An interrupted refresh may leave a partial template tree, so rerunning
update must be safe. Dependency installation still requires a compatible local
Python and network/cache access.

## Validation

Run focused Rust and Python tests covering wheel validity, embedding, Cargo
package inclusion, installation-cache invalidation, atomic replacement,
initialization, migration, template loading, wholesale refresh, and custom
composition. Run affected crate tests and the Python suite, then `cargo fmt`,
`cargo clippy`, and shell formatting checks without running e2e tests locally.

Inspect a generated project to confirm Git tracks one wheel, all managed
templates, custom infrastructure, and secrets while ignoring generated cache
state and `.framework`. Install from a copied project in a clean environment,
run a provisioning command, and verify the committed wheel and templates are
used. Review the final diff and documentation for stale expanded-tree claims.
