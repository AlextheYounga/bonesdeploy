# Idea

## Request

Add opt-in local artifact builds for native BonesDeploy projects. The local
BonesDeploy CLI exports the configured Git revision, runs the project's existing
build scripts in rootless Podman using the same Debian build contract as the
server, and uploads the completed release artifact. BonesRemote retains control
of staging, shared state, prepare scripts, release sealing, activation, service
verification, rollback, and pruning.

After local artifact builds have been validated across the supported framework
matrix, a separate change can make them the default for newly initialized
projects. Existing projects must not silently change build modes.

## Problem

BonesRemote currently exports source and performs every application build on
the production server. Modern framework builds can consume enough memory, CPU,
I/O, and temporary storage to make a small server unresponsive even when the
build user has cgroup limits. The production host also carries per-site rootless
Podman state, build caches, and build-user sessions solely to produce release
files that could be produced before deployment.

The existing local CLI cannot execute the established build-script contract,
represent a built release as a verified transport, or submit one to the shared
remote release lifecycle. Moving builds without those boundaries would risk
deploying an uncommitted tree, exposing local secrets, accepting an unsafe
archive as root, or creating a second activation and rollback implementation.

## Definitions

**Build mode:** The configured location where native application build scripts
run. `remote` preserves the current BonesRemote build-user pipeline. `local`
runs those scripts through rootless Podman on the machine executing
`bonesdeploy`. Build mode does not change the application runtime backend or
the location of prepare scripts.

**Local build:** A build performed by the local CLI from a clean Git archive of
the exact configured branch commit. It does not build the current working tree,
include ignored or untracked files, or consume production runtime secrets.

**Release artifact:** A versioned, checksummed transport containing the complete
post-build source tree that the existing remote build stage would otherwise
promote. It includes committed deployment scripts needed by server-side prepare
and excludes Git metadata, the local root `.env`, production `shared/` state,
server control-plane state, and the local build cache.

**Artifact manifest:** The strictly parsed metadata sent before artifact bytes.
It identifies the protocol version, site, exact Git revision, target platform,
builder image, archive format, declared byte length, and SHA-256 digest.

**Prepare:** The existing server-side phase that runs numbered prepare scripts
against the promoted candidate as the runtime user after shared paths are
wired. Database migrations, runtime-state changes, Laravel optimization, Rails
migrations, and Django virtual-environment installation remain prepare work.

**Remote build:** The existing build-user and rootless-Podman pipeline on the
deployment server. It remains available as an explicit configured mode and is
never selected as an automatic fallback after a local build failure.

## Desired outcome

A native project configured for local builds can run `bonesdeploy deploy` and
produce its release in local rootless Podman before opening the deployment
transaction on the server. BonesRemote verifies that the artifact represents
the exact commit currently pushed to the configured remote branch, validates
and imports the artifact into disposable staging state, and then uses the
existing prepare, preflight, activation, restart, rollback, and cleanup paths.

A local build or packaging failure leaves the production server untouched. An
upload, validation, prepare, or activation failure leaves no partial usable
release and preserves the existing failed-release and rollback semantics. No
failure silently retries the build on production.

Projects without an explicit local build selection continue to build remotely.
Local mode does not require a per-site rootless Podman namespace on the server.

## Scope

- A typed `local` or `remote` native build mode with `remote` as the
  compatibility default and an explicit initialization option for local mode.
- A shared build contract for numbered scripts, sanitized build environment,
  fixed container paths, target platform, timeout behavior, and a pinned Debian
  builder image.
- Local clean-revision export, rootless-Podman build execution, persistent local
  tool cache, artifact packaging, digest calculation, and streaming SSH upload.
- A versioned artifact manifest and framed upload protocol shared by BonesDeploy
  and BonesRemote.
- A local-artifact input to the existing BonesRemote deployment lifecycle,
  including site locking, pushed-revision verification, bounded receipt, safe
  extraction, promotion, prepare, activation, rollback, and cleanup.
- Build-mode-aware provisioning and diagnostics so local-mode native sites do
  not provision or require a per-site remote build user or rootless Podman
  session.
- Focused tests for configuration compatibility, build contract parity, local
  command construction, protocol validation, archive safety, lifecycle failure
  handling, and native remote-build regressions.
- Ignored end-to-end coverage definitions for Laravel, Next, Nuxt, SvelteKit,
  Vue, Django, and Rails local artifact deployments.
- User, architecture, and security documentation for both build modes.

## Constraints

- Git remains the source-ingress and revision-provenance boundary. Local builds
  use the configured local branch commit, and BonesRemote accepts the artifact
  only when the configured branch in the server bare repository resolves to the
  same commit.
- Local builds run only committed `infra/deployment/build/NN_*.sh` files, in the
  same order and with the same sanitized variables and `/workspace` contract as
  remote builds.
- The builder targets Linux `x86_64` with the project-owned pinned Debian image.
  The local Podman installation may use native execution or configured
  emulation, but the resulting container platform must match the server target.
- The local root `.env`, decrypted production environment, SSH configuration,
  Git credentials, host filesystem outside the exported context and cache, and
  server state are never mounted into the build container or packaged.
- Artifact input is untrusted at the root boundary even though it arrives over
  authenticated SSH. BonesRemote independently enforces framing, size, digest,
  entry type, path, symlink, file-count, and extracted-size limits before
  promotion.
- Artifact import extends the existing deployment lifecycle and `SiteMutation`
  lock. It does not write directly to `releases/` or introduce separate
  activation, rollback, or pruning behavior.
- Prepare scripts remain server-side and retain their current runtime identity,
  environment, shared-state access, and ordering.
- Remote mode remains behaviorally compatible and cannot depend on locally
  installed Podman.
- E2E tests are authored for human execution but are not run by an agent unless
  explicitly requested.
- Implementation begins only after a human approves these planning documents.

## Exclusions

- Making local builds the default for new or existing projects in this change.
- Automatically falling back to a remote build after any local failure.
- Docker Compose image builds, registry publishing, Docker image transport, or
  changes to the Docker runtime backend.
- Framework-specific artifact pruning or replacing the complete post-build tree
  with output-only packages.
- Moving prepare work into the local build. Django dependency installation and
  virtual-environment creation therefore remain server-side in this change.
- Deploying dirty, uncommitted, untracked, or unpushed source.
- Remote artifact storage, resumable uploads, artifact registries, signing,
  provenance services, SBOM generation, or reproducible-build certification.
- Removing the host-wide Podman package or shared image store, which remain
  necessary for projects configured for remote builds.
