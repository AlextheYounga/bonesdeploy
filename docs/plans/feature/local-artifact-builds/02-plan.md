# Plan

## Current behavior

`bonesdeploy deploy` in `crates/bonesdeploy/src/commands/deploy.rs` loads the
root `.env`, opens the routine `git` SSH session, synchronizes a sanitized
`RemoteDeploymentConfig`, and invokes `bonesremote deploy --site <site>`. It
does not inspect or build local source, push Git, or stream application bytes.
The SSH helpers can stream remote output and downloads, but stdin helpers accept
a complete byte slice and therefore cannot upload a large artifact without
buffering it in memory.

`bonesremote::commands::deploy::lifecycle::run_full` loads the synchronized
configuration, acquires `SiteMutation`, verifies the site is idle, requires the
dedicated build user's systemd manager and rootless Podman namespace, resolves
the configured branch in the server bare repository to one immutable commit,
and starts `DeploymentLifecycleCoordinator`.

The coordinator stages an exclusive release directory, creates a disposable
context under the site temporary-build root, exports the resolved commit with
server-side `git archive`, runs the backend build, promotes the complete context
into the candidate release, wires shared state, runs prepare scripts, seals and
preflights the release, atomically activates `current`, restarts services, rolls
back a failed activation, and prunes old releases. This common lifecycle is in
`crates/bonesremote/src/commands/deploy/coordinator.rs`.

For native sites, `release/lifecycle/build/run_scripts.rs` chowns the exported
context to `<site>-build`, discovers numbered scripts under
`infra/deployment/build`, derives sanitized build variables, loads committed
`.env.build`, and runs each script in order. `build/container.rs` starts
`buildpack-deps:bookworm` through the build user's systemd manager, mounts source
at `/workspace/source` and the private cache at `/workspace/cache`, copies the
deployment bundle to `/workspace/deployment`, and streams script output to both
the terminal and server logs. Compose sites instead run Docker Compose
validation, pull, and build on the server.

Promotion already provides a suitable post-build boundary.
`release/lifecycle/build/tree.rs` copies only directories, regular files, and
lexically contained relative symlinks into an empty staged release, applies the
runtime identity for prepare, and later seals the tree as `root:<site>`.
Prepare scripts continue from the project deployment bundle and execute on the
host as the runtime user.

`bonesdeploy-core::config::Build` currently carries only the per-script timeout.
`RemoteDeploymentConfig` transports it to BonesRemote, while
`ProvisioningRequest::SiteFields` does not expose build settings to BonesInfra.
The builder image is represented by matching mutable
`docker.io/library/buildpack-deps:bookworm` constants in Rust and Python.

BonesInfra server setup always installs and seeds the shared Podman image store.
Site setup always creates a build user with linger, starts its user manager,
installs cgroup limits and rootless storage configuration, creates its cache,
and verifies Podman. The sudoers policy permits only exact config-sync and
ordinary deploy command forms for the shared `git` identity.

Framework behavior already follows the build/prepare distinction. Vue, Next,
Nuxt, SvelteKit, Laravel, and Rails produce dependencies or application output
during build. Laravel and Rails retain server-side migration prepare work.
Django currently does no Python dependency build; its prepare script creates the
release virtual environment, installs `requirements.txt`, validates Django,
migrates, and collects static files on the server.

## Intended behavior

`BuildMode::Remote` is the serde default. Existing configuration and omitted
build mode retain the current server export, build-user readiness check,
rootless Podman build, logs, and lifecycle. Initialization accepts an explicit
local selection but continues to initialize remote mode during the opt-in
validation phase.

For local mode, `bonesdeploy deploy` resolves the configured local branch to an
exact commit, exports that commit into a private temporary context, and runs the
existing numbered build scripts in direct rootless Podman before connecting to
the server. It uses the pinned Linux `x86_64` Debian builder, the established
`/workspace` paths and environment projection, the configured per-script
timeout, and a project- and build-contract-scoped cache under the local XDG
cache root.

