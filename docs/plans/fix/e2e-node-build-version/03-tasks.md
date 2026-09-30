# Tasks

## Implementation

- [ ] Replace an existing `NODE_VERSION` assignment in the E2E project helper
  without requiring an empty value.
- [ ] Add focused tests for empty, defaulted, and missing Node-version entries.

## Validation

- [ ] Run the E2E project-helper tests and `cargo test -p e2e`.
- [ ] After the local pinned builder image is available, run Next server/static
  and Nuxt server/static E2E scenarios serially.
- [ ] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without unresolved
  warnings or formatting changes.

## Completion

- [ ] Review the final diff to confirm it contains only the helper repair,
  regression tests, and this durable plan.

## Completion notes

On 2026-09-29, the native E2E batch reached Next and Nuxt after Laravel failed
on a missing local builder image. Both Next and Nuxt failed before provisioning
because their generated `.env.build` files contained defaulted Node versions
rather than the empty assignment the E2E helper expected. No implementation
change has been made pending human review and approval of this plan.
