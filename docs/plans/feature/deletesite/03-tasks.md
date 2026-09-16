# Tasks

## Implementation

- [x] Persist opaque deletion plans, decommissioning state, and verified or
  unverified tombstones in the atomic BonesRemote site store.
- [x] Add non-interactive `decommission begin`, `complete`, and `verify` CLI
  transitions that reuse persisted plans and fail closed.
- [x] Reject decommissioning and tombstoned sites from every `SiteMutation`
  acquisition.
- [x] Add `bonesdeploy site delete` with manifest preflight, exact-name
  confirmation, persisted-plan execution, and decommission transitions.
- [x] Add validated BonesInfra deletion plans for declared artifacts and systemd
  services, with ordered idempotent removal.
- [x] Reactivate a verified tombstone only after successful site setup.

## Validation

- [x] Add focused BonesRemote tests for plan reuse, fail-closed mutation
  rejection, complete cleanup, and verification.
- [x] Run the BonesRemote package tests, `cargo clippy`, `cargo fmt`, and
  `shfmt -w .` without executing E2E tests.
- [x] Add focused CLI parsing and BonesInfra persisted-plan and deletion-order
  tests.

## Completion

- [x] Update command and architecture documentation for destructive deletion,
  retained local state, persisted plans, and setup reactivation.

## Completion notes

Implemented the public command, validated manifest plan, durable remote state,
and setup reactivation. The remote protocol uses durable atomic state between
independent CLI invocations rather than a lock-holding coordinator stream.
Focused Rust and Python checks and workspace clippy passed. E2E tests were not
run. The full Python suite has one environment-dependent etckeeper failure:
`test_record_script_fails_without_etckeeper` exits successfully because this
machine has etckeeper available.
