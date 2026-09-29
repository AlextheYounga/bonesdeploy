# Plan

## Current Behavior

The completed artifact foundation supports `BuildMode::Remote` and
`BuildMode::Local`, with omitted mode defaulting to remote. Local mode exports a
configured branch commit, removes the root `.env`, runs numbered scripts in a
rootless Podman container, packages a strict gzip tar artifact, and streams it
to BonesRemote. Remote mode preserves server checkout and rootless-Podman build
execution.

BonesRemote accepts ordinary source deployments and framed artifact deployments.
Both converge on `DeploymentLifecycleCoordinator` for promotion, shared-state
wiring, prepare, sealing, preflight, activation, restart, rollback, pruning, and
cleanup. Artifact receipt validates manifest identity, revision, target, builder,
length, digest, paths, entry types, symlinks, file count, and extracted size.

BonesInfra provisions build users, linger, cgroup limits, rootless storage,
caches, and the shared Podman image store for remote-mode sites. Local-mode sites
omit per-site build resources, but mixed-mode support keeps server-wide builder
dependencies and mode-aware diagnostics.

Production system diagnostics currently recognize Debian and Ubuntu but do not
express the settled Debian 12+ and Ubuntu 24.04+ minimum-version contract.

## Intended Behavior

Native deployment has one path. `bonesdeploy deploy` resolves and exports the
configured branch commit, runs its build scripts locally in Docker, creates the
release artifact, synchronizes deployment configuration, and streams the framed
artifact to BonesRemote. There is no build-mode setting and no remote source
deployment path to select or fall back to.

The Docker runner uses the pinned Linux `x86_64` Debian builder image, fixed
`/workspace` paths, project- and contract-scoped cache, ordered scripts,
per-script timeout, streamed output, and unconditional container cleanup. It
passes only fixed non-secret build-contract metadata. It does not load
`.env.build`, inherit the host environment, or expose host credentials, home
directories, production configuration, privileged mode, devices, or the Docker
socket.

BonesRemote accepts the artifact form as the only native deployment input. It
retains exact pushed-revision verification, bounded safe receipt, persisted
deployment phases, promotion, prepare, activation, rollback, pruning,
cancellation, and cleanup. Native deployment no longer checks out or builds
source on production.

BonesInfra provisions only runtime identities and release infrastructure for
native sites. It does not install native builder images or create application
build users, linger sessions, rootless container storage, or server build caches.

Provisioning and diagnostics require Debian 12 or newer or Ubuntu 24.04 or newer
for production hosts. Unsupported systems fail with an explicit supported-host
message before site mutation proceeds.

## Approach

Remove `BuildMode` from canonical configuration, deployment transport,
provisioning requests, initialization arguments, generated environment files,
and diagnostics. Delete remote-default deserialization and mixed-mode branches
rather than retaining compatibility aliases. Route native deployment directly
through the artifact path.

Replace the Podman command implementation in the local build module with a
focused Docker implementation. Verify that Docker is reachable, uses Linux
containers, can pull the exact pinned image, and can execute the required
`linux/amd64` target. Continue to use `--pull=never` after the verified pull and
retain `no-new-privileges`, narrow mounts, timeout handling, output streaming,
and unconditional cleanup. Do not introduce a generic multi-engine abstraction.

Reduce the build environment constructor to fixed public contract values owned
by `bonesdeploy-core`. Remove `.env.build` parsing and projection from both the
local runner and obsolete remote runner. Build scripts that require private
dependency credentials are unsupported by this release.

Remove the remote-source coordinator input and native checkout/build stages.
Keep Git revision resolution on the server solely to prove that the uploaded
artifact corresponds to the configured pushed branch. Preserve artifact safety
checks and all lifecycle behavior after validated receipt.

Remove BonesInfra native build-user creation, linger, cgroup slice, rootless
storage, cache, Podman readiness, and native builder image-store requirements.
Retain runtime users, shared state, service configuration, release ownership,
and every resource required after artifact import. Rebuild the packaged
BonesInfra wheel after Python changes.

Make supported production distributions a shared explicit policy in BonesInfra
provisioning and BonesRemote system diagnostics. Parse distribution identity and
major version, accept Debian major versions at least 12 and Ubuntu releases at
least 24.04, and reject all others with the same documented support statement.

Update ignored native framework E2E setup to use the sole artifact path without
mode configuration. Preserve first deployment, second release, and failed
activation rollback scenarios. Do not execute ignored E2E tests during agent
implementation.

## Responsibilities And Boundaries

`bonesdeploy-core` owns the fixed build contract, builder identity, target
platform, artifact manifest, and framing types. It contains no build-mode or
container-engine selection.

