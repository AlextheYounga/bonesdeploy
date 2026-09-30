# Plan

## Current Behavior

`build::package` in `crates/bonesdeploy/src/build/mod.rs` exports the committed
source context and then dispatches to either `native::build` or
`compose::build`. Native build helpers create separate `Command::new("docker")`
values for `info`, image inspection/pull, target probing, container start and
exec, ownership cleanup, and container removal. Those commands inherit the
parent selector environment.

`compose::build` discovers files, writes the existing explicit build
environment file, and runs Compose for validation, pull, build, and image
inspection. `ComposeCommand::command` calls `env_clear`, restores only
`DOCKER_DEFAULT_PLATFORM`, and supplies the Compose `--env-file`; image tag and
save helpers create additional unbound Docker commands. Thus native and
Compose commands do not share endpoint selection.

The crate's Rust convention places behavioral integration tests in
`crates/bonesdeploy/tests/`. `tests/native_build.rs` already isolates a child
test process, prepends a fake `docker` executable to `PATH`, and records command
arguments, but it covers only native behavior.

## Intended Behavior

`build::package` creates a Docker client boundary before the backend build
starts. The boundary resolves the effective selector and endpoint once,
rejects missing or malformed endpoint metadata and all non-Unix schemes, and
stores the accepted selector plus normalized Unix-socket endpoint. A local
context is valid regardless of its name, including Docker Desktop and rootless
contexts.

Every Docker command created by native and Compose build code is created
through that boundary. Commands use the stored Unix endpoint, and command
creation verifies that the parent selector still matches the accepted
snapshot. A selector change fails before the next Docker operation. Endpoint
variables are never written to the Compose interpolation file or native build
container environment file.

Remote endpoint rejection occurs before native bind mounts, Compose mutation,
image pulls, or image builds. Existing Compose arguments, explicit
`--env-file` interpolation, `DOCKER_DEFAULT_PLATFORM`, and secret exclusions
remain unchanged.

## Approach

Add a focused Docker client module in `crates/bonesdeploy/src/build` and expose
only the boundary operations needed by the build modules. Its constructor
captures `DOCKER_HOST` and `DOCKER_CONTEXT`, resolves the effective Docker
context endpoint using the Docker CLI's local context metadata, validates the
endpoint as a Unix socket, and records the exact accepted selector. Direct
`DOCKER_HOST` values are validated without allowing a remote scheme; context
endpoints are normalized to the same stored Unix host value.

Thread one client reference from `build::package` into `native::build` and
`compose::build`. Replace every Docker command constructor in those modules,
including native cleanup and Compose tag/save helpers, with client command
construction. Preserve Compose's `env_clear` policy by applying the captured
Docker client environment after clearing it, while leaving its explicit
Compose `--env-file` untouched.

Use a fake Docker executable in crate-root integration tests. Run package
operations in isolated child processes so selector environment and `PATH` are
not shared between tests. Make the fake executable return local context
metadata and record endpoint-related environment for every invocation; provide
the minimum native and Compose responses needed to exercise complete command
sequences.

## Responsibilities And Boundaries

- `build::package`: owns one package-operation client lifetime and resolves the
  endpoint before backend-specific work begins.
- `bonesdeploy::build` Docker client module: owns selector capture, endpoint
  parsing/validation, selector-stability checks, and Docker `Command` creation.
- `build::native`: owns native build ordering and existing mount/container
  behavior; it receives the client and does not resolve Docker independently.
- `build::compose`: owns Compose file discovery, explicit interpolation
  environment, image inventory, tagging, and archive creation; it receives the
  client and does not resolve Docker independently.
- `crates/bonesdeploy/tests/`: owns observable fake-Docker integration tests,
  with no production test seams or `cfg(test)` modules.

## Affected Areas

- `crates/bonesdeploy/src/build/mod.rs`: create the client for each package
  operation and pass it to the selected backend.
- `crates/bonesdeploy/src/build/docker.rs`: add the focused local Docker client
  boundary, endpoint validation, selector snapshot, and command factory.
- `crates/bonesdeploy/src/build/native.rs`: route all native Docker commands
  through the client without changing build/container semantics.
- `crates/bonesdeploy/src/build/compose.rs`: route Compose, image tag, and image
  save commands through the client while preserving environment-file behavior.
- `crates/bonesdeploy/tests/docker_endpoint.rs`: add isolated fake-Docker
  integration coverage for endpoint acceptance/rejection and cross-backend
  consistency.
- `crates/bonesdeploy/tests/native_build.rs`: update the existing fake-Docker
  fixture only where required to assert the new client environment contract.

## Decisions

- The client boundary is owned by `bonesdeploy::build` because native and
  Compose are sibling local build backends and must share one operation-scoped
  selector.
- The accepted command endpoint is the resolved Unix socket, not a context
  name. This supports non-default local contexts and prevents later context
  resolution from selecting a different daemon.
- Docker CLI selector state is applied only to child Docker commands. It is not
  included in Compose interpolation or build-container variables, preserving
  the existing secret and explicit-environment boundaries.
- Validation is performed before `source` is mounted or images are built. The
  existing source export remains the source provenance step; no new source or
  artifact mutation is introduced by endpoint validation.
- No dependency is added. Endpoint scheme parsing and process/environment
  handling use the standard library and existing `anyhow`/temporary-file
  support.

## Risks

- Docker Desktop and rootless context metadata can use socket paths outside the
  conventional `/var/run/docker.sock`; rejecting by path shape instead of
  scheme would break supported local contexts.
- Compose's current `env_clear` behavior prevents accidental host-variable
  interpolation; adding only the captured client selector must not reintroduce
  ambient variables.
- Cleanup commands run after a failed native operation and must retain the same
  endpoint or cleanup may target a different daemon.
- Existing callers of native Docker availability checks, including site doctor,
  must continue to compile and retain their diagnostic behavior while package
  commands use the operation-scoped client.

## Validation

- A fake local Unix-socket context with a non-`default` name permits a package
  operation, and every recorded native and Compose command carries the same
  accepted endpoint.
- Fake `DOCKER_HOST=tcp://...`, `DOCKER_CONTEXT` resolving to a remote
  endpoint, `ssh://...`, `npipe://...`, malformed values, and absent endpoint
  metadata fail before the fake Docker log records a mount, pull, build, or
  image mutation.
- A selector change between commands fails before the next command is started.
- Native cleanup, Compose tagging, and Compose image saving are included in the
  same endpoint-consistency assertion.
- The focused bonesdeploy integration tests pass, followed by `cargo fmt`,
  `cargo clippy`, and the repository-required `shfmt -w .`; e2e tests remain
  excluded unless explicitly requested.
- The final diff contains changes only for the planned implementation/test
  files and these plan records; no dependency or unrelated documentation change
  is introduced.
