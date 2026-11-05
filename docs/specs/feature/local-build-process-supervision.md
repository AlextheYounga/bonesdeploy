# Local Build Process Supervision

<!-- Merged from feature/local-build-process-supervision -->

## Idea

### Request

Make local Docker-backed native and Compose build subprocesses bounded and reliably cleaned up after normal failures and build timeouts. Native builds currently apply a timeout only to each build script and terminate the Docker exec client; Compose builds do not apply a timeout to their Docker commands. Both paths need reliable diagnostics and resource cleanup.

### Problem

The native build path can leave the container workload running after the Docker exec client is killed for a script timeout, and its ownership cleanup depends on the long-lived build container remaining usable. Compose commands use `status` and `output` directly, so they have no shared timeout behavior, retain weak failure diagnostics, and can leave generated release image tags in the local Docker image store. These failures make local builds hang, obscure the actual command error, or leave stale Docker state behind.

### Definitions

**Local build operation:** One externally executed Docker command used by the local native or Compose build path, including preparation, build, inspection, cleanup, and artifact commands.

**Build command runner:** The focused `bonesdeploy::build` component that owns spawning a local build operation, concurrently draining stdout and stderr, retaining bounded diagnostic tails, enforcing the configured deadline, and terminating and waiting for the child process.

**Configured build timeout:** `Bones.build.timeout_seconds` when it is nonzero. The existing `0` value means no user deadline and preserves unbounded behavior. The configured timeout applies independently to each local build operation rather than to the entire package operation.

**Generated Compose release tag:** An immutable site/service/revision image tag created for the current local Compose build and used to create the release artifact. It is temporary local build state and must be removed after the image archive is saved or when the build fails.

**Ownership cleanup:** Restoration of the original UID and GID for the mounted source and cache trees. It is distinct from removing the build container and must occur before normal container removal, or through a short-lived builder-image cleanup invocation when a timed-out workload makes the original container unusable.

### Desired outcome

Every bounded local Docker command terminates by its configured deadline, waits for its child, and reports useful bounded stdout/stderr diagnostics. A native script timeout terminates the container workload, waits for it, restores mounted ownership through a usable cleanup context, and removes stale container state. Normal native failures also restore ownership and remove the container while preserving both the primary failure and cleanup failures. Compose builds remove generated release tags after saving the artifact and on every failure path.

### Scope

The change includes:

- A focused local build command runner in `bonesdeploy::build` shared by native Docker control commands and Compose commands.
- Concurrent stdout/stderr draining with bounded diagnostic tails and clear nonzero-exit errors.
- Per-operation use of the existing `build.timeout_seconds` configuration, with `0` retaining unbounded semantics.
- Native timeout sequencing for workload termination, waiting, ownership restoration, and stale-container removal.
- Native cleanup error composition that keeps the primary build error and adds cleanup context.
- Compose generated release-tag lifecycle cleanup after save and on failures.
- Crate-root integration tests through public build APIs using fake Docker behavior for timeouts, diagnostics, child cleanup, ownership restoration, and tag cleanup.

### Constraints

- Keep native and Compose builds on the existing local artifact flow and shared build contract.
- Keep the runner focused on local Docker-backed build subprocesses; do not create a general process framework for unrelated commands.
- Drain both output streams concurrently so a child cannot block on a full pipe.
- Bound retained diagnostics so command failures cannot grow memory without limit.
- Preserve existing ownership restoration semantics and use the existing builder image for short-lived cleanup when the persistent build container cannot perform cleanup.
- Test observable behavior through the owning crate's public build APIs and fake Docker in crate-root tests.
- Do not run e2e tests for this change.

### Exclusions

- Full SIGKILL/process-crash cleanup guarantees outside normal returned failures and handled build timeouts.
- Docker daemon outage handling beyond reporting the runner's command failure and cleanup context.
- Retries, build locking, and Docker context policy.
- SSH timeouts or remote build supervision.
- Changes to build timeout configuration, defaults, or semantics.


## Plan

### Current behavior

`crates/bonesdeploy/src/build/native.rs` creates one detached builder container, runs numbered scripts through `docker exec`, drains script output on reader threads, and applies `build_timeout_seconds(config)` only to each script's Docker exec client. On timeout it kills and waits for that client, but does not terminate the workload in the builder container. Cleanup normally runs `find ... chown` inside the builder container and then force-removes it; the drop fallback ignores cleanup errors.

The same module runs Docker availability checks, image inspection/pull, target probing, container creation, ownership restoration, and removal directly through `status` or `output`. Those operations have no shared deadline or bounded diagnostics.

`crates/bonesdeploy/src/build/compose.rs` constructs Docker Compose commands for validation, pull, build, image discovery, tagging, and saving. Validation, pull, build, and tagging use `status`; image discovery uses `output` and reports only the exit status. None uses the configured build timeout or a common diagnostic path. Generated `ComposeImage` tags remain in the local Docker image store after saving or failure.

