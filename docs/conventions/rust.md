# Rust Testing

## Test Placement

Put tests next to the boundary they verify.

- Put tests of private or crate-private implementation details in the source
  module that owns those details, usually under `#[cfg(test)] mod tests`.
- Put tests of an existing public library API in the owning crate's
  crate-root `tests/` directory.
- Put tests of observable command-line behavior in the owning package's
  `tests/` directory, even when the package has no library target.
- Keep tests for embedded assets, internal representations, private parsers,
  and test-only seams in source modules when they cannot be expressed through
  the public boundary.

The crate-root `tests/` directory is Cargo's integration-test directory. It is
different from workspace-level test packages such as `tests/cleancode`.

## Public And Private Tests

A **public test** is expressible through an existing public library API or an
observable CLI contract. Observable CLI behavior includes:

- accepted and rejected arguments;
- exit status;
- standard output and standard error;
- filesystem effects; and
- Git or other external-system effects.

A **private test** calls private or crate-private items, inspects internal
representation, or depends on a `cfg(test)`-only seam. Keep it in the source
module that owns the behavior.

Classify a test by the boundary it asserts, not by whether the implementation
function currently happens to be declared `pub`.

## Binary Packages

Do not split a binary-only package into a library solely to make its tests
callable. Integration tests should execute the compiled binary through Cargo's
`CARGO_BIN_EXE_<name>` environment variable and assert its observable result.

Command tests should:

- use a temporary working directory;
- provide isolated `HOME` and relevant XDG directories;
- create explicit Git repositories and remotes when needed;
- capture and assert status, output, and durable effects; and
- avoid mutating the integration test process's current directory or global
  environment.

Shared command-test support belongs in that package's `tests/common.rs`. It
should contain only setup and execution behavior shared by multiple test
targets.

## API And Fixtures

Do not promote private items to `pub` or `pub(crate)` solely for test access.
Do not add a test-support feature or workspace-level test crate when Cargo's
standard integration-test boundary is sufficient.

Use fixtures and factories when they make setup clearer. Keep fixtures focused
on the behavior under test and isolate external state so tests remain
repeatable and independent.

Test observable behavior rather than private methods. A test that cannot be
written through the public boundary is evidence that it belongs as a unit test
or that the production boundary needs deliberate redesign, not accidental
visibility.

## Validation

Run the affected crate tests and the relevant workspace tests after changing
test placement. Run the repository's standard formatting and lint checks.
Do not run the `e2e` package as part of ordinary local validation unless the
task explicitly requires it.
