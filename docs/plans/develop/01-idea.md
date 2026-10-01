# Idea

## Request

Add a build command that performs the same local build used for deployments
without pushing an artifact to the remote server.

## Problem

The only public command that runs the production build also synchronizes
control-plane state and uploads the resulting artifact. A user cannot verify
that their configured deployment branch builds successfully without starting a
remote deployment.

## Definitions

**Build-only command:** `bonesdeploy build`, which resolves the configured
deployment branch, performs the configured Native or Docker Compose local
build, and packages the deployment artifact. It does not connect to the remote
host, publish secrets, synchronize the control plane, upload an artifact, or
create a release.

**Temporary artifact:** The packaged deployment artifact retained only for the
duration of the build-only command and deleted when that command exits. It is
not an export or a user-managed release file.

## Desired outcome

Running `bonesdeploy build` succeeds only when the same local build and
artifact packaging work that precedes `bonesdeploy deploy` succeeds. It reports
the built revision and artifact size, then leaves no remote or persistent
artifact state.

## Scope

The change adds the top-level `build` CLI command, invokes the existing local
artifact packaging boundary, reports successful completion, covers the command
behavior with tests, and documents the command alongside deployment.

## Constraints

The command must reuse the existing `build::package` implementation so Native
and Compose behavior remains identical to deployment. It must not establish an
SSH connection or invoke any remote operation. Existing deployment behavior
must remain unchanged.

## Exclusions

The command does not retain, export, inspect, sign, upload, or deploy the
artifact. It does not add a remote dry-run, deploy preflight, secret validation,
or a new configuration setting.
