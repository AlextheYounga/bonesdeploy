# Distribution Python Runtime Tasks

## Implementation

- [x] Replace production CPython source compilation with distribution Python and
  native dependency build packages installed through APT.
- [x] Remove managed Python runtime paths and the obsolete source installer.
- [x] Remove local Django dependency outputs from artifacts and create the target
  virtualenv before installing production requirements during prepare.
- [x] Drop `python_version` from new remote descriptors while reading and
  discarding descriptors that previously included it.

## Validation

- [x] Update focused Python and Rust coverage for provisioning, AppArmor,
  artifacts, prepare ordering, and descriptor compatibility.
- [x] Rebuild the embedded BonesInfra wheel and run all required non-E2E checks.

## Completion

- [x] Update runtime and deployment documentation for distribution Python,
  target-side pip installation, and host-version compatibility.
- [x] Review the final diff for stale managed-Python paths and unrelated changes.

## Completion Notes

Production Django now uses distribution Python from APT. Local builds do not
install Python or dependencies and remove stale dependency output from artifacts.
Prepare creates a release-owned virtualenv before installing requirements with
isolated, cacheless pip. Migration skip handling still occurs only after
dependencies and validation are ready.

The embedded wheel was rebuilt. Ruff checks, 516 Python tests, focused runtime
tests, the workspace Rust suite excluding E2E, Clippy, Rustfmt, Shfmt, and
`git diff --check` passed. Full E2E was not run.
