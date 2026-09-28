# Tasks

## Implementation

- [x] Make combined, server, and site setup help state their distinct scopes.
- [x] Make initialization output distinguish fresh-host and prepared-host next steps.
- [x] Align embedded agent docs and README workflows with once-per-host and once-per-project setup.
- [x] Add CLI integration coverage for rendered setup help and initialization guidance.

## Validation

- [x] Run focused BonesDeploy CLI and init tests successfully.
- [x] Inspect root, server setup, and site setup help for the required scope wording.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without warnings or errors.

## Completion

- [x] Review the final diff for consistent terminology and unintended changes.

## Completion notes

Implemented without changing setup orchestration or provisioning behavior.
`cargo test -p bonesdeploy --test cli --test init` passed 17 tests. Rendered
help distinguishes the combined fresh-host path from once-per-host server setup
and once-per-project site setup. `cargo fmt`, `cargo clippy`, `shfmt -w .`, and
`git diff --check` completed successfully. E2E tests were not run.
