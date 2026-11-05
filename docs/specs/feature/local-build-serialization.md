# Local Build Serialization

<!-- Merged from feature/local-build-serialization -->

## Idea

### Request

Prevent two local builds or deploys for the same BonesDeploy site from concurrently using shared local Docker state.

### Problem

`build::package` exports a revision and then invokes either the native local builder or Docker Compose before packaging the artifact. Native builds use a per-site local cache, while Compose uses the stable project name `bonesdeploy-{site}`. Two local operations for the same site can therefore mutate the same cache, Compose project, images, containers, and generated artifact paths at the same time. The resulting race can corrupt build state or make one operation observe the other operation's intermediate state.

### Definitions

**Local build:** The `bonesdeploy::build::package` operation, including source export, native or Compose execution, and artifact packaging. A local deploy is a command path that invokes this operation before sending the packaged artifact.

**Site:** The validated BonesDeploy `Bones::project_name` identity used to derive the local cache path and Compose project name.

**LocalBuildLock:** A process-scoped RAII guard owned by `bonesdeploy::build`. It holds an OS advisory lock on one site-specific file for the full local build operation and releases that lock when the guard is dropped.

**Same-site contention:** A second local build attempts to acquire the lock for the same validated site while the first guard remains alive. It is an immediate failure, not a wait or retry.

### Desired outcome

Every local build acquires its site's `LocalBuildLock` before source export and holds it through native or Compose execution and artifact packaging. A concurrent same-site operation fails immediately with a clear diagnostic naming the site. Local builds for different sites acquire different locks and proceed independently. The advisory lock is released by RAII during normal cleanup and by the operating system when the owning process exits.

### Scope

Add site-scoped local-build serialization at the `bonesdeploy::build` boundary; use the existing build-cache root for lock-file placement; validate the site identity before deriving the lock path; preserve the existing native and Compose build flows; and add crate-root integration tests covering same-site rejection and different-site concurrency.

### Constraints

Use an advisory non-blocking lock on `std::fs::File` with RAII ownership and the diagnostic style established by `bonesremote::release::state::DeploymentLock`.

The lock identity is the validated site name, and the lock file is below the existing BonesDeploy build-cache root in that site's cache directory. Path construction must not allow an unvalidated site value to escape that directory or alias another site.

Contention fails immediately and names the site. Tests are crate-root Rust integration tests and do not use e2e coverage.

### Exclusions

This change does not add waiting, retries, stale-PID detection, a global lock, or remote deployment locking. It does not change native cache layout, Compose project naming, Docker commands, artifact formats, deploy transport, or remote BonesRemote locking semantics.


## Plan

### Current behavior

`crates/bonesdeploy/src/build/mod.rs::package` currently exports the committed source with `source::export`, dispatches on `config.runtime.backend`, runs `native::build` or `compose::build`, and then packages the result with `artifact::package` or `artifact::package_compose`. There is no operation-wide lock around this sequence.

`crates/bonesdeploy/src/build/native.rs::local_cache_path` derives the native cache below `paths::bones_cache_root().join("build")`, using `build_contract::cache_path` to append the site, target platform, and builder identity. `native::build` creates and mutates that cache while its Docker build container runs.

`crates/bonesdeploy/src/build/compose.rs::ComposeCommand::command` validates the site name and invokes Docker Compose with the stable project name `bonesdeploy-{site}`. Compose also reserves and writes generated artifact paths in the exported source and mutates local Docker image and project state.

`crates/bonesremote/src/release/state/mod.rs::DeploymentLock` is the existing diagnostic and RAII precedent: it creates a persistent per-site lock file, attempts an advisory non-blocking file lock, returns a site-specific contention error, and releases the lock on drop.

### Intended behavior

`build::package` validates the configured site identity with `bonesdeploy_core::config::validate_site_name` and acquires a `LocalBuildLock` before exporting source. The guard remains in scope through the selected native or Compose build and the corresponding artifact packaging call. A second process attempting the same site's lock receives an immediate error containing that site name; it never waits for the first operation. Different validated sites use different lock files and continue concurrently.

The lock file remains in the site's directory below the existing build-cache root. Its contents are not used for ownership or stale-process recovery. The OS advisory lock is the authority: dropping the `File` releases it during normal control flow and process exit releases it when the owner terminates.

### Approach

Add a focused `LocalBuildLock` implementation under `crates/bonesdeploy/src/build/`, backed by an opened `std::fs::File`. Call `bonesdeploy_core::config::validate_site_name` before deriving the path from the existing build-cache root and site name, then create the parent site cache directory before opening the persistent lock file. Acquire the file lock in non-blocking mode and translate `WouldBlock` into a clear `A local build is already running for {site}` diagnostic; translate other lock or filesystem failures with the lock path and site context.

Register the lock module from `build/mod.rs` and acquire the guard as the first operation in `package`. Keep the existing backend dispatch and artifact packaging unchanged apart from the guard's lifetime. Place integration coverage at the crate root under `crates/bonesdeploy/tests/`; exercise actual package entry-point behavior in helper processes so the test observes process-scoped lock ownership, assert same-site contention fails immediately with the site in the error, and assert distinct sites can proceed at the same time.

### Responsibilities and boundaries

`bonesdeploy::build::LocalBuildLock` owns lock-file path derivation, parent creation, advisory acquisition, contention translation, and RAII release. It does not own Docker commands, cache contents, Compose project naming, remote deployment state, or stale-process metadata.

`bonesdeploy::build::package` owns operation lifetime. It calls `validate_site_name` before path derivation, acquires the guard before source export, and keeps the guard alive through artifact packaging.

