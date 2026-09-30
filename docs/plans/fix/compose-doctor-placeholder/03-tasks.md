# Tasks

## Implementation

- [x] Classify a canonical placeholder `current` release as pending in
  `check_docker_runtime` before active Compose configuration validation.
- [x] Add a focused remote-doctor regression test covering a placeholder
  release for a Docker-backed site.

## Validation

- [x] Run the focused doctor regression and `cargo test -p bonesremote`; confirm
  placeholder handling does not relax invalid active Compose release checks.
- [ ] Run the ignored `docker_compose` E2E scenario serially; verify site setup
  and the complete Compose deployment lifecycle pass.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without unresolved
  warnings or formatting changes.

## Completion

- [x] Review the final diff to confirm it contains only the Compose doctor
  classification, its regression test, and this durable plan.

## Completion notes

On 2026-09-30, Compose doctor now reuses `services::current_is_placeholder` to
classify the canonical placeholder current release as pending before active
Compose validation. The focused `bonesremote` regression covers both the
placeholder and a named deployed release. The ignored Incus-backed E2E scenario
was not run per request.

Validation results:

- `cargo test -p bonesremote` passed.
- `cargo fmt` completed without changes.
- `cargo clippy --workspace --all-targets --all-features` passed.
- `shfmt -w .` completed without changes.
