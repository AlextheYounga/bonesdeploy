# Plan

## Current Behavior

Native deployment resolves and exports the configured local branch commit,
builds numbered scripts in local Docker, packages the complete resulting tree,
synchronizes deployment configuration, and streams a framed artifact to
BonesRemote. The Docker runner uses a pinned Linux `x86_64` image, scoped cache,
explicit public build environment, per-script timeout, detached container,
repeated Docker preflight, and ownership-restoring cleanup.

BonesRemote still resolves the configured branch in a production bare
repository before accepting a native artifact and rejects an artifact whose
revision does not match that separately pushed ref. Docker Compose deployment
still takes the ordinary no-artifact path: BonesRemote exports the pushed source
revision, runs Compose `config`, `pull`, and `build` on production, then promotes
and activates the release.

Both paths converge on `DeploymentLifecycleCoordinator` for staging, candidate
materialization, shared-state wiring, prepare, sealing, preflight, activation,
restart, rollback, pruning, cancellation, and cleanup. The persisted phases
still use source-export and build terminology even when those operations already
happened locally.

BonesInfra provisions no native build users, caches, Podman state, or builder
image store. It still provisions a bare repository for every site and carries
repository and branch fields through paths, contexts, manifests, deletion, and
diagnostics. It provisions the production runtime, services, security policy,
backups, and operational infrastructure required after a release arrives.

Framework artifacts are complete only inconsistently. Laravel installs Composer
dependencies locally, while Django installs application requirements during
remote prepare. Node-based artifacts retain build-time dependency trees and
caches even when their production runtime needs only compiled or standalone
output.

## Intended Behavior

All application builds happen locally. `bonesdeploy deploy` resolves and exports
the configured branch commit once, constructs the selected runtime's complete
artifact with local Docker, synchronizes deployment configuration, and streams
one framed deployment to BonesRemote. Production never checks out application
source, resolves an application Git ref, downloads application dependencies,
pulls Compose images, or builds application code or images.

Deployment configuration contains only release retention and the selected
backend's remotely consumed values. Native configuration carries the web root
and optional Ruby version needed by remote Rails migrations. Docker
configuration carries the optional ingress port used by diagnostics and the
Compose startup timeout. Framework identity, Node and Python build versions,
permission defaults, and arbitrary runtime extras remain local or in the
separate BonesInfra provisioning request.

Native artifacts contain a complete runnable release tree. Framework build
scripts install dependencies and compile all deterministic output locally, then
remove build-only dependency trees, caches, and numbered build scripts that the
runtime does not need. Remote prepare performs only production-state operations
such as shared-path setup, migrations, production-secret-based configuration,
and runtime validation.

Compose artifacts contain the committed release tree, a generated BonesDeploy
override, an image inventory, and a bounded Docker image archive. BonesDeploy
runs Compose validation, pull, and build locally for `linux/amd64` with the same
stable project name used remotely. It discovers every resulting service image,
assigns an immutable tag derived from site, service, and revision, and writes an
override selecting those tags. Production validates and loads the archive and
starts the stack with `--no-build --pull never`.

The production release directory retains the original Compose files for
operational inspection and bind-mounted release content. The generated override
is always passed last and is owned by the release protocol, not the project.
Rollback activates the previous release directory and its immutable image tags.
Failed-release cleanup and release pruning remove tags owned solely by the
discarded release while allowing Docker to retain shared layers still referenced
by another release.

Local Git is authoritative for source selection. The full commit hash remains in
the artifact manifest, release name, deployment state, status, and logs as audit
metadata. Production does not maintain an application repository or require a
first push. Compose and native sites use the same artifact-only BonesRemote
deploy entry point.

The local Docker runner provides a clean compatible build environment rather
than production tenant isolation. It retains the pinned image and platform,
explicit environment, absence of production secrets and ambient variables,
narrow project/cache inputs, timeout, streamed output, and reliable cleanup. Its
command lifecycle is simplified where direct foreground Docker execution can
replace detached containers, redundant probes, copied deployment trees, and
ownership-repair machinery without weakening those guarantees.

BonesRemote treats all artifact bytes as untrusted privileged input. It retains
strict framing, identity, digest, path, entry-type, symlink, conflict, file-count,
compressed-size, and expanded-size enforcement. The manifest identifies artifact
kind and enforceable payload contents. It does not claim that a client-supplied
builder image identifier proves build provenance.

BonesInfra provisions the production machine and execution environment only. It
retains supported-system validation, server hardening, deploy/runtime identities,
BonesRemote and constrained sudo, release/shared/configuration paths, language
runtimes, Nginx, systemd, AppArmor, Docker Engine and Compose runtime, backups,
SSL, tunnels, manifests, deletion, and patches. It removes application bare
repositories and repository-derived configuration, readiness, and inventory.

