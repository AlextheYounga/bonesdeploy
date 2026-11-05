# Build Without Deploy

<!-- Merged from feature/dedicated-build-command -->

## Idea

### Request

Add a build command that performs the same local build used for deployments
without pushing an artifact to the remote server.

### Problem

The only public command that runs the production build also synchronizes
control-plane state and uploads the resulting artifact. A user cannot verify
that their configured deployment branch builds successfully without starting a
remote deployment.

### Definitions

**Build-only command:** `bonesdeploy build`, which resolves the configured
deployment branch, performs the configured Native or Docker Compose local
build, and packages the deployment artifact. It does not connect to the remote
host, publish secrets, synchronize the control plane, upload an artifact, or
create a release.

**Temporary artifact:** The packaged deployment artifact retained only for the
duration of the build-only command and deleted when that command exits. It is
not an export or a user-managed release file.

### Desired outcome

Running `bonesdeploy build` succeeds only when the same local build and
artifact packaging work that precedes `bonesdeploy deploy` succeeds. It reports
the built revision and artifact size, then leaves no remote or persistent
artifact state.

### Scope

The change adds the top-level `build` CLI command, invokes the existing local
artifact packaging boundary, reports successful completion, covers the command
behavior with tests, and documents the command alongside deployment.

### Constraints

The command must reuse the existing `build::package` implementation so Native
and Compose behavior remains identical to deployment. It must not establish an
SSH connection or invoke any remote operation. Existing deployment behavior
must remain unchanged.

### Exclusions

The command does not retain, export, inspect, sign, upload, or deploy the
artifact. It does not add a remote dry-run, deploy preflight, secret validation,
or a new configuration setting.


## Plan

### Current behavior

`Command::Deploy` dispatches to `commands::deploy::run`. That command loads
the root `.env` configuration and invokes `DeployWorkflow`. Its first operation
calls `build::package`, which acquires the per-site build lock, exports the
configured committed branch, runs the Native or Compose build, and packages the
result as a temporary artifact. The remaining workflow operations handle
production secrets, establish SSH, synchronize control-plane configuration, and
stream the artifact to BonesRemote.

`PackagedArtifact` owns a `NamedTempFile`, so the existing artifact naturally
disappears when it is dropped.

### Intended behavior

`bonesdeploy build` loads the same root configuration as deployment, calls
`build::package`, prints a successful build message containing the artifact
revision and byte length, and returns. Dropping the artifact removes its
temporary file. No remote operation is reachable from this command.

### Approach

Add a top-level `Build` command variant and dispatch it to a focused
`commands::build` module. The module loads the local configuration, calls the
existing artifact packager directly, and presents the artifact manifest's
revision and length. Keep `commands::deploy` and `DeployWorkflow` unchanged so
production deployment retains its existing sequence.

### Responsibilities and boundaries

`cli::args` owns public command parsing, and `cli::dispatch` owns routing the
new command. `commands::build` owns the build-only user workflow. The existing
`build` module remains the sole owner of source export, backend-specific build
execution, packaging, and temporary artifact lifecycle.

### Affected areas

- `crates/bonesdeploy/src/cli/args.rs`
- `crates/bonesdeploy/src/cli/dispatch.rs`
- `crates/bonesdeploy/src/commands/mod.rs`
- `crates/bonesdeploy/src/commands/build.rs`
- Focused BonesDeploy command tests
- `README.md`

### Decisions

The command packages an artifact, rather than stopping after the backend build,
because artifact packaging is part of the deployment's local work and can fail.

The command does not write an artifact to a user-selected path. The requested
behavior is build verification, and retaining transport bytes introduces an
unrequested artifact-export interface and cleanup responsibility.

### Risks

Duplicating the build process would allow build-only and deploy behavior to
drift. Calling only the existing packager avoids that risk. A command wiring
error could accidentally reach production operations; focused tests will prove
the build-only workflow invokes the packager without the deploy workflow.

### Validation

Add focused tests for the build-only command workflow, including branch-build
failure propagation and a successful artifact result. Run the relevant
BonesDeploy test suite, full non-E2E Rust tests, `cargo clippy`, `cargo fmt`,
`shfmt -w .`, the required Python checks, and `git diff --check`. Confirm the
help text and README expose `bonesdeploy build` as the local-only pre-deploy
verification command.


## Tasks

### Implementation

- [x] Add the top-level `build` CLI variant and dispatch it to the build-only
  command module.
- [x] Implement the build-only workflow by loading the root configuration,
  calling `build::package`, and reporting the revision and artifact size without
  invoking deployment operations.
- [x] Add focused tests proving successful local packaging and packaging-failure
  propagation without SSH, secret publication, control-plane synchronization,
  or upload.

### Validation

- [x] Run focused BonesDeploy build-command tests and confirm a successful build
  reaches artifact packaging while a missing configured branch fails locally.
- [x] Run full non-E2E Rust and Python test suites, `cargo clippy`, `cargo fmt`,
  `shfmt -w .`, Python Ruff checks, and `git diff --check` with no warnings or
  failures.

### Completion

- [x] Document `bonesdeploy build` as local-only deployment build verification
  in the README.
- [x] Review the final diff for duplicated build behavior, accidental remote
  calls, persistent artifact output, and unrelated changes.

### Completion notes

`bonesdeploy build` calls the existing `build::package` boundary and reports the
artifact revision and size. It does not invoke deployment operations. The full
non-E2E Rust suite and Python suite passed, alongside Clippy, Rustfmt, Shfmt,
Ruff, and `git diff --check`. The embedded BonesInfra wheel was rebuilt after
the Laravel shared-directory change.
