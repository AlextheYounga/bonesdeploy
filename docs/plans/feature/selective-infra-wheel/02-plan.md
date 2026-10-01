# Plan

## Current behavior

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

## Intended behavior

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

## Approach

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

## Responsibilities and boundaries

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

## Affected areas

- `crates/bonesinfra/src/lib.rs`
- A focused BonesInfra module for the framework ruleset if keeping it in
  `lib.rs` would mix responsibilities or push the file beyond its size limit
- `crates/bonesinfra/tests/project_core.rs`
- `crates/bonesdeploy/src/commands/init/scaffold.rs`
- `crates/bonesdeploy/src/commands/update/mod.rs`
- `crates/bonesdeploy/tests/init.rs`
- Relevant README, context, and architecture documentation

## Decisions

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

## Risks

- An incomplete ruleset can leave irrelevant framework files in a project.
- An incorrect ruleset path can delete shared or project-owned content. Exact
  path declarations and inventory tests mitigate this.
- Pruning before selection validation could partially delete a project for an
  invalid framework. Validation must happen before the first removal.
- Update ordering must keep the selected templates available before Python
  patches execute.

## Validation

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