After a successful build, BonesDeploy packages the complete context as the
versioned release artifact, calculates its byte length and SHA-256 digest, opens
SSH, synchronizes configuration, and streams a framed manifest plus artifact to
the artifact form of the existing deploy command. Build or packaging failure
occurs before remote deployment state is created. SSH or protocol failure is
reported and never triggers remote fallback.

BonesRemote acquires the same site lock and idle checks as remote deployment,
confirms local mode and the native backend, resolves the configured branch in
the bare repository, and requires it to equal the manifest revision. It stages
the release and disposable context, receives the bounded stream, validates the
manifest and digest, safely materializes the artifact context, records the
source-exported and built phases, and joins the common lifecycle at promotion.
Prepare, sealing, preflight, activation, restart, rollback, pruning, status,
cancellation, and crash cleanup retain their existing semantics.

Local-mode site provisioning creates the runtime user and normal site layout but
does not create, linger, start, configure, or verify a per-site build user.
Remote doctor and deploy do not require that build user or its rootless Podman
namespace for local mode. Host-wide Podman and the shared image store remain
provisioned because one server may contain both local- and remote-mode sites.

## Approach

Add a lowercase serde `BuildMode` to the canonical `Build` model and carry it
through both deployment and provisioning transports. Missing mode is remote.
Initialization accepts `--build-mode local|remote`; during this feature's
opt-in phase its default remains remote. Validate that local mode is used only
with the native runtime backend.

Move the genuinely shared native build contract from BonesRemote into focused
`bonesdeploy-core` modules: numbered-script discovery, sanitized build-variable
projection, fixed container paths, artifact manifest types, protocol version,
and target platform vocabulary. Keep process execution, Podman commands, SSH,
filesystem mutation, and lifecycle state in their owning binaries.

Replace the mutable builder tag with a project-owned image reference pinned by
digest in both local and remote build paths. Local Podman pulls that exact image
when absent, verifies that it can execute the Linux `x86_64` target, and then
uses `--pull=never` for the build container. Scope the local cache by site,
target platform, and builder digest so incompatible toolchains cannot share one
cache accidentally.

Create a focused local build module in `bonesdeploy`. It uses local Git helpers
to resolve `refs/heads/<configured branch>` and export that commit, never the
working tree. A local container implementation mirrors the existing source,
cache, environment-file, deployment-bundle, workdir, security option, script
ordering, output streaming, timeout, and cleanup behavior without server
`systemd-run` or a dedicated host identity. Container names and temporary files
are unique and removed on every success or error path.

Package the complete built context into a gzip-compressed POSIX tar artifact in
a private local temporary file. Preserve regular files, directories,
executable bits, and relative symlinks because package-manager trees and
application source use them. Do not package the cache, local configuration, Git
metadata, or files outside the clean exported context. Compute the digest over
the exact compressed bytes that will be sent.

Add a length-prefixed protocol header containing strict JSON
`ArtifactManifest`, followed by exactly the declared artifact bytes. Add an SSH
upload helper that accepts an asynchronous reader, writes it to remote stdin,
and concurrently drains remote stdout and stderr. The remote command remains
narrow and site-scoped, and the sudoers expression allows only its exact
argument order.

Add an artifact-receipt module to BonesRemote and route both source forms
through one coordinator. The receiver limits manifest length, compressed bytes,
entry count, path length, regular-file bytes, and total extracted bytes. It
rejects unknown protocol fields or versions, target and builder mismatches,
absolute paths, parent traversal, duplicate/conflicting entries, hard links,
special files, and unsafe symlink targets. It creates directories and regular
files only beneath the exclusive context, records validated symlinks, and
creates those symlinks only after all non-symlink entries are complete so an
archive entry cannot redirect later extraction. A digest or structural failure
deletes the context and staged release through existing abort handling.

Represent the coordinator input as the real domain distinction between remote
source and a local artifact. Remote input performs checkout and build exactly as
today. Artifact input performs receipt and marks the same persisted phases.
After either input reaches `Built`, call the existing promotion-through-cleanup
path without branching again.

