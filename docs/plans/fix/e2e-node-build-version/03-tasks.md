# Tasks

## Implementation

- [x] Replace an existing `NODE_VERSION` assignment in the E2E project helper
  without requiring an empty value.
- [x] Add focused tests for empty, defaulted, and missing Node-version entries.

## Validation

- [x] Run the E2E project-helper tests and `cargo test -p e2e`.
- [ ] After the local pinned builder image is available, run Next server/static
  and Nuxt server/static E2E scenarios serially. Not run per the task constraint
  against E2E scenarios and ignored Incus-backed tests.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without unresolved
  warnings or formatting changes.

## Completion

- [x] Review the final diff to confirm it contains only the helper repair,
  regression tests, and this durable plan.

## Completion notes

On 2026-09-29, the native E2E batch reached Next and Nuxt after Laravel failed
on a missing local builder image. Both Next and Nuxt failed before provisioning
because their generated `.env.build` files contained defaulted Node versions
rather than the empty assignment the E2E helper expected.

Implemented `SampleProject::pin_node_version` with line-aware replacement of the
existing `NODE_VERSION=` assignment. Other `.env.build` content is preserved,
and a missing assignment still returns an error without writing the file.
Added three focused unit tests covering empty, defaulted, and missing keys.

Validation completed:

- `cargo test -p e2e --lib project::tests`
- `cargo test -p e2e`
- `cargo test -p e2e --test fixture_consistency`
- `cargo fmt --all -- --check`
- `cargo clippy -p e2e --all-targets --all-features -- -D warnings`
- `shfmt -w .`

Next/Nuxt Incus-backed E2E scenarios remain unrun as explicitly requested.
