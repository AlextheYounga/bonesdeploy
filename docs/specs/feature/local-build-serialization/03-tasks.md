# Tasks

## Implementation

- [x] Add `crates/bonesdeploy/src/build/local_lock.rs` with `LocalBuildLock`, site validation before lock-path derivation, the per-site build-cache lock file, non-blocking advisory acquisition, site-naming diagnostics, and RAII release.
- [x] Register `local_lock` in `crates/bonesdeploy/src/build/mod.rs` and acquire the guard at the beginning of `build::package`, retaining it through source export, native or Compose execution, and artifact packaging.
- [x] Preserve the existing native cache path in `crates/bonesdeploy/src/build/native.rs` and the stable Compose project name in `crates/bonesdeploy/src/build/compose.rs`; do not add backend-specific lock ownership.
- [x] Add `crates/bonesdeploy/tests/local_build_lock.rs` and its test-only artifact-open barrier as crate-root integration support that exposes same-site contention during artifact packaging and concurrent different-site progress through the package operation.

## Validation

- [x] Verify a same-site helper process holding `LocalBuildLock` causes a second package attempt to fail immediately and report the site name.
- [x] Verify different-site helper processes acquire separate locks and both complete without contention errors.
- [x] Verify the first helper's lock remains held through artifact packaging rather than ending after source export or backend execution.
- [x] Run focused `bonesdeploy` Rust tests, excluding e2e tests, and confirm the lock integration tests pass.
- [x] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `shfmt -w .`; address every warning or error.

## Completion

- [x] Review the final implementation diff for lock-path traversal, site aliasing, blocking acquisition, stale-PID logic, global locking, remote-lock coupling, and a guard lifetime that ends before artifact packaging.
- [x] Confirm the implementation changes only the listed build and crate-root test files and that no additional documentation is required beyond this planning record.

## Completion notes

Implemented without material deviation. `LocalBuildLock` validates the site before deriving a per-site cache lock path, uses non-blocking advisory acquisition, and is retained by `build::package` through artifact packaging. The crate-root process integration test compiles a test-only preload barrier that stops the first helper exactly when the artifact packager opens its designated exported file, then verifies same-site rejection before releasing packaging; it also verifies concurrent progress for distinct sites. Strict validation passed: `cargo test -p bonesdeploy --test local_build_lock`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `shfmt -w .`. E2E tests were not run. No related user documentation required updates.