Split BonesInfra site identity setup into common runtime identity work and
remote-build-only identity, limits, storage, cache, linger, and Podman work.
Update site diagnostics, manifests, deletion behavior, and tests to accept the
intentional absence of build-user resources for local mode while preserving
their requirements for remote mode.

Keep each framework's existing build and prepare scripts unchanged except for
fixes required to prove existing contracts under both build locations. The
first artifact contains the same complete tree currently promoted remotely;
framework-specific pruning and moving Django prepare work are separate changes.

## Responsibilities and boundaries

`bonesdeploy-core` owns `BuildMode`, the shared build contract, fixed builder
identity and target vocabulary, `ArtifactManifest`, strict serialization, and
protocol framing definitions. It does not execute Git, Podman, SSH, archives,
or deployment state changes.

`bonesdeploy` owns local Git revision resolution and export, local Podman
preflight and build execution, local cache and temporary artifacts, packaging,
digest calculation, user-facing progress and failures, and streaming the framed
artifact over the existing SSH boundary.

`bonesremote` owns the privileged receipt boundary, manifest and pushed-revision
verification, resource and archive validation, safe context materialization,
site locking, persisted deployment phases, promotion, prepare, sealing,
activation, rollback, and cleanup. Artifact receipt is a deployment input, not
a second release lifecycle.

BonesInfra owns mode-aware server provisioning and declared host state. It
creates build identities and rootless Podman resources only for remote-mode
sites while retaining common runtime identities and host-wide support for mixed
build modes.

Project framework build scripts remain the owners of application compilation
and dependency production. Prepare scripts remain the owners of
environment-dependent runtime preparation and persistent-state transitions.

## Affected areas

- `crates/bonesdeploy-core/src/config/model.rs`, `config/transport.rs`, exports,
  paths/build-contract modules, and tests for build mode and artifact protocol.
- `crates/bonesdeploy/src/cli/args.rs`, dispatch, init configuration and tests,
  `commands/deploy.rs`, local Git helpers, SSH streaming, and new focused local
  build/artifact modules.
- `crates/bonesremote/src/cli/`, `commands/deploy/`, `release/lifecycle/`, Git
  verification, artifact receipt, state/status/cancellation behavior, and their
  unit and integration tests.
- `crates/bonesinfra/python/src/bonesinfra/config/`, site user provisioning,
  site setup, manifests, doctor expectations, deletion, sudoers assets, and
  Python tests for mode-aware build resources.
- Builder-image constants and server image-store provisioning for a pinned
  Debian image shared by both build modes.
- `e2e/` harness and framework scenarios for opt-in local build mode; definitions
  are updated but not executed by an agent.
- `README.md`, `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`, architecture
  reference, and security invariants describing artifact provenance, local
  trust, remote receipt, and mode-specific resources.

## Decisions

- Local mode is opt-in in this change. Missing configuration and newly
  initialized projects remain remote until the framework validation phase is
  complete and a separate default-change is approved.
- Git remains authoritative. The artifact revision must equal both the local
  configured branch commit used for `git archive` and the server bare
  repository's configured branch commit.
- Local builds deploy only committed and pushed source. Dirty and untracked
  working-tree content is irrelevant because it is not part of the archive.
- The first artifact is the complete post-build context. This maximizes parity
  with current promotion before any framework-specific size optimization.
- Prepare remains remote. Local build mode therefore removes native build
  scripts from production but does not claim that every framework performs zero
  installation work during prepare; Django retains its existing behavior.
- Local mode supports the native runtime backend only. Compose continues its
  existing server-side image pull and build behavior.
- Local and remote runners share data contracts but keep separate process
  implementations because server systemd/build-user execution does not belong
  in the local CLI.
- The artifact is gzip-compressed tar with strict typed framing and SHA-256.
  BonesRemote streams it into an exclusive disposable context rather than
  retaining an artifact archive on the server.
- Safe extraction is enforced before promotion even though SSH authenticates
  the sender. A compromised deploy identity must not turn archive parsing into
  root filesystem writes.
- Local-mode sites omit per-site build users and rootless Podman sessions. The
  host-wide Podman baseline remains to support mixed-mode hosts and remote
  builds.
