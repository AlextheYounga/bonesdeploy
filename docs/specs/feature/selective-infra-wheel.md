# Selective Infra Wheel

<!-- Merged from feature/selective-infra-wheel -->

## Idea

### Request

Keep one universal BonesInfra wheel that installs normally. After BonesDeploy
materializes the complete `infra/` directory, selectively prune files and
folders that belong to frameworks other than the project's selected framework.
Define the framework-owned paths in a ruleset represented as a map in
BonesInfra.

### Problem

BonesInfra currently materializes templates for every built-in framework into
each project's `infra/` directory. A project uses one native framework, or no
native framework when it uses Docker, so the other framework-specific files are
irrelevant and obscure which managed infrastructure applies to the project.

### Definitions

**Universal wheel:** The single release-built `bonesinfra-*.whl` containing all
shared and framework-specific Python code. Every project receives the same wheel
bytes. The wheel is not tailored, rewritten, or pruned.

**Materialized infrastructure:** The files written under a project's `infra/`
directory by BonesDeploy and BonesInfra. This includes the universal wheel,
managed templates, deployment scripts, and project-owned directories.

**Framework ruleset:** A BonesInfra map from each supported framework name to
the exact `infra/`-relative managed files or directories owned by that
framework. Only paths named by this map are eligible for framework pruning.

**Pruning:** Removing framework-owned paths from the materialized `infra/`
directory after normal materialization. Pruning does not inspect or alter wheel
contents.

### Desired outcome

Every initialized or updated project contains the unchanged universal
BonesInfra wheel, shared infrastructure files, and only the materialized files
associated with its selected native framework. Docker projects contain no
framework-owned materialized files. Re-running materialization after a
framework or backend change removes paths belonging to the previous selection.

### Scope

- Add the framework-to-path ruleset to BonesInfra.
- Prune unselected framework paths after normal project materialization.
- Pass the finalized backend and framework selection through init and update.
- Cover native, custom, Docker, repeated materialization, and profile-change
  behavior with tests.
- Update documentation that currently describes complete framework template
  materialization.

### Constraints

- Keep exactly one canonical universal wheel and copy it unchanged into every
  project.
- Materialize the complete managed infrastructure before pruning it.
- Delete only exact paths declared in the framework ruleset.
- Keep shared infrastructure, the selected framework's paths, project-owned
  `infra/custom/`, `infra/secrets/`, and the universal wheel.
- Treat a native empty template as `custom`; treat Docker as having no selected
  framework.
- Use the finalized runtime configuration rather than independently parsing
  partially written project files.

### Exclusions

- Tailoring, rebuilding, or editing wheel archives and wheel metadata.
- Deleting framework packages from the cached Python environment.
- Static dependency or import-graph analysis.
- Pruning shared language runtimes, server logic, Compose support, or other
  paths not explicitly associated with a framework in the ruleset.
- Introducing a plugin system or user-configurable pruning rules.


## Plan

### Current behavior

`crates/bonesinfra/src/lib.rs` embeds one `bonesinfra-*.whl` and copies its
bytes into `infra/`. It separately embeds shared assets and every framework's
templates, stages them all, and atomically replaces `infra/templates/`.

`crates/bonesdeploy/src/commands/init/scaffold.rs` already knows the finalized
runtime backend and selected framework. It materializes selected deployment
scripts through `scaffold_framework_project()` and then asks BonesInfra to
materialize the universal wheel and all templates.

`crates/bonesdeploy/src/commands/update/mod.rs` currently rematerializes the
wheel and complete template tree before loading project configuration and
applying local patches. Its later infrastructure sync already selects one
framework's deployment scripts.

The only complete all-framework tree currently written to a project is
`infra/templates/frameworks/<framework>/`. The wheel also contains all
framework Python packages, but remains a universal installation artifact rather
than a pruning target.

### Intended behavior

BonesInfra continues its existing materialization unchanged: copy the universal
wheel and replace `infra/templates/` with the complete embedded template tree.
It then uses the framework ruleset to remove paths belonging to every
unselected framework.

A native project keeps the paths mapped to its selected framework. A native
project with an empty template keeps the `custom` paths. A Docker project has no
selected framework and removes every framework-owned path. Paths absent from
the ruleset are never removed by framework pruning.

Init and update both supply the selection derived from finalized runtime
configuration. Updating after a framework or backend change rematerializes the
complete tree first and then prunes it for the new selection, so stale paths do
not survive and newly selected paths are restored.

### Approach

1. Define a BonesInfra framework ruleset as a map from each supported framework
   name to its exact `infra/`-relative managed paths. Initially each framework
   owns its `templates/frameworks/<name>` directory.
2. Represent the project selection as either one framework name or no framework
   for Docker. Validate a selected name against the map before deleting files.
3. Keep the existing wheel copy and complete template replacement logic.
4. After materialization succeeds, iterate the ruleset entries other than the
   selected framework and remove their listed files or directories with the
   existing symlink-aware path removal helper.
