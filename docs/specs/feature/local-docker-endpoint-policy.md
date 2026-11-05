# Local Docker Endpoint Policy

<!-- Merged from feature/local-docker-endpoint-policy -->

## Idea

### Request

Create one local Docker client boundary in `bonesdeploy::build`. For each
package operation, resolve the effective Docker endpoint once, accept only a
local Unix-socket endpoint, snapshot the accepted selector, and construct every
native and Compose Docker command against that endpoint.

### Problem

Native commands in `crates/bonesdeploy/src/build/native.rs` inherit
`DOCKER_HOST` and `DOCKER_CONTEXT`, so a local build can unknowingly use a
remote daemon whose bind mounts refer to a different host. Compose commands in
`compose.rs` clear the environment and can therefore select a different
daemon. The two build paths can operate on different Docker endpoints during
one package operation, and a remote endpoint can be reached before the mistake
is visible.

### Definitions

**Package operation:** The synchronous work performed by `build::package`,
from Docker endpoint resolution through native or Compose image construction
and artifact creation.

**Docker client boundary:** The `bonesdeploy::build`-owned value that contains
the accepted Docker selector and resolved endpoint, validates selector
stability, and creates all Docker child-process commands for one package
operation.

**Local Unix-socket endpoint:** A Docker endpoint using the Unix socket scheme
with a valid socket path. This includes Docker Desktop and rootless Docker
contexts; it does not include TCP, SSH, Windows named-pipe, TLS-remote, or
malformed endpoints.

**Accepted selector:** The effective `DOCKER_HOST`/`DOCKER_CONTEXT` selection
observed when the Docker client boundary is created, together with its
resolved local Unix-socket endpoint. All commands in the package operation use
this snapshot rather than independently resolving Docker configuration.

**Compose/build-container environment:** Variables deliberately supplied to
Compose through its explicit `--env-file` and to native build containers
through the existing generated environment file. Docker client selector
variables belong to the Docker CLI process environment and must not be added to
these container inputs.

### Desired outcome

Before any source is mounted or image is built, a package operation rejects a
missing, malformed, remote, or unstable Docker endpoint. With a valid local
endpoint, native and Compose command sequences use the same accepted endpoint,
including image tagging and saving. Compose retains explicit interpolation
environment behavior and the existing build-container secret exclusions.

### Scope

- Add the local Docker client boundary under `bonesdeploy::build`.
- Resolve and validate the effective endpoint once per package operation.
- Reject `tcp://`, `ssh://`, `npipe://`, malformed, and missing endpoints.
- Apply the accepted endpoint consistently to native and Compose Docker
  commands, including cleanup, tagging, and image archiving.
- Detect changes to the accepted selector during the package operation before a
  subsequent Docker command runs.
- Add crate-root fake-Docker integration coverage for local context acceptance,
  remote selector rejection before mutation, and identical endpoint selection
  across native and Compose command sequences.

### Constraints

- Support local Unix-socket Docker Desktop and rootless contexts; do not
  require the context name `default`.
- Preserve Compose's explicit interpolation environment and build container
  secret exclusions.
- Keep Docker CLI client environment separate from variables injected into
  Compose or build containers.
- Ground the implementation in the current `build::package`, `native.rs`, and
  `compose.rs` flow and use existing dependencies.
- Tests must use the crate-root `crates/bonesdeploy/tests/` integration boundary.
- Implementation begins only after human review and approval of this planning
  record.

### Exclusions

- Podman support.
- Docker daemon installation or configuration.
- TLS remote Docker support.
- Retries, local build locking, process timeouts, or supervision changes.
- Registry credential support.


## Plan

### Current Behavior

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

### Intended Behavior

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

### Approach

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

### Responsibilities And Boundaries

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

### Affected Areas

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

### Decisions

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

### Risks

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

### Validation

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


## Tasks

### Implementation

- [x] Add the operation-scoped Docker client boundary under
  `crates/bonesdeploy/src/build`, including effective selector capture,
  context endpoint resolution, strict local Unix-socket validation, and a
  selector snapshot check before command creation.
- [x] Create the client in `build::package` before native or Compose Docker
  work and pass the same client reference through the selected backend.
- [x] Route every native Docker command in `native.rs`, including availability,
  image, target probe, container lifecycle, exec, and cleanup commands, through
  the operation-scoped client without changing build semantics.
- [x] Route Compose validation, pull, build, config inspection, image tagging,
  and image saving commands through the same client while preserving
  `env_clear`, `DOCKER_DEFAULT_PLATFORM`, explicit `--env-file` interpolation,
  and build-container secret exclusions.
- [x] Keep non-package Docker availability callers such as site doctor compiling
  and using the shared endpoint policy without creating a second command
  construction path.
- [x] Add crate-root fake-Docker integration coverage in
  `crates/bonesdeploy/tests/docker_endpoint.rs`, with isolated child processes,
  a non-`default` local context, remote/malformed/missing endpoint cases, and
  command-log assertions across native and Compose sequences.
- [x] Adjust `tests/native_build.rs`'s existing fake-Docker fixture only as
  necessary for the shared client environment contract.

### Validation

- [x] Verify the local non-`default` context completes and all recorded Docker
  invocations use one identical Unix endpoint, including native cleanup and
  Compose tag/save operations.
- [x] Verify TCP, SSH, named-pipe, malformed, missing, and remote-context
  selectors fail before any fake-Docker mount, pull, build, tag, save, or
  container mutation is recorded.
- [x] Verify a selector change during a package operation is rejected before
  the following Docker command starts for both `DOCKER_HOST` and
  `DOCKER_CONTEXT`.
- [x] Verify fake Docker captures the actual native and Compose `--env-file`
  contents and neither contains a Docker selector variable.
- [x] Verify endpoint validation rejects whitespace, control characters, and
  embedded newlines from direct hosts and context metadata.
- [x] Run the focused `bonesdeploy` integration tests and relevant crate tests;
  do not run e2e tests.
- [x] Run `cargo fmt`, `cargo clippy`, and `shfmt -w .`, addressing all
  warnings and errors.

### Completion

- [x] Confirm the implementation uses no new dependency and that Docker client
  selector variables are absent from Compose/build-container input files.
- [x] Review the final diff for scope: only the Docker build boundary, affected
  tests, and this plan record are changed; Podman, daemon installation, TLS
  remote Docker, retries, locking, supervision, and registry credentials remain
  excluded.

### Completion notes

Implemented the operation-scoped `DockerClient` boundary and routed native and
Compose commands through its accepted Unix endpoint. Added isolated crate-root
fake-Docker coverage for non-default local contexts, invalid selectors,
`DOCKER_HOST` and `DOCKER_CONTEXT` selector drift, and strict whitespace/control
endpoint rejection. The tests archive and inspect the generated native and
Compose environment files to prove Docker selectors are absent. No dependency
was added; Compose and build-container environment files continue to receive
only their existing explicit build inputs.

Validation completed: focused `bonesdeploy` tests, `cargo fmt --all`, strict
workspace Clippy, and `shfmt -w .`. E2E tests were not run.
