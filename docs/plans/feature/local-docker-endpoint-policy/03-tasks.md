# Tasks

## Implementation

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

## Validation

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

## Completion

- [x] Confirm the implementation uses no new dependency and that Docker client
  selector variables are absent from Compose/build-container input files.
- [x] Review the final diff for scope: only the Docker build boundary, affected
  tests, and this plan record are changed; Podman, daemon installation, TLS
  remote Docker, retries, locking, supervision, and registry credentials remain
  excluded.

## Completion notes

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
