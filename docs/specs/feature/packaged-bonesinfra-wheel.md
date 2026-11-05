# Packaged BonesInfra Wheel

<!-- Merged from feature/infra-binary -->

## Idea

### Request

Replace the expanded, committed `infra/.framework/` BonesInfra source tree with
one committed wheel artifact. Keep BonesInfra templates as ordinary committed
files under `infra/` so projects can inspect and edit them.

### Problem

Projects currently commit the complete managed BonesInfra source tree, including
many files that are not project-specific. This creates noisy repositories and
makes it difficult to distinguish project infrastructure from the BonesDeploy
implementation it depends on.

### Definitions

**Managed wheel:** A pure-Python `.whl` containing BonesInfra code, metadata, and
the package resources required by the runtime. It is generated and committed in
the `bonesinfra` Rust crate, embedded in the `bonesdeploy` executable, and copied
to each project as one artifact. It is not a native executable.

**Managed templates:** The Jinja and script templates supplied by BonesInfra and
copied into the project’s `infra/` directory. They are ordinary project files
and may be edited by the project owner.

### Desired outcome

An initialized project contains one committed BonesInfra wheel and a committed
template tree under `infra/`; it does not contain `infra/.framework/` or the
expanded managed Python source. BonesDeploy installs and executes the committed
wheel in its existing project-scoped cached virtualenv, while provisioning uses
the project’s committed templates and `infra/custom/` extensions.

`bonesdeploy update` replaces the managed wheel and refreshes managed templates
from the new executable. Managed templates are intentionally replaced wholesale
in v1. Project-specific behavior belongs in `infra/custom/`.

### Scope

This includes committed wheel generation and embedding, the project artifact
layout, wheel installation and hash detection, initialization, update
synchronization, template loading, custom provisioning composition, migration of
existing projects, and tests and documentation.

### Constraints

The wheel must be a portable pure-Python `py3-none-any` artifact. It is committed
in the Rust crate, included in the crates.io package, and embedded in the
executable. End users must not need Python wheel-building tools during Cargo
compilation. BonesDeploy installs the wheel rather than importing from the wheel
archive. Existing project-scoped dependency caching and atomic replacement
behavior are preserved.

All supported framework templates and shared template assets are copied into the
project template tree. `infra/custom/` and `infra/secrets/` remain project-owned.

### Exclusions

V1 does not create a native platform-specific executable, preserve edits to
managed templates across updates, merge templates, or create a template
manifest. It does not move `infra/custom/`, `infra/secrets/`, or deployment
scripts into the wheel, lock third-party dependencies, or redesign the remote
`bonesremote` lifecycle.


## Plan

### Current behavior

`crates/bonesinfra/src/lib.rs` embeds the Python tree, materializes it at
`infra/.framework/`, creates a cached virtualenv, and installs that directory
editable with pip. Commands run `python -m bonesinfra` with
`infra/.framework/src` on `PYTHONPATH`.

`bonesinfra.project` imports managed framework modules from that tree and loads
`infra/custom/` separately. Framework and shared templates are read beside the
managed Python modules. Initialization materializes the tree, while update
refreshes it before applying local patches.

### Intended behavior

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

### Approach

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

### Responsibilities and boundaries

`bonesinfra::` owns wheel embedding, artifact validation, cache installation,
atomic artifact replacement, and template materialization.

`commands/init/scaffold.rs` owns the initial wheel/template layout.
`commands/update/sync.rs` owns wholesale template refresh; `commands/update/mod.rs`
owns update ordering and the self-update handoff before the new wheel is used.

The Python project loader owns managed-package discovery and composition. Python
framework and shared service modules consume the project template paths but do
not own copying or update policy. `infra/custom/` remains the project-owned
extension boundary.

### Affected areas

`crates/bonesinfra/src/lib.rs` and tests; the Python project loader,
configuration/path helpers, template consumers, and tests; initialization and
update scaffolding/synchronization; `.gitignore` and path constants; wheel
generation/version/release packaging; Cargo package inclusion; fixture tests;
and `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`,
`docs/ARCHITECTURE.md`, `docs/architecture/reference.md`, and `README.md`.

### Decisions

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

### Risks

The wheel may omit a resource previously available beside the source tree, so
package-resource tests must cover every runtime path. Template path changes may
alter generated server configuration, so rendering tests must use the project
snapshot. An interrupted refresh may leave a partial template tree, so rerunning
update must be safe. Dependency installation still requires a compatible local
Python and network/cache access.

### Validation

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


## Tasks

### Implementation

- [x] Define the committed wheel and `infra/templates/` paths, cache stamp identity, and centralized path constants; explicitly omit the v1 template manifest.
- [x] Add the generated `py3-none-any` wheel as a committed `bonesinfra` crate asset, update release/version tooling, and verify Cargo/crates.io packaging includes it.
- [x] Embed the committed wheel and complete template inventory from the `bonesinfra` crate.
- [x] Replace editable `.framework` materialization with wheel and template materialization, including migration from existing expanded trees without promising preservation of managed `.framework` edits.
- [x] Install and execute the committed wheel through the project-scoped virtualenv, keyed by wheel content identity, while preserving typed request stdin behavior.
- [x] Update Python managed-framework and asset consumers to load all templates from `infra/templates/` while preserving `infra/custom/` composition.
- [x] Update initialization, `.gitignore`, update ordering, and template synchronization to refresh managed templates wholesale while preserving `infra/custom/` and `infra/secrets/`.
- [x] Ensure the post-self-update execution path uses the new executable’s embedded wheel and templates before local patches run.
- [x] Revise focused Rust/Python/e2e fixture tests and affected documentation for the wheel and template workflow.

### Validation

- [x] Verify focused `bonesinfra` tests prove the wheel is valid, installable, complete, embedded, and included in the Cargo package archive.
- [x] Verify init tests prove the project contains one committed wheel plus the complete template tree and no expanded `.framework` tree.
- [x] Verify update tests prove wheel replacement, cache invalidation, wholesale template refresh, old-tree migration, and preservation of custom infrastructure and secrets.
- [x] Run affected Rust tests and `uv run pytest` in `crates/bonesinfra/python`; do not run e2e tests locally.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`, then address all warnings and errors.
- [x] Review a clean generated project and execute a provisioning command from its committed wheel and templates.

### Completion

- [x] Remove obsolete expanded-tree assumptions, stale tests, unused materialization code, and outdated documentation.
- [x] Update `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`, `README.md`, and architecture references where the ownership or layout changed.
- [x] Review the final diff for artifact integrity, template refresh safety, path centralization, and absence of unrelated changes.

### Completion notes

<!-- Record meaningful deviations, validation results, and deliberately unfinished work after implementation. -->

Implemented the committed pure-Python wheel and project template snapshot. The
 wheel is generated by `cargo build-wheel`, embedded by Rust, installed into the
 project cache, and copied into versioned `infra/bonesinfra-<version>-py3-none-any.whl`; managed templates are
copied into `infra/templates/` and refreshed wholesale. E2E tests were not run
per repository guidance. `cargo test --workspace --exclude e2e`, Python pytest,
ruff, cargo clippy, cargo fmt, and shfmt completed successfully.
