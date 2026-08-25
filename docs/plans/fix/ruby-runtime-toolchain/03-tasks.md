# Tasks

## Implementation

- [x] Define the supported exact Ruby releases and add the verified host installer.
- [x] Make BonesInfra install and return the versioned Ruby executable.
- [x] Add cache-local Ruby installation and activation for Rails builds.
- [x] Change Rails selection and the Rails E2E fixture to an exact Ruby version.
- [x] Add focused Python and Rust regression coverage.

## Validation

- [x] Run focused Python and Rust tests for Ruby and Rails configuration behavior.
- [x] Run Ruff, Python formatting, Cargo formatting, Clippy, and shell formatting.

## Completion

- [x] Update applicable runtime documentation and review the final diff.

## Completion notes

Legacy `X.Y` Rails configuration remains supported by resolving it to the
corresponding pinned release; new projects write exact versions.

`uv run ruff check .`, `uv run pytest` (403 tests), `cargo fmt`, `cargo clippy`,
`shfmt -w .`, `cargo test -p bonesdeploy --test assets --test config_frameworks`,
and `cargo test -p bonesremote --test release` passed. Full E2E was not run.