`bonesdeploy-core::config::build_timeout_seconds` converts a nonzero `Bones.build.timeout_seconds` to `Some(seconds)` and converts `0` to `None`; the configuration default remains 300 seconds. The crate-root `crates/bonesdeploy/tests/native_build.rs` already exercises native packaging with a fake Docker executable and verifies command sequencing and ownership cleanup. The Rust conventions require behavioral tests in the owning crate's crate-root `tests/` directory and prohibit test modules in production files.

### Intended behavior

All local Docker-backed build operations go through one build command runner in `bonesdeploy::build`. The runner spawns the command, drains stdout and stderr concurrently, retains bounded tails for diagnostics, applies the configured per-operation deadline, and on timeout terminates and waits for the child before returning a timeout error. With no configured deadline (`build.timeout_seconds == 0`), it waits without a user deadline as it does today.

Native script timeout handling first uses the runner to terminate and wait for the Docker exec client, then terminates the script workload in the builder container and waits for that workload to stop. Ownership is restored before stale container removal: normal failures use the existing builder container, while a timed-out or otherwise unusable container uses a short-lived invocation of the existing builder image with the source and cache mounts. The primary build error remains the top-level error, with ownership and container cleanup failures added as context.

Compose validation, pull, build, image discovery, tagging, and saving all use the runner. Nonzero commands include bounded stderr and stdout tails in their errors. Tags created for the current release are tracked as temporary resources, removed after the archive is saved, and removed during failure cleanup. Tag cleanup errors are combined with the primary failure without hiding it.

### Approach

Add a focused runner module below `crates/bonesdeploy/src/build/` and register it from `build/mod.rs`. Give it the existing `Command` inputs and a small result carrying command output needed by Compose inventory; keep process supervision, stream draining, timeout enforcement, termination, waiting, and diagnostic formatting inside this boundary.

Refactor `native.rs` so all local Docker control commands use the runner. Keep the long-lived builder container responsible for ordinary script execution and ordinary ownership cleanup. On a script timeout, perform the explicit workload termination and wait sequence, then invoke ownership restoration in a short-lived builder-image cleanup context before force-removing the original container. Make the container cleanup path idempotent and compose primary and cleanup errors with the existing `anyhow` error-chain style.

Refactor `compose.rs` to use the runner for every Docker command. Introduce a small tag lifecycle guard in the Compose build flow that records successfully created release tags, deletes them after `save`, and deletes any partial set when an operation fails. Keep artifact file creation and persistence behavior unchanged apart from ensuring tag cleanup surrounds the complete flow.

### Responsibilities and boundaries

- `bonesdeploy::build::command` owns local child-process spawning, concurrent pipe draining, per-operation deadlines, child termination/waiting, bounded output tails, and command error construction.
- `build::native` owns native build-container lifecycle, script workload termination, ownership restoration, and composition of primary and cleanup errors. It supplies the runner with Docker commands but does not reimplement supervision.
- `build::compose` owns Compose command sequencing, image inventory, generated tag tracking, archive saving, and tag cleanup. It consumes runner output rather than handling `Command::status` or `Command::output` directly.
- `build::mod` exposes the existing package API and registers the focused runner module; it does not own Docker details.
- `crates/bonesdeploy/tests/` owns integration coverage through public build APIs and fake Docker executables. Production modules receive no test-only implementation seams.

### Affected areas

- `crates/bonesdeploy/src/build/mod.rs`: register the focused runner module.
- `crates/bonesdeploy/src/build/command.rs`: add the local build command runner.
- `crates/bonesdeploy/src/build/native.rs`: route Docker control commands through the runner and implement timeout workload/ownership/container cleanup sequencing.
- `crates/bonesdeploy/src/build/compose.rs`: route all Compose and image commands through the runner and manage generated release-tag cleanup.
- `crates/bonesdeploy/tests/native_build.rs`: extend fake-Docker public-API coverage for timeout, child cleanup, diagnostics, and ownership restoration.
- `crates/bonesdeploy/tests/compose_build.rs`: add public-API fake-Docker coverage for nonzero stderr diagnostics, operation timeout, and generated tag cleanup.

### Decisions

- Use one focused runner for native and Compose local Docker commands because both paths need identical deadline, pipe-draining, termination, waiting, and diagnostic behavior.
- Apply `build.timeout_seconds` independently to every external local build operation. A value of `0` means no user deadline and retains existing unbounded semantics; no new configuration is introduced.
- Retain bounded output tails rather than unbounded command capture so failures remain diagnosable without allowing child output to consume arbitrary memory.
- On native script timeout, clean ownership through a short-lived existing builder-image invocation after the workload has been terminated and waited for, then remove the stale long-lived container. Removing the container before ownership cleanup is not permitted because mounted files must be normalized first.
- Track generated Compose tags as temporary resources and delete them after archive save or during failure cleanup, so the local Docker image store does not accumulate release-only names.
- Preserve the primary operation error when cleanup also fails, adding cleanup details as error context instead of replacing the cause.
- Keep tests at the crate-root integration boundary and use fake Docker; do not add e2e coverage or test-only production hooks.

### Risks

