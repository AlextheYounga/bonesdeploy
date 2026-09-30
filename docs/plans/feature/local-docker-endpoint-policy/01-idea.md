# Idea

## Request

Create one local Docker client boundary in `bonesdeploy::build`. For each
package operation, resolve the effective Docker endpoint once, accept only a
local Unix-socket endpoint, snapshot the accepted selector, and construct every
native and Compose Docker command against that endpoint.

## Problem

Native commands in `crates/bonesdeploy/src/build/native.rs` inherit
`DOCKER_HOST` and `DOCKER_CONTEXT`, so a local build can unknowingly use a
remote daemon whose bind mounts refer to a different host. Compose commands in
`compose.rs` clear the environment and can therefore select a different
daemon. The two build paths can operate on different Docker endpoints during
one package operation, and a remote endpoint can be reached before the mistake
is visible.

## Definitions

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

## Desired outcome

Before any source is mounted or image is built, a package operation rejects a
missing, malformed, remote, or unstable Docker endpoint. With a valid local
endpoint, native and Compose command sequences use the same accepted endpoint,
including image tagging and saving. Compose retains explicit interpolation
environment behavior and the existing build-container secret exclusions.

## Scope

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

## Constraints

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

## Exclusions

- Podman support.
- Docker daemon installation or configuration.
- TLS remote Docker support.
- Retries, local build locking, process timeouts, or supervision changes.
- Registry credential support.
