# Tasks

## Implementation

- [x] Create `bonesdeploy-core` integration tests for the existing public
  config, app-default, environment-build, path, and validation assertions, and
  remove their five source-file test modules.
- [x] Create isolated `bonesdeploy` command-test support that runs the compiled
  binary with a temporary working directory, temporary HOME and XDG roots, and
  explicit Git repository setup.
- [x] Move `bonesdeploy` public CLI argument, config output, init workflow,
  push-state, and local doctor deployment-script assertions into command-level
  integration tests; remove the corresponding source tests and the in-process
  init test environment harness while retaining private unit tests.
- [x] Create `bonesremote` command-test support and move doctor required-site
  argument validation into `crates/bonesremote/tests/cli.rs`; remove its
  source-file test module.
- [x] Update source-scanning assertions and the generated assertion inventory
  for the moved test paths and changed production-file sizes.

## Validation

- [x] Run the `bonesdeploy-core`, `bonesdeploy`, and `bonesremote` test targets
  and verify each migrated external test passes.
- [x] Run workspace tests without the `e2e` package and verify retained private
  unit tests pass with the new integration tests.
- [x] Run `cargo clippy`, `cargo fmt`, and `shfmt -w .`; resolve all reported
  warnings and errors.
- [x] Run the `cleancode` package tests and verify the updated source inventory
  passes.

## Completion

- [x] Review the final diff to confirm public tests are in their owning crate's
  `tests/` directory, private tests remain in source, and no production item
  was exposed solely for testing.

## Completion notes

Implementation complete. All workspace tests (excluding the `e2e` package),
`cargo clippy`, `cargo fmt`, `shfmt -w .`, and the `cleancode` package pass with
no warnings. No production API or target structure changed; `bonesdeploy` and
`bonesremote` remain binary-only and their external tests run the compiled
binaries with isolated child-process environments. `bonesdeploy-core` gained a
`tempfile` dev-dependency for its temporary-file integration tests.
