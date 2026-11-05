# Idea

## Request

Prevent two local builds or deploys for the same BonesDeploy site from concurrently using shared local Docker state.

## Problem

`build::package` exports a revision and then invokes either the native local builder or Docker Compose before packaging the artifact. Native builds use a per-site local cache, while Compose uses the stable project name `bonesdeploy-{site}`. Two local operations for the same site can therefore mutate the same cache, Compose project, images, containers, and generated artifact paths at the same time. The resulting race can corrupt build state or make one operation observe the other operation's intermediate state.

## Definitions

**Local build:** The `bonesdeploy::build::package` operation, including source export, native or Compose execution, and artifact packaging. A local deploy is a command path that invokes this operation before sending the packaged artifact.

**Site:** The validated BonesDeploy `Bones::project_name` identity used to derive the local cache path and Compose project name.

**LocalBuildLock:** A process-scoped RAII guard owned by `bonesdeploy::build`. It holds an OS advisory lock on one site-specific file for the full local build operation and releases that lock when the guard is dropped.

**Same-site contention:** A second local build attempts to acquire the lock for the same validated site while the first guard remains alive. It is an immediate failure, not a wait or retry.

## Desired outcome

Every local build acquires its site's `LocalBuildLock` before source export and holds it through native or Compose execution and artifact packaging. A concurrent same-site operation fails immediately with a clear diagnostic naming the site. Local builds for different sites acquire different locks and proceed independently. The advisory lock is released by RAII during normal cleanup and by the operating system when the owning process exits.

## Scope

Add site-scoped local-build serialization at the `bonesdeploy::build` boundary; use the existing build-cache root for lock-file placement; validate the site identity before deriving the lock path; preserve the existing native and Compose build flows; and add crate-root integration tests covering same-site rejection and different-site concurrency.

## Constraints

Use an advisory non-blocking lock on `std::fs::File` with RAII ownership and the diagnostic style established by `bonesremote::release::state::DeploymentLock`.

The lock identity is the validated site name, and the lock file is below the existing BonesDeploy build-cache root in that site's cache directory. Path construction must not allow an unvalidated site value to escape that directory or alias another site.

Contention fails immediately and names the site. Tests are crate-root Rust integration tests and do not use e2e coverage.

## Exclusions

This change does not add waiting, retries, stale-PID detection, a global lock, or remote deployment locking. It does not change native cache layout, Compose project naming, Docker commands, artifact formats, deploy transport, or remote BonesRemote locking semantics.
