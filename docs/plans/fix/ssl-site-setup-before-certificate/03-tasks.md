# Tasks

## Implementation

- [ ] Remove `ssl_enabled` from local configuration, provisioning transport, Python request/context, and SSL persistence.
- [ ] Determine TLS router eligibility from the remote certificate pair before router setup renders nginx.
- [ ] Derive manifest and next-step SSL state from remote certificate files.
- [ ] Cover absent, incomplete, and complete certificate states in focused configuration and runtime nginx tests.
- [ ] Regenerate the embedded BonesInfra wheel after Python source changes.

## Validation

- [ ] Run the focused runtime nginx test module and verify both certificate states.
- [ ] Run the Python test suite, Ruff check, and Ruff formatter.
- [ ] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`.

## Completion

- [ ] Review the diff, update completion notes, and confirm no unrelated changes are included.

## Completion notes

No implementation work has begun.
