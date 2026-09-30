# Tasks

## Implementation

- [ ] Add `crates/bonesdeploy/src/build/local_lock.rs` with `LocalBuildLock`, site validation before lock-path derivation, the per-site build-cache lock file, non-blocking advisory acquisition, site-naming diagnostics, and RAII release.
- [ ] Register `local_lock` in `crates/bonesdeploy/src/build/mod.rs` and acquire the guard at the beginning of `build::package`, retaining it through source export, native or Compose execution, and artifact packaging.
- [ ] Preserve the existing native cache path in `crates/bonesdeploy/src/build/native.rs` and the stable Compose project name in `crates/bonesdeploy/src/build/compose.rs`; do not add backend-specific lock ownership.
- [ ] Add `crates/bonesdeploy/tests/local_build_lock.rs` as a crate-root integration test with process helpers that expose same-site contention and concurrent different-site progress through the package operation.

## Validation

- [ ] Verify a same-site helper process holding `LocalBuildLock` causes a second package attempt to fail immediately and report the site name.
- [ ] Verify different-site helper processes acquire separate locks and both complete without contention errors.
- [ ] Verify the first helper's lock remains held through artifact packaging rather than ending after source export or backend execution.
- [ ] Run focused `bonesdeploy` Rust tests, excluding e2e tests, and confirm the lock integration tests pass.
- [ ] Run `cargo fmt --check`, workspace `cargo clippy`, and `shfmt -w .`; address every warning or error.

## Completion

- [ ] Review the final implementation diff for lock-path traversal, site aliasing, blocking acquisition, stale-PID logic, global locking, remote-lock coupling, and a guard lifetime that ends before artifact packaging.
- [ ] Confirm the implementation changes only the listed build and crate-root test files and that no additional documentation is required beyond this planning record.

## Completion notes

Implementation and validation have not started. Completion notes will record material deviations, validation evidence, discoveries, and deliberately unfinished work after implementation.