- A runner timeout that kills only the Docker client could still leave the in-container script active; the native timeout sequence must explicitly terminate and wait for the workload and test the fake Docker event order.
- Sequential rather than concurrent pipe draining could deadlock a verbose Docker command; tests and code review must verify both streams are drained independently.
- Cleanup that runs after container removal could leave host-mounted files owned by the container user; ownership restoration must precede removal or use the short-lived cleanup context.
- Tag cleanup could remove an image name that predates the current build; only tags successfully created by this build may be tracked and removed.
- Cleanup failures could obscure the actual build failure; error assertions must verify primary diagnostics remain visible alongside cleanup context.
- Applying one deadline to the whole package instead of each operation would change established behavior; tests must distinguish per-operation deadlines and the unbounded `0` case.

### Validation

1. Add crate-root integration tests that exercise the public build API with fake Docker and prove a timed-out native operation terminates and waits for the child, terminates the native workload, restores ownership, and removes the container.
2. Add tests proving native and Compose nonzero failures retain bounded stdout/stderr diagnostics, including stderr from the failed command.
3. Add tests proving generated Compose release tags are removed after a successful archive save and after failures at each relevant post-tagging step.
4. Add a test proving `build.timeout_seconds == 0` leaves the operation unbounded and a configured nonzero value applies to each external operation.
5. Run the affected `bonesdeploy` crate tests and relevant workspace tests, excluding e2e tests.
6. Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` from the worktree, then review the final diff to confirm only intended files changed and no scaffold text remains.


## Tasks

### Implementation

- [x] Add the focused `bonesdeploy::build` command runner and register it from `build/mod.rs`; it must concurrently drain stdout and stderr, retain bounded tails, apply each operation's configured deadline, terminate and wait on timeout, and return command results suitable for Compose inventory.
- [x] Route native Docker availability, image, probe, container, script, ownership, and removal commands through the runner while preserving the existing builder-image and mounted-path contract.
- [x] Implement native script-timeout cleanup in the settled order: terminate the container workload, wait for it, restore source/cache ownership through a short-lived existing builder-image cleanup context, remove stale container state, and preserve primary plus cleanup errors.
- [x] Refactor Compose validation, pull, build, image discovery, tagging, and saving to use the runner and include bounded command diagnostics on nonzero exit.
- [x] Add Compose generated-tag lifecycle tracking so successfully created release tags are removed after archive save and on every failure path without removing unrelated image tags.
- [x] Extend crate-root fake-Docker integration coverage through public build APIs for configured and unbounded operation deadlines, nonzero stderr diagnostics, child/workload cleanup, ownership restoration, and Compose tag cleanup.

### Validation

- [x] Verify the native timeout test observes child termination and waiting before ownership restoration and container removal.
- [x] Verify failure assertions retain the original operation error and include cleanup failure context when cleanup also fails.
- [x] Verify both output streams remain drainable for verbose commands and retained diagnostics stay within the runner's configured bound.
- [x] Verify Compose tags are absent after successful save and after failures occurring after tagging.
- [x] Run the affected `bonesdeploy` tests and relevant non-e2e workspace tests; record the observed results in completion notes.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`; address all warnings and errors.

### Completion

- [x] Review the final diff for scope compliance: only the planned implementation and crate-root test files changed, with no e2e work, scaffold text, or unrelated configuration changes.
- [x] Confirm the three authoritative planning files still describe one coherent approach and that timeout `0` remains explicitly unbounded.

### Completion notes

Implemented the focused command runner, native timeout cleanup sequence, and Compose tag lifecycle cleanup. The runner retains 64 KiB tails per stream; full stdout is retained only for successful Compose configuration inventory parsing. The Compose command retains `PATH` after its existing environment clear so Docker can be resolved without exposing the ambient build environment. Focused integration tests cover native timeout sequencing, verbose two-stream diagnostic tails, primary-plus-cleanup error composition, Compose cleanup after success and save failure, configured deadlines, and timeout `0` unbounded behavior.

Validation passed: `cargo test -p bonesdeploy --test native_build --test compose_build` (10 tests); `cargo test --workspace --exclude e2e`; `cargo fmt`; and `cargo clippy`. `shfmt -w .` encountered generated non-shell Python-venv launchers in ignored `target/`; formatting all tracked `*.sh` files with `git ls-files -z '*.sh' | xargs -0 shfmt -w` succeeded. No e2e tests were run. Final diff and status were reviewed; changed files are the planned build implementation/tests, this plan record, and timeout documentation.

Review remediation: the timed-out workload stop now has its own 15-second supervision bound around Docker's 10-second stop grace period, and the timeout runner waits for the child and both output drainers even if termination races with process exit. Ownership cleanup falls back to a short-lived builder container when ordinary cleanup fails, and failed force-removal leaves the container eligible for the existing Drop retry. Tests now verify completed workload stopping before cleanup, exact 64 KiB tails for each verbose stream, partial Compose tag cleanup, and cleanup-error composition. Revalidation is pending.

Revalidation passed: `cargo test -p bonesdeploy --test native_build --test compose_build` (14 tests), `cargo test --workspace --exclude e2e`, `cargo fmt`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and tracked-shell `shfmt`. No e2e test target was run.
