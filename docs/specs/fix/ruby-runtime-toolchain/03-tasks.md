# Tasks

## Implementation

- [x] Replace host Ruby source compilation with distribution Ruby, Bundler, and
  native gem build packages installed through PyInfra's APT operation.
- [x] Use distribution Ruby and Bundler paths throughout Rails runtime setup,
  placeholder installation, validation, systemd, and AppArmor configuration.
- [x] Remove the local build's `vendor/bundle` before artifact packaging while
  retaining precompiled assets.
- [x] Install the production bundle on the staged target release before running
  migrations, and remove the obsolete managed-Ruby environment projection.
- [x] Update focused Python and Rust regression coverage for the new host package,
  executable-path, artifact, and prepare behavior.

## Validation

- [x] Run focused Python and Rust tests for Ruby and Rails runtime, build, and
  prepare behavior.
- [x] Run the non-E2E Python and Rust suites, Ruff, Python formatting, Cargo
  formatting, Clippy, shell formatting, and `git diff --check` without errors.

## Completion

- [x] Explain distribution Ruby, host compatibility, target bundle installation,
  and deployment network/build requirements in the README.
- [x] Update applicable architecture/context documentation and review the final
  diff for stale managed-Ruby paths or source-install behavior.

## Completion notes

The earlier exact source-built production Ruby implementation was superseded by
the distribution Ruby decision recorded in `04-distribution-ruby-clarity.md`.
Rails setup now installs distribution Ruby and native gem build packages through
APT. Local Rails builds discard `vendor/bundle`, and remote prepare installs the
target bundle even when migrations are explicitly skipped.

`uv run ruff check .`, `uv run ruff format --check .`, and `uv run pytest` (518
tests) passed. `cargo test --workspace --exclude e2e`, `cargo clippy`, `cargo
fmt`, `shfmt -w .`, focused Rails/Ruby tests, and `git diff --check` passed. The
embedded BonesInfra wheel was rebuilt. Full E2E was not run.
