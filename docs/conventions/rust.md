# Rust Testing

## Test Placement

All Rust test code belongs in the owning crate's crate-root `tests/` directory.
Production files under `src/` must not contain `#[cfg(test)]` modules, test
functions, test fixtures, test assertions, or test-only seams.

- Test library behavior through deliberate public APIs.
- Test observable command-line behavior by executing the compiled binary.
- Organize larger suites with modules below `tests/`; only Rust files directly
  inside `tests/` are compiled automatically as integration-test targets.
- Keep shared integration-test support under `tests/common/` or in a focused
  support module below `tests/`.

The crate-root `tests/` directory is Cargo's integration-test directory. It is
different from workspace-level test packages such as `tests/cleancode`.

## Testable Boundaries

Tests assert public library behavior or an observable CLI contract. Observable
CLI behavior includes:

- accepted and rejected arguments;
- exit status;
- standard output and standard error;
- filesystem effects; and
- Git or other external-system effects.

Do not reproduce production algorithms in integration tests. When important
behavior cannot be tested through an existing boundary, restructure the
production code around a deliberate, cohesive library API. Do not expose an
arbitrary internal helper merely so a test can call it.

## Binary Packages

Keep binary entry points thin. Put reusable parsing, orchestration, and domain
behavior in the package's library target so integration tests and the binary
use the same public boundary. Execute the compiled binary through Cargo's
`CARGO_BIN_EXE_<name>` environment variable when testing the command-line
contract itself.

Command tests should:

- use a temporary working directory;
- provide isolated `HOME` and relevant XDG directories;
- create explicit Git repositories and remotes when needed;
- capture and assert status, output, and durable effects; and
- avoid mutating the integration test process's current directory or global
  environment.

Shared command-test support belongs under that package's `tests/` directory. It
should contain only setup and execution behavior shared by multiple test
targets.

## API And Fixtures

Do not promote arbitrary private items to `pub` solely for test access. Extract
a public API only when it represents behavior that callers can meaningfully
use. Do not add a test-support feature or workspace-level test crate when
Cargo's standard integration-test boundary is sufficient.

Use fixtures and factories when they make setup clearer. Keep fixtures focused
on the behavior under test and isolate external state so tests remain
repeatable and independent.

Test observable behavior rather than private methods. A test that cannot be
written through the public boundary is evidence that the production boundary
needs deliberate redesign or that the assertion is coupled to an
implementation detail and should be removed.

## Validation

Run the affected crate tests and the relevant workspace tests after changing
test placement. Run the repository's standard formatting and lint checks.
Do not run the `e2e` package as part of ordinary local validation unless the
task explicitly requires it.