`bonesdeploy` owns clean local Git export, Docker preflight and execution, local
cache, artifact packaging, progress and failures, and SSH upload. Docker is a
local execution dependency, not a production dependency.

`bonesremote` owns pushed-revision verification, privileged artifact receipt,
safe extraction, site locking, lifecycle state, promotion, prepare, activation,
rollback, pruning, cancellation, and cleanup. It never executes native build
scripts.

BonesInfra owns production operating-system validation and the runtime resources
required to host imported artifacts. It does not provision native build
resources.

Framework build scripts own compilation and dependency production without
secret inputs. Prepare scripts own environment-dependent runtime preparation
and persistent-state transitions.

## Affected Areas

- Build configuration and transport models in `bonesdeploy-core`, including
  removal of `BuildMode` and user-defined build-environment projection.
- BonesDeploy init, deploy, doctor, local build, Git, artifact, and tests.
- BonesRemote deploy entry points, coordinator inputs, native checkout/build
  modules, doctor, cancellation, security collection, and tests.
- BonesInfra request/context models, user and image provisioning, manifests,
  supported-system validation, sudoers assets, tests, and generated wheel.
- Ignored native framework E2E setup and validation documentation.
- README, context, architecture, security, support, and upgrade documentation.

## Decisions

- This is a breaking release. Compatibility with remote-build configuration and
  previously provisioned native build resources is not retained.
- Docker is the only local build engine. Podman is not an alternative or
  fallback, and no container-engine abstraction is introduced.
- Local native artifacts are the only native deployment input. Production hosts
  never build native application source.
- Build scripts receive fixed public contract metadata only. `.env.build`, host
  environment variables, production variables, and credentials are excluded.
- Docker's local daemon trust is accepted. The build container still receives no
  privileged mode, Docker socket, host home directory, or credential mounts.
- Git remains authoritative for revision provenance, and server-side branch
  comparison remains mandatory before artifact promotion.
- Production support is exactly Debian 12+ and Ubuntu 24.04+. Broader Linux
  support is not inferred from Docker build-host portability.
- Compose artifact support and previous-installation migration are separate
  future changes, not compatibility requirements for this release.

## Risks

- Docker daemon access is highly privileged on Linux. Command construction must
  never expose the socket or permit project-controlled Docker arguments.
- Supply-chain code can read source, mutate artifacts, use the network, consume
  resources, and poison persistent caches. Secret exclusion limits credential
  theft but does not make application dependencies trustworthy.
- Removing `.env.build` can break projects that treated it as build input. The
  breaking release must report the unsupported contract clearly rather than
  silently dropping required private values.
- Docker Desktop and non-`x86_64` machines depend on Linux VM and emulation
  behavior. Passing the execution probe establishes target capability but does
  not constitute native Windows support.
- Removing server build resources can accidentally delete helpers still used
  for runtime release ownership or Compose. Final dependency review must
  distinguish native build-only behavior from shared lifecycle behavior.
- Distribution version parsing errors can reject supported hosts or accept
  unsupported hosts. Tests must cover exact boundaries and malformed metadata.
- Complete artifacts remain large. Docker does not change upload, temporary
  storage, or remote extraction pressure.

## Validation

- Core and configuration tests prove there is no build-mode field, default, or
  remote compatibility path and that build environment projection contains only
  the fixed public contract.
- Local build tests prove Docker commands use the pinned digest and
  `linux/amd64`, reject unavailable or non-Linux Docker engines, expose no Docker
  socket or secret-bearing paths, apply timeouts, stream output, scope caches,
  and remove containers and protected temporary files on every path.
- Deploy tests prove every native deployment builds before SSH, uploads the
  framed artifact, and has no remote-build fallback or ordinary native source
  deploy command.
- BonesRemote tests prove artifact deployment retains revision verification,
  bounded safe receipt, lifecycle phases, prepare, activation rollback,
  cancellation, and cleanup without build-user, Podman, or checkout readiness.
- BonesInfra tests prove native sites have runtime resources but no build user,
  linger, build slice, rootless storage, build cache, Podman verification, or
  builder image-store requirement.
- Production-platform tests accept Debian 12 and newer and Ubuntu 24.04 and newer
  while rejecting older versions, other distributions, missing identity, and
  malformed versions before mutation.
- Ignored E2E definitions cover first deploy, second release, and failed
  activation rollback for the supported native framework matrix. They compile
  but are not agent-executed.
- Run focused Rust and Python tests, the full non-E2E suites, Clippy, Rustfmt,
  shfmt, Ruff checks and formatting, regenerate and validate the BonesInfra
  wheel, run `git diff --check`, and review the final diff for obsolete remote or
  Podman behavior, secret exposure, unsafe receipt, and unsupported-host drift.
