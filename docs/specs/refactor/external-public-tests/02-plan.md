# Plan

## Current behavior

The workspace has four Rust packages with source-file test modules:
`bonesdeploy-core`, `bonesdeploy`, `bonesremote`, and `bonesinfra`.
`bonesdeploy-core` and `bonesinfra` have library targets. `bonesdeploy` and
`bonesremote` define only binary targets in `src/main.rs`.

`bonesdeploy-core/src/lib.rs` publicly exposes the `config`, `env_build`, and
`paths` modules. Its 21 source-file tests in `app.rs`, `config.rs`,
`env_build.rs`, `paths.rs`, and `validation.rs` call that public API.

`bonesdeploy` has source-file tests for public CLI workflows and for private
parsers, rendering, asset embedding, and update implementation. Its init tests
currently use an in-process `TestEnvironment` that serializes and mutates the
test process's directory and XDG environment. Its push-state test exercises
private Git helpers directly.

`bonesremote` has source-file tests for CLI argument validation plus private
root-only host operations, internal state, and parsing. Its `commands::run`
path performs privilege checks, so only argument-validation behavior is a
portable command-level test.

`bonesinfra` has three source-file tests of the private `PythonSource` embed
type and private stamp function. It already has an external Python behavior
test at `crates/bonesinfra/tests/pytest.rs`.

## Intended behavior

Each selected public test is an integration test in its owning crate's
`tests/` directory. `bonesdeploy-core` tests link to its public library API.
`bonesdeploy` and `bonesremote` tests invoke their Cargo-built executables and
assert user-visible status, output, and filesystem or Git effects. Each
command test supplies its own temporary working directory and child-process
environment.

The source modules retain only private tests. The moved tests continue to
prove valid and invalid configuration, environment-build parsing, paths, init
workflow effects, config output, push-state Git effects, local doctor script
validation, and CLI argument requirements.

## Approach

Create integration-test files in each owning crate and migrate assertions by
their public boundary.

For `bonesdeploy-core`, move the existing tests into four files:
`tests/config.rs` covers config validation and `App` defaults;
`tests/env_build.rs` covers parsing and loading build environment files;
`tests/paths.rs` covers site target naming; and `tests/validation.rs` covers
project and script-name validation. Import only the crate's existing public
API.

For the binary packages, add small per-crate integration-test support modules
that resolve the binary path, run a command in a temporary directory, and
capture its output. `bonesdeploy` command tests set `HOME` and the XDG roots on
the child process, initialize temporary Git repositories where required, and
assert the current init, config, push-state, doctor, and CLI behaviors.
`bonesremote` command tests assert the doctor command's required-site
validation before command execution reaches root-only behavior.

Delete the migrated test modules and the `bonesdeploy` in-process test
environment harness. Retain mixed test modules with their private tests after
removing only their public test cases. No production API or target structure
changes are required.

## Responsibilities and boundaries

- `crates/bonesdeploy-core/tests/` owns verification of the
  `bonesdeploy-core` public library contract.
- `crates/bonesdeploy/tests/` owns verification of public `bonesdeploy` CLI
  workflows. Its shared test module owns only isolated child-process and Git
  setup needed by more than one command test.
- `crates/bonesremote/tests/` owns verification of public `bonesremote` CLI
  argument validation.
- Source modules continue to own tests that require their private functions,
  types, embedded resources, or test-only state seams.
- `src/main.rs` remains the binary entry point; no library boundary is added to
  either command-line package.
- Ignore `ASSERTIONS.md`, that is an auto-generated file. 

## Affected areas

- `crates/bonesdeploy-core/src/{app,config,env_build,paths,validation}.rs` and
  new `crates/bonesdeploy-core/tests/{config,env_build,paths,validation}.rs`.
- `crates/bonesdeploy/src/cli/args.rs`, `commands/{config,doctor,push_state}.rs`,
  and `commands/init/{mod,config,framework,tests}.rs`; new
  `crates/bonesdeploy/tests/{common,cli,config,doctor,init,push_state}.rs`.
- `crates/bonesremote/src/cli/args.rs` and new
  `crates/bonesremote/tests/{common,cli}.rs`.
- `bonesdeploy-core` gains a `tempfile` dev-dependency for its temporary-file
  integration tests; `bonesdeploy` already depends on `tempfile`, and
  `bonesremote` tests run the binary without temporary state.

## Decisions

- Classify tests by the asserted boundary, not by whether the current source
  function happens to be declared `pub`. This moves public CLI workflows from
  binary-only packages without exposing Rust internals.
- Use child processes for binary tests. This tests the real CLI boundary and
  eliminates the init suite's process-wide mutex and unsafe environment
  mutation.
- Do not move `bonesinfra` embed tests. The embedded tree and stamp are private
  representation details; its external Python suite already covers public
  behavior.
- Do not move `bonesremote` state, lifecycle, security, and host-operation
  tests. They require private types, test-only state overrides, or root-only
  operations that are not portable command-level behavior.
- Keep binary packages binary-only. Cargo integration tests can execute their
  compiled binary without a library target, avoiding a broad API refactor.
- No project documentation change is required because the documented developer
  commands and the workspace's top-level structure remain unchanged.

## Risks

- Command-level tests can fail to isolate home, XDG, Git identity, current
  directory, or temporary repository state. Per-child environment and explicit
  Git configuration prevent leakage and test-order dependence.
- CLI tests can assert presentation details too broadly. Assertions will focus
  on status, required arguments, requested values, and durable filesystem or
  Git effects.
- Removing mixed test modules can accidentally remove private coverage. Each
  edited module must retain its private tests and compile before the old module
  is deleted.
- `cleancode` scans source paths and file sizes, so moved files and removed
  test blocks can require its expected inventory to be updated.

## Validation

- Run each affected crate's test target. The external tests must prove all
  migrated library and command behaviors while remaining independent of the
  developer environment.
- Run the workspace test suite excluding the `e2e` package. All retained unit
  tests and new integration tests must pass together.
- Run `cargo clippy`, `cargo fmt`, and `shfmt -w .`, addressing every warning
  and error.
- Review the final diff to confirm that every moved public test is under its
  owning crate's `tests/` directory, no private implementation item gained
  visibility for tests, and no private test was removed.