`native.rs` and `compose.rs` retain ownership of their existing backend-specific local state. `bonesremote::release::state::DeploymentLock` remains the owner of remote deployment serialization and is used only as semantic and diagnostic precedent.

`crates/bonesdeploy/tests/` owns crate-root integration tests for the observable package behavior. No e2e test owns this requirement.

### Affected areas

- `crates/bonesdeploy/src/build/mod.rs`: register and acquire the site-scoped guard at the package boundary.
- `crates/bonesdeploy/src/build/local_lock.rs`: implement `LocalBuildLock`, its build-cache lock path, non-blocking advisory acquisition, diagnostics, and drop release.
- `crates/bonesdeploy/tests/local_build_lock.rs`: add crate-root integration coverage for same-site contention and independent different-site operations, using process helpers where required by process-scoped locking.
- `crates/bonesdeploy/src/build/native.rs`: retain the existing per-site cache behavior; no backend logic changes are required beyond the package-level guard surrounding it.
- `crates/bonesdeploy/src/build/compose.rs`: retain the existing stable `bonesdeploy-{site}` Compose project behavior; no Compose-specific lock is added.

### Decisions

- The serialization boundary is `build::package`, because it covers source export, both build backends, and artifact packaging with one guard and prevents a race before either backend starts mutating state.
- Lock identity is the validated `project_name` site value. The lock path is `<BonesDeploy build-cache root>/<site>/local-build.lock`, keeping all lock state under the existing site cache boundary while separating sites.
- Acquisition is advisory and non-blocking. This matches the existing `DeploymentLock` semantics while ensuring a concurrent CLI fails promptly rather than consuming a process waiting on another local build.
- The lock is process-scoped and file-backed. OS release on process exit makes stale PID files and liveness protocols unnecessary and avoids treating file contents as ownership state.
- Ownership remains in `bonesdeploy::build`; remote deployment locking is a separate lifecycle concern and must not be coupled to local Docker state.
- Tests are crate-root integration tests because the promised behavior is the package operation's observable error and concurrency behavior, not an internal helper's implementation detail.

### Risks

- Acquiring before source export holds the lock during export failures and prevents another same-site operation from entering while the first operation is still handling its failure. RAII drop must cover every return path.
- Deriving a lock path from an unvalidated site could permit traversal or cause two identities to share a file. Validation before path construction, plus the existing site-name rules, prevents this path-aliasing risk.
- A lock held only around native or Compose execution would leave artifact packaging concurrent. The guard's scope must include both backend dispatch and the final artifact packaging call.
- A test that runs contenders only as threads could fail to represent process-scoped advisory locking. The integration test must use helper processes for ownership and contention evidence.
- Persistent lock files remain after normal completion, but their advisory locks do not. Tests and diagnostics must treat the lock state, not file existence, as contention evidence.

### Validation

- The local lock integration test starts a helper process that holds one site's package lock, starts a second helper for that same site, and verifies immediate failure with the site name in the diagnostic.
- The integration test starts concurrent helper processes for two different valid sites and verifies both acquire their locks and complete without contention errors.
- The package-path test verifies the guard is acquired before backend work and remains held through artifact packaging by making a second same-site package attempt while the first helper is inside the operation.
- Focused Rust tests for `crates/bonesdeploy` pass without running e2e tests.
- `cargo fmt --check`, workspace `cargo clippy`, and `shfmt -w .` complete without warnings or errors; the final diff contains only the planned lock implementation, tests, and planning records.


## Tasks

### Implementation

- [x] Add `crates/bonesdeploy/src/build/local_lock.rs` with `LocalBuildLock`, site validation before lock-path derivation, the per-site build-cache lock file, non-blocking advisory acquisition, site-naming diagnostics, and RAII release.
- [x] Register `local_lock` in `crates/bonesdeploy/src/build/mod.rs` and acquire the guard at the beginning of `build::package`, retaining it through source export, native or Compose execution, and artifact packaging.
- [x] Preserve the existing native cache path in `crates/bonesdeploy/src/build/native.rs` and the stable Compose project name in `crates/bonesdeploy/src/build/compose.rs`; do not add backend-specific lock ownership.
- [x] Add `crates/bonesdeploy/tests/local_build_lock.rs` and its test-only artifact-open barrier as crate-root integration support that exposes same-site contention during artifact packaging and concurrent different-site progress through the package operation.

### Validation

- [x] Verify a same-site helper process holding `LocalBuildLock` causes a second package attempt to fail immediately and report the site name.
- [x] Verify different-site helper processes acquire separate locks and both complete without contention errors.
- [x] Verify the first helper's lock remains held through artifact packaging rather than ending after source export or backend execution.
- [x] Run focused `bonesdeploy` Rust tests, excluding e2e tests, and confirm the lock integration tests pass.
- [x] Run `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `shfmt -w .`; address every warning or error.

### Completion

- [x] Review the final implementation diff for lock-path traversal, site aliasing, blocking acquisition, stale-PID logic, global locking, remote-lock coupling, and a guard lifetime that ends before artifact packaging.
- [x] Confirm the implementation changes only the listed build and crate-root test files and that no additional documentation is required beyond this planning record.

### Completion notes

Implemented without material deviation. `LocalBuildLock` validates the site before deriving a per-site cache lock path, uses non-blocking advisory acquisition, and is retained by `build::package` through artifact packaging. The crate-root process integration test compiles a test-only preload barrier that stops the first helper exactly when the artifact packager opens its designated exported file, then verifies same-site rejection before releasing packaging; it also verifies concurrent progress for distinct sites. Strict validation passed: `cargo test -p bonesdeploy --test local_build_lock`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `shfmt -w .`. E2E tests were not run. No related user documentation required updates.
