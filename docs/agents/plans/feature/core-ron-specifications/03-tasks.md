# Tasks

## Implementation

- [x] Name every top-level and nested RON struct value after its corresponding
  Rust struct so no specification object uses anonymous `(...)` syntax.
- [x] Add a short repository convention requiring named RON struct values.
- [x] Add the `ron` dependency and the Core specifications module that embeds,
  deserializes, and exposes the five typed RON documents with contextual errors.
- [x] Create the topic-oriented RON documents for all current Core static paths
  and application, runtime, build, service, and release-permission defaults.
- [x] Migrate `paths.rs` static values to the typed paths specification while
  preserving every existing project- and site-specific derivation result.
- [x] Migrate `App`, `Runtime`, `Build`, and `Services` defaults to the typed
  specifications and replace untyped `Runtime.permissions` with typed release
  permission rules that serialize to the current project TOML representation.
- [x] Update Core, `bonesdeploy`, and `bonesremote` consumers to use the
  specification-backed paths and defaults without retaining duplicate shared
  infrastructure values.
- [x] Remove the static kit `bones.toml` and update fresh initialization so the
  existing configuration-save boundary solely writes the generated project
  configuration while other kit assets remain unchanged.

## Validation

- [x] Run the embedded Core specification tests after introducing named RON
  struct syntax.
- [x] Re-run `cargo fmt`, `cargo clippy`, and `shfmt -w .` with no remaining
  warnings or errors.
- [x] Add Core tests that load every embedded RON document and assert migrated
  defaults, representative derived paths, and default release permission rules.
- [x] Update initialization tests to assert a fresh project receives the
  existing TOML defaults and typed inline release-permission entries without a
  kit `bones.toml` asset.
- [x] Run focused Core, `bonesdeploy`, and `bonesremote` tests, then the
  workspace test suite excluding end-to-end tests.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` with no remaining
  warnings or errors.

## Completion

- [x] Update `README.md` where necessary to distinguish embedded Core RON
  defaults from the generated project `bones.toml`.
- [x] Review the final diff for removed duplicate Core constants and defaults,
  preserved asset behavior, and scope compliance.

## Completion notes

Implementation completed in `feature/core-ron-specifications`.

Validation completed:

- `cargo test --workspace --exclude e2e`
- `cargo check -p e2e` (compile-only; e2e tests were not run)
- `cargo fmt --check`
- `cargo clippy --workspace --exclude e2e --all-targets`
- `shfmt -w .`

The path accessor implementation was split into `src/paths/mod.rs` and
`src/paths/values.rs` to satisfy the repository's 400-line source-file limit.

After clarification, every RON struct value was changed from anonymous syntax
to its corresponding Rust type name. `cargo test -p bonesdeploy-core specs`,
`cargo fmt`, `cargo clippy --workspace --exclude e2e --all-targets`, and
`shfmt -w .` all completed successfully.