5. Derive and pass the selection from init's finalized runtime configuration.
6. During update, load configuration before materialization, derive the same
   selection, materialize and prune, then apply local patches and continue the
   existing infrastructure sync.

### Responsibilities and boundaries

- `bonesinfra` owns the ruleset, selection validation, materialization order,
  and pruning because it owns the complete managed template inventory and the
  project artifact writer.
- `bonesdeploy` owns deriving the selected framework or Docker state from its
  canonical runtime configuration and passing that selection at init/update
  boundaries.
- Existing deployment scaffolding remains responsible for selecting deployment
  scripts; it is not duplicated in the pruning ruleset unless those paths later
  become part of complete materialization.
- The Python package has no responsibility for repository pruning.

### Affected areas

- `crates/bonesinfra/src/lib.rs`
- A focused BonesInfra module for the framework ruleset if keeping it in
  `lib.rs` would mix responsibilities or push the file beyond its size limit
- `crates/bonesinfra/tests/project_core.rs`
- `crates/bonesdeploy/src/commands/init/scaffold.rs`
- `crates/bonesdeploy/src/commands/update/mod.rs`
- `crates/bonesdeploy/tests/init.rs`
- Relevant README, context, and architecture documentation

### Decisions

- The wheel stays universal and byte-identical because Python framework code is
  an installation concern, not repository file structure.
- The ruleset lists removable paths rather than deriving ownership from path
  names. This makes deletion explicit and prevents accidental pruning of shared
  or project-owned files.
- Pruning follows complete materialization. This keeps one simple source
  artifact and naturally restores files when a project changes framework.
- Docker is represented by no selected framework, so the same loop removes all
  mapped framework paths without separate deletion logic.
- Unknown selected frameworks fail before pruning. Silently treating an unknown
  value as Docker could delete every framework path.

### Risks

- An incomplete ruleset can leave irrelevant framework files in a project.
- An incorrect ruleset path can delete shared or project-owned content. Exact
  path declarations and inventory tests mitigate this.
- Pruning before selection validation could partially delete a project for an
  invalid framework. Validation must happen before the first removal.
- Update ordering must keep the selected templates available before Python
  patches execute.

### Validation

- BonesInfra integration tests compare the materialized wheel bytes with the
  embedded universal wheel for every selection.
- Ruleset tests prove every supported native framework keeps its mapped paths,
  removes every other framework's paths, and retains shared templates.
- Docker tests prove every framework-owned path is removed while the wheel and
  shared templates remain.
- Repeated materialization and framework-change tests prove deleted paths are
  restored when selected and stale paths are removed after a selection change.
- Safety tests prove unknown frameworks fail before deletion and unmapped
  project-owned paths remain untouched.
- Init tests verify representative native, custom, and Docker project
  inventories.
- Existing non-E2E Rust and Python suites, formatting, Clippy, and shell
  formatting remain clean.
- Final diff review confirms there is no wheel transformation, Python artifact
  profile metadata, or pruning outside declared repository paths.


## Tasks

### Implementation

- [x] Add the BonesInfra framework ruleset mapping every supported framework to
  exact `infra/`-relative managed paths, initially its framework template
  directory.
- [x] Extend project artifact materialization to validate the selected
  framework, copy the universal wheel and complete template tree as today, then
  remove every unselected framework's mapped paths.
- [x] Pass the finalized native framework or Docker's no-framework selection
  from fresh init into BonesInfra materialization.
- [x] Reorder local update configuration loading so update rematerializes and
  prunes for the finalized runtime selection before applying Python patches.
- [x] Remove any obsolete assumptions that every project retains every
  framework template directory.

### Validation

- [x] Add BonesInfra tests proving native, custom, and Docker selections produce
  the expected template inventory while preserving byte-identical universal
  wheel output and shared files.
- [x] Add BonesInfra tests proving profile changes restore newly selected paths,
  remove stale framework paths, reject unknown frameworks before deletion, and
  preserve unmapped project-owned paths.
- [x] Update init tests to verify representative native, custom, and Docker
  project inventories without inspecting or modifying wheel contents.
- [x] Run focused BonesInfra and BonesDeploy tests, then all non-E2E Rust and
  Python tests.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`, addressing all reported
  warnings and errors.

### Completion

- [x] Update relevant README, context, and architecture documentation to explain
  universal-wheel materialization followed by repository path pruning.
- [x] Review the final diff for undeclared deletion paths, wheel transformation,
  stale generated artifacts, accidental changes, and incomplete ruleset
  coverage.

### Completion notes

Implemented as planned. `FRAMEWORK_PATHS` is the sole framework pruning
ruleset; materialization writes the unchanged universal wheel and complete
template snapshot before pruning exact unselected paths. Focused tests, the
non-E2E Rust workspace, 508 Python tests, Clippy, Rust formatting, Ruff, and
shell formatting passed. No E2E tests were run. The only unrelated source
change imports `std::process::Output` in an existing test to satisfy the
workspace Clippy warning policy.
