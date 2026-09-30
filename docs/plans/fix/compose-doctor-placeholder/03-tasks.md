# Tasks

## Implementation

- [ ] Classify a canonical placeholder `current` release as pending in
  `check_docker_runtime` before active Compose configuration validation.
- [ ] Add a focused remote-doctor regression test covering a placeholder
  release for a Docker-backed site.

## Validation

- [ ] Run the focused doctor regression and `cargo test -p bonesremote`; confirm
  placeholder handling does not relax invalid active Compose release checks.
- [ ] Run the ignored `docker_compose` E2E scenario serially; verify site setup
  and the complete Compose deployment lifecycle pass.
- [ ] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` without unresolved
  warnings or formatting changes.

## Completion

- [ ] Review the final diff to confirm it contains only the Compose doctor
  classification, its regression test, and this durable plan.

## Completion notes

On 2026-09-29, the serialized E2E matrix stopped at the first `docker_compose`
scenario. The Incus container, server setup, and site provisioning completed;
`site doctor` then rejected the expected placeholder release because it lacks a
Compose file. No implementation change has been made pending human review and
approval of this plan.
