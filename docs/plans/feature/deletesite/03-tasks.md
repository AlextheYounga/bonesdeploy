# Tasks

## Implementation

- [x] Replace BonesRemote begin/complete/verify/reactivate transitions with one
  safe, idempotent site-registration removal command.
- [x] Remove deletion plan/tombstone fields and deletion-specific mutation
  blocking from BonesRemote state.
- [x] Make BonesDeploy execute the current preflight directly and remove the
  registration only after successful BonesInfra teardown.
- [x] Remove setup reactivation and restore ordinary fresh setup behavior.
- [x] Remove obsolete verified-tombstone doctor handling and tests.

## Validation

- [x] Add registration-removal tests for success, missing state, invalid names,
  symlinks, root boundaries, sibling lock preservation, and legacy state.
- [x] Update BonesDeploy, BonesRemote CLI, mutation, setup, and deletion tests for
  the direct teardown flow.
- [x] Run full non-E2E Rust and Python suites plus required lint and format checks.

## Completion

- [x] Update README, CONTEXT, and architecture documentation to remove the
  decommission/tombstone model.
- [x] Review the final diff for obsolete state-machine code, unsafe deletion,
  generated artifact drift, and unrelated changes.

## Completion Notes

Implemented as a direct idempotent teardown. The non-E2E Rust suite, embedded
Python suite, Clippy, rustfmt, Ruff, and shfmt checks pass.
