# Tasks

## Implementation

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

## Validation

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

## Completion

- [x] Update relevant README, context, and architecture documentation to explain
  universal-wheel materialization followed by repository path pruning.
- [x] Review the final diff for undeclared deletion paths, wheel transformation,
  stale generated artifacts, accidental changes, and incomplete ruleset
  coverage.

## Completion notes

Implemented as planned. `FRAMEWORK_PATHS` is the sole framework pruning
ruleset; materialization writes the unchanged universal wheel and complete
template snapshot before pruning exact unselected paths. Focused tests, the
non-E2E Rust workspace, 508 Python tests, Clippy, Rust formatting, Ruff, and
shell formatting passed. No E2E tests were run. The only unrelated source
change imports `std::process::Output` in an existing test to satisfy the
workspace Clippy warning policy.
