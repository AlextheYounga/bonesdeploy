# Tasks

## Implementation

- [x] Remove Cloudflared installation, setup, start, and removal from normal
  runtime orchestration and SSL provisioning.
- [x] Remove Quick Tunnel ownership from required runtime manifests and obsolete
  release-target restart filtering.
- [x] Add a loopback-only nginx origin route and update the Cloudflared unit to
  use a supported HTTP URL outside the site target.
- [x] Implement explicit BonesInfra tunnel start/stop provisioning with package
  installation, service ownership, restart policy, cleanup, and etckeeper.
- [x] Add `bonesdeploy site tunnel start|stop|status` following existing site
  command, prompt, request, SSH status, and output conventions.
- [x] Change domainless setup guidance to advertise the optional tunnel command
  without starting it.

## Validation

- [x] Add focused Python tests for normal-runtime separation, loopback origin,
  independent service lifecycle, cleanup, manifest behavior, and SSL behavior.
- [x] Add Rust CLI/status tests for the explicit command surface and lifecycle
  messages.
- [x] Run focused tests and the full non-E2E Python suite.
- [x] Run `ruff check .`, `ruff format .`, `cargo clippy`, `cargo fmt`, and
  `shfmt -w .` and address all findings.

## Completion

- [x] Update user, architecture, context, and prior tunnel plan/task
  documentation to describe the opt-in lifecycle and loopback origin.
- [x] Review the final diff for obsolete automatic behavior, accidental files,
  and preservation of `tests/cleancode/Cargo.toml`.

## Completion Notes

The opt-in lifecycle and supported loopback HTTP origin are implemented. Focused
Rust tests and the full 507-test BonesInfra Python suite pass, as do formatting
and lint checks. The generated BonesInfra wheel was rebuilt after the Python
changes. End-to-end and manual server validation were not run.
