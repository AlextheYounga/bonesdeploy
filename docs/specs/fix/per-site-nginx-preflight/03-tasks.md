# Tasks

## Implementation

- [x] Change the pre-cut-over nginx command to test the derived site nginx configuration.
- [x] Pass the registered site identifier from deployment lifecycle to the preflight command.
- [x] Add regression coverage for the derived per-site nginx configuration path.
- [x] Document the per-site nginx pre-cut-over gate.

## Validation

- [x] Run the bonesremote test suite.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`.

## Completion

- [x] Review the final diff and record validation results.

## Completion notes

Implemented the explicit per-site nginx preflight with the existing Phase A
gate. `cargo test -p bonesremote`, `cargo fmt`, `cargo clippy`, `shfmt -w .`,
and `git diff --check` passed. End-to-end tests were not run.
