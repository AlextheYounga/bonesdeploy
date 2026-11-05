# Tasks

## Implementation

- [x] Replace cache-source extraction with atomic complete-core materialization
  and project-scoped dependency environment preparation.
- [x] Require local core in the Python loader, preserve core/custom composition,
  and remove the framework-only materialization CLI.
- [x] Materialize complete core during init and remove framework-core update
  synchronization.
- [x] Re-exec update after local installation, materialize core before patches,
  and retain deployment asset refresh ordering.
- [x] Migrate legacy project-owned `.bones` content into custom provisioning
  while preserving pre-materialized core and rejecting destination collisions.
- [x] Correct affected architecture, context, README, and superseded plan text.

## Validation

- [x] Add and run focused Python tests for mandatory local core, composition,
  and legacy migration.
- [x] Add and run focused Rust tests for complete materialization, preservation
  of custom provisioning, init layout, and deployment-only synchronization.
- [x] Run package, workspace, formatting, lint, and shell formatting checks
  without e2e tests.

## Completion

- [x] Review the final diff for executable cached source, built-in fallback,
  and obsolete framework-only synchronization.

## Completion notes

- Replaced cache checkout extraction with project-local complete distribution
  materialization. Cached environments use the package version from
  `pyproject.toml` and a readable project-path cache location.
- Validation passed: `ruff check .`, `ruff format .`, `uv run pytest`,
  `cargo test --workspace --exclude e2e`, `cargo clippy --workspace --exclude
  e2e --all-targets --all-features -- -D warnings`, `cargo fmt`, and
  `shfmt -w .`.
