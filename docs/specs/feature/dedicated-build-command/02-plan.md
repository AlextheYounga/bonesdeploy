# Plan

## Current behavior

`Command::Deploy` dispatches to `commands::deploy::run`. That command loads
the root `.env` configuration and invokes `DeployWorkflow`. Its first operation
calls `build::package`, which acquires the per-site build lock, exports the
configured committed branch, runs the Native or Compose build, and packages the
result as a temporary artifact. The remaining workflow operations handle
production secrets, establish SSH, synchronize control-plane configuration, and
stream the artifact to BonesRemote.

`PackagedArtifact` owns a `NamedTempFile`, so the existing artifact naturally
disappears when it is dropped.

## Intended behavior

`bonesdeploy build` loads the same root configuration as deployment, calls
`build::package`, prints a successful build message containing the artifact
revision and byte length, and returns. Dropping the artifact removes its
temporary file. No remote operation is reachable from this command.

## Approach

Add a top-level `Build` command variant and dispatch it to a focused
`commands::build` module. The module loads the local configuration, calls the
existing artifact packager directly, and presents the artifact manifest's
revision and length. Keep `commands::deploy` and `DeployWorkflow` unchanged so
production deployment retains its existing sequence.

## Responsibilities and boundaries

`cli::args` owns public command parsing, and `cli::dispatch` owns routing the
new command. `commands::build` owns the build-only user workflow. The existing
`build` module remains the sole owner of source export, backend-specific build
execution, packaging, and temporary artifact lifecycle.

## Affected areas

- `crates/bonesdeploy/src/cli/args.rs`
- `crates/bonesdeploy/src/cli/dispatch.rs`
- `crates/bonesdeploy/src/commands/mod.rs`
- `crates/bonesdeploy/src/commands/build.rs`
- Focused BonesDeploy command tests
- `README.md`

## Decisions

The command packages an artifact, rather than stopping after the backend build,
because artifact packaging is part of the deployment's local work and can fail.

The command does not write an artifact to a user-selected path. The requested
behavior is build verification, and retaining transport bytes introduces an
unrequested artifact-export interface and cleanup responsibility.

## Risks

Duplicating the build process would allow build-only and deploy behavior to
drift. Calling only the existing packager avoids that risk. A command wiring
error could accidentally reach production operations; focused tests will prove
the build-only workflow invokes the packager without the deploy workflow.

## Validation

Add focused tests for the build-only command workflow, including branch-build
failure propagation and a successful artifact result. Run the relevant
BonesDeploy test suite, full non-E2E Rust tests, `cargo clippy`, `cargo fmt`,
`shfmt -w .`, the required Python checks, and `git diff --check`. Confirm the
help text and README expose `bonesdeploy build` as the local-only pre-deploy
verification command.
