# Plan

## Current behavior

`crates/bonesdeploy/src/build/native.rs` creates one detached builder container, runs numbered scripts through `docker exec`, drains script output on reader threads, and applies `build_timeout_seconds(config)` only to each script's Docker exec client. On timeout it kills and waits for that client, but does not terminate the workload in the builder container. Cleanup normally runs `find ... chown` inside the builder container and then force-removes it; the drop fallback ignores cleanup errors.

The same module runs Docker availability checks, image inspection/pull, target probing, container creation, ownership restoration, and removal directly through `status` or `output`. Those operations have no shared deadline or bounded diagnostics.

`crates/bonesdeploy/src/build/compose.rs` constructs Docker Compose commands for validation, pull, build, image discovery, tagging, and saving. Validation, pull, build, and tagging use `status`; image discovery uses `output` and reports only the exit status. None uses the configured build timeout or a common diagnostic path. Generated `ComposeImage` tags remain in the local Docker image store after saving or failure.

`bonesdeploy-core::config::build_timeout_seconds` converts a nonzero `Bones.build.timeout_seconds` to `Some(seconds)` and converts `0` to `None`; the configuration default remains 300 seconds. The crate-root `crates/bonesdeploy/tests/native_build.rs` already exercises native packaging with a fake Docker executable and verifies command sequencing and ownership cleanup. The Rust conventions require behavioral tests in the owning crate's crate-root `tests/` directory and prohibit test modules in production files.

## Intended behavior

All local Docker-backed build operations go through one build command runner in `bonesdeploy::build`. The runner spawns the command, drains stdout and stderr concurrently, retains bounded tails for diagnostics, applies the configured per-operation deadline, and on timeout terminates and waits for the child before returning a timeout error. With no configured deadline (`build.timeout_seconds == 0`), it waits without a user deadline as it does today.

Native script timeout handling first uses the runner to terminate and wait for the Docker exec client, then terminates the script workload in the builder container and waits for that workload to stop. Ownership is restored before stale container removal: normal failures use the existing builder container, while a timed-out or otherwise unusable container uses a short-lived invocation of the existing builder image with the source and cache mounts. The primary build error remains the top-level error, with ownership and container cleanup failures added as context.

Compose validation, pull, build, image discovery, tagging, and saving all use the runner. Nonzero commands include bounded stderr and stdout tails in their errors. Tags created for the current release are tracked as temporary resources, removed after the archive is saved, and removed during failure cleanup. Tag cleanup errors are combined with the primary failure without hiding it.

## Approach

Add a focused runner module below `crates/bonesdeploy/src/build/` and register it from `build/mod.rs`. Give it the existing `Command` inputs and a small result carrying command output needed by Compose inventory; keep process supervision, stream draining, timeout enforcement, termination, waiting, and diagnostic formatting inside this boundary.

Refactor `native.rs` so all local Docker control commands use the runner. Keep the long-lived builder container responsible for ordinary script execution and ordinary ownership cleanup. On a script timeout, perform the explicit workload termination and wait sequence, then invoke ownership restoration in a short-lived builder-image cleanup context before force-removing the original container. Make the container cleanup path idempotent and compose primary and cleanup errors with the existing `anyhow` error-chain style.

Refactor `compose.rs` to use the runner for every Docker command. Introduce a small tag lifecycle guard in the Compose build flow that records successfully created release tags, deletes them after `save`, and deletes any partial set when an operation fails. Keep artifact file creation and persistence behavior unchanged apart from ensuring tag cleanup surrounds the complete flow.

## Responsibilities and boundaries

- `bonesdeploy::build::command` owns local child-process spawning, concurrent pipe draining, per-operation deadlines, child termination/waiting, bounded output tails, and command error construction.
- `build::native` owns native build-container lifecycle, script workload termination, ownership restoration, and composition of primary and cleanup errors. It supplies the runner with Docker commands but does not reimplement supervision.
- `build::compose` owns Compose command sequencing, image inventory, generated tag tracking, archive saving, and tag cleanup. It consumes runner output rather than handling `Command::status` or `Command::output` directly.
- `build::mod` exposes the existing package API and registers the focused runner module; it does not own Docker details.
- `crates/bonesdeploy/tests/` owns integration coverage through public build APIs and fake Docker executables. Production modules receive no test-only implementation seams.

## Affected areas

- `crates/bonesdeploy/src/build/mod.rs`: register the focused runner module.
- `crates/bonesdeploy/src/build/command.rs`: add the local build command runner.
- `crates/bonesdeploy/src/build/native.rs`: route Docker control commands through the runner and implement timeout workload/ownership/container cleanup sequencing.
- `crates/bonesdeploy/src/build/compose.rs`: route all Compose and image commands through the runner and manage generated release-tag cleanup.
- `crates/bonesdeploy/tests/native_build.rs`: extend fake-Docker public-API coverage for timeout, child cleanup, diagnostics, and ownership restoration.
- `crates/bonesdeploy/tests/compose_build.rs`: add public-API fake-Docker coverage for nonzero stderr diagnostics, operation timeout, and generated tag cleanup.

## Decisions

- Use one focused runner for native and Compose local Docker commands because both paths need identical deadline, pipe-draining, termination, waiting, and diagnostic behavior.
- Apply `build.timeout_seconds` independently to every external local build operation. A value of `0` means no user deadline and retains existing unbounded semantics; no new configuration is introduced.
- Retain bounded output tails rather than unbounded command capture so failures remain diagnosable without allowing child output to consume arbitrary memory.
- On native script timeout, clean ownership through a short-lived existing builder-image invocation after the workload has been terminated and waited for, then remove the stale long-lived container. Removing the container before ownership cleanup is not permitted because mounted files must be normalized first.
- Track generated Compose tags as temporary resources and delete them after archive save or during failure cleanup, so the local Docker image store does not accumulate release-only names.
- Preserve the primary operation error when cleanup also fails, adding cleanup details as error context instead of replacing the cause.
- Keep tests at the crate-root integration boundary and use fake Docker; do not add e2e coverage or test-only production hooks.

## Risks

- A runner timeout that kills only the Docker client could still leave the in-container script active; the native timeout sequence must explicitly terminate and wait for the workload and test the fake Docker event order.
- Sequential rather than concurrent pipe draining could deadlock a verbose Docker command; tests and code review must verify both streams are drained independently.
- Cleanup that runs after container removal could leave host-mounted files owned by the container user; ownership restoration must precede removal or use the short-lived cleanup context.
- Tag cleanup could remove an image name that predates the current build; only tags successfully created by this build may be tracked and removed.
- Cleanup failures could obscure the actual build failure; error assertions must verify primary diagnostics remain visible alongside cleanup context.
- Applying one deadline to the whole package instead of each operation would change established behavior; tests must distinguish per-operation deadlines and the unbounded `0` case.

## Validation

1. Add crate-root integration tests that exercise the public build API with fake Docker and prove a timed-out native operation terminates and waits for the child, terminates the native workload, restores ownership, and removes the container.
2. Add tests proving native and Compose nonzero failures retain bounded stdout/stderr diagnostics, including stderr from the failed command.
3. Add tests proving generated Compose release tags are removed after a successful archive save and after failures at each relevant post-tagging step.
4. Add a test proving `build.timeout_seconds == 0` leaves the operation unbounded and a configured nonzero value applies to each external operation.
5. Run the affected `bonesdeploy` crate tests and relevant workspace tests, excluding e2e tests.
6. Run `cargo fmt`, `cargo clippy`, and `shfmt -w .` from the worktree, then review the final diff to confirm only intended files changed and no scaffold text remains.
