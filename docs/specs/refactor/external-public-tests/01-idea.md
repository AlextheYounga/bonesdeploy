# Idea

## Request

Move public tests out of Rust production files into the `tests/` directory
inside each crate. Keep private tests in their current source files.

## Problem

Production files currently mix application code with test bodies. The mixed
files are harder to read and navigate. Moving tests that prove a crate's
external behavior into Cargo integration-test directories separates that
behavioral verification from implementation code without exposing private
implementation details.

## Definitions

**Public test:** A test whose assertion is expressible through an existing
public library API or through a package's observable command-line behavior.
For a command-line package, command arguments, exit status, standard output,
standard error, and filesystem or Git effects are public behavior.

**Private test:** A test that calls a private or crate-private item, inspects
an internal representation, or requires a `cfg(test)`-only seam. Private tests
remain unit tests in the source module that owns the behavior.

**Integration-test directory:** The crate-root `tests/` directory recognized
by Cargo. Each Rust file there is compiled as a separate crate and may access
only the tested library's public API. It may also execute that package's
compiled binary through Cargo's `CARGO_BIN_EXE_<name>` environment variable.
It is distinct from the workspace `tests/cleancode` structural-lint crate.

## Desired outcome

The public behaviors currently covered by selected source-file tests are
covered by executable tests under their owning crates' `tests/` directories.
Their former production modules contain no moved test bodies. Private tests
continue to cover private helpers and internal representations in place. The
same public library and CLI behaviors remain verified by `cargo test`.

## Scope

The change includes:

- moving all `bonesdeploy-core` source-file tests to
  `crates/bonesdeploy-core/tests/`, because they exercise its existing public
  library API;
- moving `bonesdeploy` tests for CLI syntax, `config`, `init`, `push-state`,
  and local deployment-script validation to `crates/bonesdeploy/tests/` as
  command-level tests;
- moving `bonesremote` doctor argument validation to
  `crates/bonesremote/tests/` as a command-level test;
- removing only the corresponding test bodies and test-only in-source harness
  code from production modules; and
- preserving the existing test assertions' user-visible behavior, including
  init's filesystem and configuration effects and push-state's remote Git
  update.

## Constraints

- External tests belong to the `tests/` directory of the crate they exercise.
- Private items remain private; this work does not promote implementation
  details to `pub` or `pub(crate)` for test access.
- `bonesdeploy` and `bonesremote` remain binary-only packages. Their external
  tests execute the binaries rather than requiring a library target.
- The tests use Cargo's binary path environment variables and isolated child
  process environments instead of mutating the test process's working
  directory or environment.
- The existing `e2e` suite is not run.

## Exclusions

- Private-helper, embedded-asset, state-store, lifecycle, parsing, and
  internal-rendering tests remain source-file unit tests.
- The `bonesinfra` embed tests remain in `crates/bonesinfra/src/lib.rs`; its
  existing Python behavior test remains in `crates/bonesinfra/tests/pytest.rs`.
- No new public Rust API, binary-to-library split, test-support feature, or
  workspace-level test crate is introduced.
- No production behavior, CLI contract, or embedded assets are changed.