## Approach

Extend the artifact protocol with an explicit native or Compose artifact kind
and kind-specific validated metadata. Keep the common framed manifest, declared
length, SHA-256, and safe filesystem extraction. Add a bounded Compose image
payload and image inventory whose service names and release tags are validated
against the site and revision before invoking Docker.

Refactor local source export into the common start of every deployment. Native
projects execute their numbered build scripts and package the pruned result.
Compose projects execute Docker Compose against the exported context with
controlled `linux/amd64` defaults and `.env.build`, collect both locally built
and pulled service images, add immutable release tags, generate the protected
override, and save those exact tags into the artifact. Production `.env` remains
absent from every local build.

Make framework build and prepare scripts obey the complete-artifact boundary.
Move Django dependency installation into local build output. Preserve remote
migrations and environment-dependent framework work. Add framework-specific
pruning after successful builds, including removal of Laravel root
`node_modules`, Next/Nuxt/SvelteKit/Vue build caches and unnecessary dependency
trees according to their runtime output, and numbered native build scripts.

Replace BonesRemote's source/artifact input split with one artifact input.
Receive and validate the payload, materialize the release tree, load and verify
Compose images when present, promote the candidate, wire shared state, run
native prepare where applicable, seal, preflight, activate, and restart. Remove
server repository resolution, archive export, Compose pull/build, and source
deployment routing. Start Compose with builds and pulls disabled.

Replace the control-plane snapshot's complete local `Runtime` value with one
serde-tagged remote runtime enum. Convert it back into the minimal site runtime
state required by the existing release lifecycle at the BonesRemote boundary.
Remove the unused Python-version prepare environment and tests that preserve
arbitrary runtime extras. Reject old broad snapshots under the breaking-release
compatibility policy rather than retaining a legacy parser.

Rename persisted phases and coordinator operations around remote facts rather
than historical source/build operations. Preserve compatibility only with state
written by the current breaking-release branch where concrete recovery requires
it; do not retain obsolete application build behavior.

Remove repository fields and resources from canonical remote/provisioning
transports and BonesInfra. Site setup creates control-plane, project, release,
shared, placeholder, backup, and runtime resources without a bare repository.
Local initialization no longer creates a deployment Git remote, and setup and
doctor no longer instruct users to make a first push.

Keep the existing deploy SSH principal and narrowly anchored sudo policy, but
rename Git-specific terminology where it describes artifact transport rather
than source hosting. Rebuild the packaged BonesInfra wheel after Python changes.

## Responsibilities And Boundaries

`bonesdeploy-core` owns artifact framing, enforceable manifest types, target
platform, explicit build environment vocabulary, and shared validation. It does
not model server application repositories or unverifiable build attestation. It
also owns the narrow backend-specific remote deployment descriptor.

`bonesdeploy` owns local committed-source selection, native and Compose build
execution, build cache, framework pruning, Compose image collection and immutable
tagging, artifact packaging, progress, failures, and SSH upload.

`bonesremote` owns privileged artifact receipt, safe extraction, Compose image
loading, site locking, lifecycle state, promotion, production preparation,
activation, rollback, image/release pruning, cancellation, and cleanup. It never
builds application source or pulls application images.

BonesInfra owns production operating-system policy and resources required to run
imported releases. It installs runtime interpreters and services but does not
install application dependencies into a release or provision source/build
infrastructure.

Framework build scripts own application dependency installation, compilation,
runtime artifact layout, and removal of build-only content. Prepare scripts own
production-state transitions and environment-dependent runtime preparation.

## Affected Areas

- Artifact and configuration contracts in `bonesdeploy-core`, including artifact
  kind, Compose image inventory, repository-field removal, narrow remote runtime
  configuration, and lifecycle terms.
- BonesDeploy init, deploy, doctor, local Docker execution, Git export, native
  packaging, Compose build/image export, progress, and tests.
- Framework build/prepare assets for complete runtime output and pruning.
- BonesRemote deploy CLI and coordinator, artifact receipt, Compose image load,
  runtime start, rollback, failed-release cleanup, pruning, doctor, state, and
  tests.
- BonesInfra request/context/path models, site directories, manifests, deletion,
  runtime provisioning, packages, tests, and generated wheel.
- E2E setup and scenarios for artifact-only native and Compose first deploy,
  subsequent deploy, failed activation, rollback, and pruning.
- README, context, architecture, security, framework, and operational docs.

## Decisions

- Native and Docker Compose releases build locally and deploy only through the
  artifact protocol.
- Production application bare repositories and first-push workflows are removed.
- Local committed Git state is the sole source-selection boundary; revision is
  retained as audit metadata rather than compared with a production ref.
