# Idea

## Request

Make local Docker-backed native and Compose build subprocesses bounded and reliably cleaned up after normal failures and build timeouts. Native builds currently apply a timeout only to each build script and terminate the Docker exec client; Compose builds do not apply a timeout to their Docker commands. Both paths need reliable diagnostics and resource cleanup.

## Problem

The native build path can leave the container workload running after the Docker exec client is killed for a script timeout, and its ownership cleanup depends on the long-lived build container remaining usable. Compose commands use `status` and `output` directly, so they have no shared timeout behavior, retain weak failure diagnostics, and can leave generated release image tags in the local Docker image store. These failures make local builds hang, obscure the actual command error, or leave stale Docker state behind.

## Definitions

**Local build operation:** One externally executed Docker command used by the local native or Compose build path, including preparation, build, inspection, cleanup, and artifact commands.

**Build command runner:** The focused `bonesdeploy::build` component that owns spawning a local build operation, concurrently draining stdout and stderr, retaining bounded diagnostic tails, enforcing the configured deadline, and terminating and waiting for the child process.

**Configured build timeout:** `Bones.build.timeout_seconds` when it is nonzero. The existing `0` value means no user deadline and preserves unbounded behavior. The configured timeout applies independently to each local build operation rather than to the entire package operation.

**Generated Compose release tag:** An immutable site/service/revision image tag created for the current local Compose build and used to create the release artifact. It is temporary local build state and must be removed after the image archive is saved or when the build fails.

**Ownership cleanup:** Restoration of the original UID and GID for the mounted source and cache trees. It is distinct from removing the build container and must occur before normal container removal, or through a short-lived builder-image cleanup invocation when a timed-out workload makes the original container unusable.

## Desired outcome

Every bounded local Docker command terminates by its configured deadline, waits for its child, and reports useful bounded stdout/stderr diagnostics. A native script timeout terminates the container workload, waits for it, restores mounted ownership through a usable cleanup context, and removes stale container state. Normal native failures also restore ownership and remove the container while preserving both the primary failure and cleanup failures. Compose builds remove generated release tags after saving the artifact and on every failure path.

## Scope

The change includes:

- A focused local build command runner in `bonesdeploy::build` shared by native Docker control commands and Compose commands.
- Concurrent stdout/stderr draining with bounded diagnostic tails and clear nonzero-exit errors.
- Per-operation use of the existing `build.timeout_seconds` configuration, with `0` retaining unbounded semantics.
- Native timeout sequencing for workload termination, waiting, ownership restoration, and stale-container removal.
- Native cleanup error composition that keeps the primary build error and adds cleanup context.
- Compose generated release-tag lifecycle cleanup after save and on failures.
- Crate-root integration tests through public build APIs using fake Docker behavior for timeouts, diagnostics, child cleanup, ownership restoration, and tag cleanup.

## Constraints

- Keep native and Compose builds on the existing local artifact flow and shared build contract.
- Keep the runner focused on local Docker-backed build subprocesses; do not create a general process framework for unrelated commands.
- Drain both output streams concurrently so a child cannot block on a full pipe.
- Bound retained diagnostics so command failures cannot grow memory without limit.
- Preserve existing ownership restoration semantics and use the existing builder image for short-lived cleanup when the persistent build container cannot perform cleanup.
- Test observable behavior through the owning crate's public build APIs and fake Docker in crate-root tests.
- Do not run e2e tests for this change.

## Exclusions

- Full SIGKILL/process-crash cleanup guarantees outside normal returned failures and handled build timeouts.
- Docker daemon outage handling beyond reporting the runner's command failure and cleanup context.
- Retries, build locking, and Docker context policy.
- SSH timeouts or remote build supervision.
- Changes to build timeout configuration, defaults, or semantics.