- Local failure never falls back. Operators change the configured build mode
  deliberately when they need the remote path.

## Risks

- A command or environment mismatch between local and remote runners can make
  the same build scripts behave differently. Shared contract tests and
  command-construction parity tests must cover mounts, variables, workdir,
  image, target, script order, timeout, and cleanup.
- The current builder image is mutable. Failing to pin and validate one digest
  would allow local and remote builds to use different Debian userspaces under
  the same name.
- Local Podman may run on a non-`x86_64` machine. Platform verification must
  fail before building unless Podman can execute the required Linux `x86_64`
  image correctly.
- Complete contexts can contain large `node_modules`, `vendor`, or
  `vendor/bundle` trees. Local builds remove production compilation pressure but
  can increase transfer time; clear progress, length limits, temporary-file
  cleanup, and later measured pruning are necessary.
- Archive extraction runs inside the privileged deployment process. Incorrect
  path, symlink, duplicate-entry, special-file, or resource-limit handling can
  become a root write or disk-exhaustion vulnerability.
- A local commit that has not been pushed could produce release metadata with no
  server-side provenance. Exact branch-commit comparison must happen before
  staging useful release content.
- Adding build mode to both Rust and Python transports can accidentally skip
  build-user provisioning for existing sites. Missing mode must deserialize as
  remote at every boundary, with regression tests for old descriptors.
- Local mode removes build-user readiness from deployment. Doctor, status,
  cancellation, deletion, and cleanup must not assume a build container or user
  exists, while remote mode must retain those checks.
- Build logs move from server files to local command output. Failures must remain
  clear and identify the exact script, but centralized local log retention is
  outside this change.
- Django continues dependency installation during prepare, so local mode does
  not remove all deployment-time resource consumption for that framework. The
  documentation must state this precisely.

## Validation

- Core tests prove missing build mode is remote, explicit modes serialize and
  deserialize strictly, local mode rejects the Docker backend, artifact
  manifests reject unknown fields and unsupported versions, and framing rejects
  oversized or truncated headers.
- Local Git tests prove the configured branch commit is resolved, the exported
  source is committed-only, and dirty, ignored, and untracked files never enter
  the build context.
- Local build tests prove Podman commands use the pinned image and Linux
  `x86_64` target, expose only the established mounts and variables, run scripts
  lexically, apply timeouts, reuse only the scoped cache, stream failures, and
  remove containers and protected environment files on every path.
- Packaging and receipt tests round-trip regular files, executable files,
  directories, and safe relative symlinks. They reject digest and length
  mismatches, truncation, trailing bytes, path traversal, absolute paths,
  escaping symlinks, hard links, devices, FIFOs, duplicate paths, oversized
  entries, too many entries, and unsupported targets or builder identities.
- BonesRemote lifecycle tests prove artifact deploy uses the normal site lock,
  requires an idle local-mode native site, verifies the pushed configured
  branch commit, skips build-user and Podman readiness, records normal phases,
  joins promotion and prepare, cleans interrupted uploads, and restores the
  previous release after failed activation.
- Remote-mode regression tests prove existing deploy command construction,
  build-user readiness, checkout, Podman build, status, cancellation, and
  rollback remain unchanged for omitted and explicit remote mode.
- BonesInfra tests prove local-mode sites retain runtime identities and release
  layout while omitting build-user linger, slices, rootless storage, cache, and
  verification; remote-mode and old requests retain every current resource.
- Ignored E2E definitions cover local artifact deploys for Laravel, Next server
  and static, Nuxt server and static, SvelteKit, Vue, Django, and Rails, including
  a second release and failed-activation rollback. A human runs and records this
  matrix before a separate change makes local mode the new-project default.
- Run focused Rust and Python tests, then `cargo test --workspace --exclude
  e2e`, `cargo clippy --workspace --exclude e2e`, `cargo fmt`, `shfmt -w .`,
  `ruff check .`, `ruff format .`, and the Python test suite. Resolve every
  warning and error and review the final diff for secret exposure, unsafe
  extraction, broad sudo access, silent fallback, and remote-mode regressions.