- Compose service images travel over SSH in the deployment artifact. A registry
  is not part of this change.
- Compose images use immutable release-specific tags, and remote Compose start
  disables both builds and pulls so rollback remains release-correct.
- `.env.build` is public build configuration for native tools and Compose build
  interpolation. Production `.env` and ambient host variables remain excluded.
- Complete artifacts include application dependencies. Remote prepare cannot
  install or compile them.
- BonesRemote config sync carries only values with current remote consumers;
  BonesInfra provisioning and production secrets remain separate contracts.
- Framework-specific artifact pruning is required, not deferred.
- Local Docker is a compatibility/reproducibility dependency, not a production
  isolation boundary. Secret exclusion and narrow tool-owned inputs remain
  mandatory.
- Artifact receipt remains a strict privileged trust boundary.
- BonesInfra remains the runtime and machine provisioner; friendly public CLI
  commands and operational inspection remain in scope and are not cleanup
  targets.
- Production support remains Debian 12+ or Ubuntu 24.04+ on `x86_64`.
- Previous-installation migration, registries, private build credentials,
  signing, SBOMs, and resumable upload remain separate changes.

## Risks

- Compose image archives can be substantially larger than native filesystem
  artifacts. Limits must remain explicit while accommodating a realistic
  multi-service release, and interrupted uploads must leave removable state.
- Compose files can vary topology through interpolation and profiles. Public
  build inputs must determine the complete image topology; production secrets
  cannot select an image that was absent from the artifact.
- Mutable Compose tags would silently break rollback. Generated overrides,
  remote start commands, pruning, and tests must prove release-specific image
  identity end to end.
- Loading images mutates Docker state before activation. Abort and pruning paths
  must remove release-owned tags without deleting layers or images used by the
  active or previous release.
- Framework pruning can remove runtime-required dependencies. Each framework
  needs observable runtime-output tests rather than a universal exclusion rule.
- Locally built native extensions must be compatible with the supported runtime
  and production ABI. Builder runtimes and production runtime versions must stay
  aligned.
- Simplifying Docker execution can regress timeout, output, cache, ownership, or
  cleanup behavior. Tests must preserve outcomes rather than the detached
  container implementation.
- Removing repository state touches provisioning, deletion, doctor, setup,
  configuration, and recovery assumptions across Rust and Python.
- Narrowing the persisted control-plane snapshot can expose files written by a
  pre-breaking-release binary. This is intentional under the no-compatibility
  policy; every deployment synchronizes the new descriptor before invoking the
  artifact lifecycle.
- Omitting a remotely consumed backend value would break prepare, preflight,
  Compose startup, or diagnostics. Focused consumer and serialization tests must
  cover every retained field and prove unrelated fields are absent.

## Validation

- Core protocol tests round-trip native and Compose manifests and reject wrong
  kinds, sites, revisions, lengths, digests, image inventories, tags, paths,
  entry types, conflicts, symlinks, and resource-limit violations.
- Native build tests prove complete dependency output, framework-specific
  pruning, explicit build variables, secret exclusion, target compatibility,
  timeout, output, cache, ownership, and cleanup.
- Compose tests prove local `config`, pull, and build use the exported commit,
  controlled environment, stable project name, and `linux/amd64`; every service
  image is inventoried, immutably tagged, saved, loaded, and selected by the
  generated override.
- Remote Compose tests prove deployment and service restart use `--no-build
  --pull never`, perform no network pull or build, preserve release-specific
  rollback, and remove only pruned or failed release tags.
- Lifecycle tests prove both artifact kinds share receipt, locking, promotion,
  activation, failure rollback, cancellation, pruning, and cleanup without
  source-export or production-build phases.
- BonesInfra tests prove no backend provisions or declares a bare repository,
  branch ref, build user, builder cache, builder image, or application build
  operation while all runtime, service, security, backup, SSL, manifest, and
  deletion responsibilities remain intact.
- CLI and documentation tests prove initialization, setup, doctor, and deploy no
  longer create a deployment Git remote or require a first push, while public
  operational commands remain available.
- Remote configuration tests prove native and Docker descriptors carry exactly
  their backend-specific values, omit local build and provisioning data, derive
  the required internal site runtime, and reject old broad snapshot shapes.
- Remote prepare tests prove Rails still receives its Ruby version while no
  Python build version or arbitrary runtime extra is projected remotely.
- Ignored E2E definitions cover native and Compose first deploy, second release,
  failed activation, rollback, and old-release/image pruning. They compile but
  are not agent-executed.
- Run focused Rust and Python tests, full non-E2E suites, Clippy with warnings
  denied, Rustfmt, shfmt, Ruff checks and formatting, generated-wheel validation,
  `git diff --check`, and final architecture/security review.
