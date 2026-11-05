# Tasks

## Implementation

- [x] Add the focused `bonesdeploy::build` command runner and register it from `build/mod.rs`; it must concurrently drain stdout and stderr, retain bounded tails, apply each operation's configured deadline, terminate and wait on timeout, and return command results suitable for Compose inventory.
- [x] Route native Docker availability, image, probe, container, script, ownership, and removal commands through the runner while preserving the existing builder-image and mounted-path contract.
- [x] Implement native script-timeout cleanup in the settled order: terminate the container workload, wait for it, restore source/cache ownership through a short-lived existing builder-image cleanup context, remove stale container state, and preserve primary plus cleanup errors.
- [x] Refactor Compose validation, pull, build, image discovery, tagging, and saving to use the runner and include bounded command diagnostics on nonzero exit.
- [x] Add Compose generated-tag lifecycle tracking so successfully created release tags are removed after archive save and on every failure path without removing unrelated image tags.
- [x] Extend crate-root fake-Docker integration coverage through public build APIs for configured and unbounded operation deadlines, nonzero stderr diagnostics, child/workload cleanup, ownership restoration, and Compose tag cleanup.

## Validation

- [x] Verify the native timeout test observes child termination and waiting before ownership restoration and container removal.
- [x] Verify failure assertions retain the original operation error and include cleanup failure context when cleanup also fails.
- [x] Verify both output streams remain drainable for verbose commands and retained diagnostics stay within the runner's configured bound.
- [x] Verify Compose tags are absent after successful save and after failures occurring after tagging.
- [x] Run the affected `bonesdeploy` tests and relevant non-e2e workspace tests; record the observed results in completion notes.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`; address all warnings and errors.

## Completion

- [x] Review the final diff for scope compliance: only the planned implementation and crate-root test files changed, with no e2e work, scaffold text, or unrelated configuration changes.
- [x] Confirm the three authoritative planning files still describe one coherent approach and that timeout `0` remains explicitly unbounded.

## Completion notes

Implemented the focused command runner, native timeout cleanup sequence, and Compose tag lifecycle cleanup. The runner retains 64 KiB tails per stream; full stdout is retained only for successful Compose configuration inventory parsing. The Compose command retains `PATH` after its existing environment clear so Docker can be resolved without exposing the ambient build environment. Focused integration tests cover native timeout sequencing, verbose two-stream diagnostic tails, primary-plus-cleanup error composition, Compose cleanup after success and save failure, configured deadlines, and timeout `0` unbounded behavior.

Validation passed: `cargo test -p bonesdeploy --test native_build --test compose_build` (10 tests); `cargo test --workspace --exclude e2e`; `cargo fmt`; and `cargo clippy`. `shfmt -w .` encountered generated non-shell Python-venv launchers in ignored `target/`; formatting all tracked `*.sh` files with `git ls-files -z '*.sh' | xargs -0 shfmt -w` succeeded. No e2e tests were run. Final diff and status were reviewed; changed files are the planned build implementation/tests, this plan record, and timeout documentation.

Review remediation: the timed-out workload stop now has its own 15-second supervision bound around Docker's 10-second stop grace period, and the timeout runner waits for the child and both output drainers even if termination races with process exit. Ownership cleanup falls back to a short-lived builder container when ordinary cleanup fails, and failed force-removal leaves the container eligible for the existing Drop retry. Tests now verify completed workload stopping before cleanup, exact 64 KiB tails for each verbose stream, partial Compose tag cleanup, and cleanup-error composition. Revalidation is pending.

Revalidation passed: `cargo test -p bonesdeploy --test native_build --test compose_build` (14 tests), `cargo test --workspace --exclude e2e`, `cargo fmt`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and tracked-shell `shfmt`. No e2e test target was run.
