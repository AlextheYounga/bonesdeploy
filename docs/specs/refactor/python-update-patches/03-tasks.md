# Tasks

## Implementation

- [x] Add the Python patch registry with `0001-config-repo` and
  `0002-root-config-repo`, `0.7.3` selection, prerelease normalization, and
  atomic per-scope marker writes.
- [x] Implement local Python config-repository patches that initialize `.bones`,
  preserve `0001`'s unexpected-origin rejection, and apply `0002`'s root-owned
  origin migration.
- [x] Implement the remote pyinfra config-repository patch plan, including
  legacy repository migration, canonical bare repository configuration,
  root ownership, pre-receive hook installation, and remote markers.
- [x] Add the private `bonesinfra patches apply` command and the runner's
  explicit SSH-user override so remote patch plans connect as root.
- [x] Replace Rust update patch calls with the private BonesInfra command while
  retaining local/remote update sequencing, remote release download, and
  project-root preparation.
- [x] Delete the Rust update patch modules and the `bonesremote patch` command,
  its argument definitions, dispatch, and command-module registration.
- [x] Update update-patch ownership documentation in `CONTEXT.md` and
  `crates/bonesinfra/python/CONTEXT.md`.

## Validation

- [x] Add and run focused Python tests for registry selection, local Git
  migration, marker retry behavior, remote pyinfra plan, CLI scope dispatch,
  and root SSH override.
- [x] Run affected Rust tests to prove update compilation and behavior after
  removal of the Rust patch boundaries.
- [x] Run `ruff check .`, `ruff format .`, and `uv run pytest` from
  `crates/bonesinfra/python`.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without running
  end-to-end tests.

## Completion

- [x] Review the final diff for a single Python-owned patch implementation,
  preserved marker compatibility, removed Rust patch code, and accurate
  documentation.

## Completion notes

Implementation is complete. Python owns patch selection, local Git migration,
remote pyinfra migration, and completion markers. The Rust update command only
coordinates the private embedded BonesInfra command, and the obsolete Rust
patch implementations and `bonesremote patch` command are removed.

Validation completed:

- `uv run pytest`: 302 passed.
- Focused Python patch, runner, and cleancode tests: 77 passed.
- `cargo test -p bonesdeploy --bin bonesdeploy`: 61 passed.
- `cargo test -p bonesremote`: 106 passed.
- `cargo clippy --all-targets --all-features -- -D warnings`: passed.
- `cargo fmt --all`: passed.
- `shfmt -w .`: passed.

The broader `cargo test -p bonesdeploy -p bonesremote` run was not used as the
completion gate because an existing `doctor` integration test attempted an
interactive GPG signing operation and failed with `Operation cancelled`; no
feature test failed. End-to-end tests were not run.
