# Tasks

## Implementation

- [x] Add the top-level `build` CLI variant and dispatch it to the build-only
  command module.
- [x] Implement the build-only workflow by loading the root configuration,
  calling `build::package`, and reporting the revision and artifact size without
  invoking deployment operations.
- [x] Add focused tests proving successful local packaging and packaging-failure
  propagation without SSH, secret publication, control-plane synchronization,
  or upload.

## Validation

- [x] Run focused BonesDeploy build-command tests and confirm a successful build
  reaches artifact packaging while a missing configured branch fails locally.
- [x] Run full non-E2E Rust and Python test suites, `cargo clippy`, `cargo fmt`,
  `shfmt -w .`, Python Ruff checks, and `git diff --check` with no warnings or
  failures.

## Completion

- [x] Document `bonesdeploy build` as local-only deployment build verification
  in the README.
- [x] Review the final diff for duplicated build behavior, accidental remote
  calls, persistent artifact output, and unrelated changes.

## Completion notes

`bonesdeploy build` calls the existing `build::package` boundary and reports the
artifact revision and size. It does not invoke deployment operations. The full
non-E2E Rust suite and Python suite passed, alongside Clippy, Rustfmt, Shfmt,
Ruff, and `git diff --check`. The embedded BonesInfra wheel was rebuilt after
the Laravel shared-directory change.
