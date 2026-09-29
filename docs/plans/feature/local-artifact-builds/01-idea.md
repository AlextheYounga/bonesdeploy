# Idea

## Request

Make local artifact builds the only BonesDeploy application build path in the
next breaking release. The local BonesDeploy CLI builds the committed
application in Docker and uploads the completed native release or Compose image
artifact. BonesRemote does not retain a server-side application build path.

Synchronize only the release and backend-specific runtime values BonesRemote
still consumes. Local build configuration and arbitrary framework settings do
not belong in the remote control-plane snapshot.

Support only Debian 12 or newer and Ubuntu 24.04 or newer as production host
operating systems. Existing installations and configuration formats do not
constrain this breaking release; migration support can be designed separately.

## Problem

Building applications on production hosts consumes scarce CPU, memory, storage,
and I/O, and requires build users, rootless container state, caches, and builder
images on every server. Retaining both local and remote build modes would keep
that operational and security complexity even though the project does not need
compatibility at this stage.

The first local implementation established the artifact protocol and shared
release lifecycle, but deliberately retained remote builds and used rootless
Podman for compatibility. Docker is more widely available on developer machines
and is an acceptable local trust dependency when the build receives no host or
production secrets and no privileged host interfaces.

Production platform support is also broader than the environments the project
intends to validate. A narrow Debian and Ubuntu contract makes provisioning,
diagnostics, service management, and operational documentation explicit.

## Definitions

**Local artifact build:** The sole application build path. BonesDeploy exports
the exact configured branch commit and builds it with local Docker. Native
projects run numbered build scripts and package the resulting runtime tree.
Compose projects build and collect the exact service images plus the committed
release tree and generated release override. Neither path builds the working
tree or executes application builds on production.

**Build environment:** The fixed, non-secret metadata required by the build
contract plus values explicitly declared in the committed `.env.build` file,
such as `NODE_VERSION`. Every `.env.build` value is public build configuration,
not a secret. The build environment excludes the ambient host environment, root
`.env`, production variables, credentials, tokens, SSH agents, and Docker
socket.

**Release artifact:** A versioned, checksummed transport containing everything
production needs to activate one immutable release. A native artifact contains
the complete runnable filesystem tree and prepare scripts. A Compose artifact
contains the committed release tree, generated release-specific override, image
inventory, and exact service images. Artifacts exclude Git metadata, environment
files, production shared state, server control-plane state, local build caches,
and build-only content not needed by the selected runtime.

**Production host:** The machine running BonesRemote and application services.
A supported production host runs Debian 12 or newer or Ubuntu 24.04 or newer.
Other Linux distributions and older releases are unsupported.

**Prepare:** The existing server-side phase that runs numbered prepare scripts
against the promoted candidate as the runtime user after shared paths are wired.
Database migrations and environment-dependent runtime preparation remain
prepare work. Dependency installation, compilation, and other application build
work do not.

**Legacy remote build:** The server-side Git export, build-user, rootless-Podman,
and build-cache pipeline. It is removed rather than retained as a selectable mode
or automatic fallback.

## Desired Outcome

`bonesdeploy deploy` always builds the configured committed revision locally in
Docker before opening the deployment transaction. BonesRemote safely imports
the resulting native tree or Compose images and uses the existing prepare,
preflight, activation, restart, rollback, pruning, and cleanup lifecycle without
consulting an application repository or building application code.

Build or packaging failure leaves production untouched. Upload, validation,
prepare, or activation failure preserves existing cleanup and rollback
semantics. No failure can execute a build on production.

Fresh server provisioning and diagnostics accept only supported Debian and
Ubuntu versions. Production hosts contain no application source repository,
application builder image, per-site build user, build cache, or rootless Podman
state. Compose hosts retain Docker Engine and the Compose plugin only to load and
run prepared images.

## Scope

- Remove native build-mode configuration, selection, and compatibility behavior.
- Replace the local rootless-Podman runner and diagnostics with Docker.
- Keep the pinned Debian builder image, Linux `x86_64` target, clean Git revision
  provenance, complete-context artifact, and strict artifact protocol.
- Restrict build inputs to fixed non-secret contract metadata and explicitly
  declared, committed `.env.build` values.
- Remove native server checkout/build execution and its build users, caches,
  rootless container resources, image-store requirements, diagnostics, and tests.
- Build Docker Compose service images locally, upload them with their committed
  release tree, and prohibit production Compose pulls and builds.
- Remove production application bare repositories, Git-push readiness, branch
  resolution, and source materialization for every runtime backend.
- Produce complete runnable framework artifacts and remove build-only dependency
  trees, caches, and scripts that the selected production runtime does not use.
- Retain one BonesRemote artifact receipt and release lifecycle for native and
  Compose releases.
- Replace the complete local runtime model in config sync with a narrow native
  or Docker remote runtime descriptor, and remove unused remote prepare inputs.
- Enforce and document Debian 12+ and Ubuntu 24.04+ production support in
  provisioning and diagnostics.
- Update focused, regression, ignored E2E, architecture, security, and user
  documentation for the breaking contract.

## Constraints

- Docker is the sole supported local build engine. Builds use Linux containers
  and the project-owned builder image pinned by digest.
- Builds target Linux `x86_64`. Non-`x86_64` build machines must provide working
  Docker emulation and pass an execution probe before the build starts.
- Docker containers are never privileged and never receive the Docker socket,
  host home directory, SSH configuration, credential stores, production state,
  or ambient host environment.
- The local root `.env`, decrypted production environment, backup settings,
  ambient host variables, and other secret variables are not build inputs.
- `.env.build` remains the explicit project-owned input for non-secret build-tool
  settings. It must not contain passwords, tokens, private keys, or credentials.
- Local Git remains the source-selection and revision-provenance boundary. The
  artifact records the exact locally exported commit; production does not
  require a second repository or pushed branch.
- Artifact input remains untrusted at the privileged boundary. BonesRemote
  retains framing, size, digest, path, entry-type, symlink, file-count, and
  extracted-size enforcement before promotion.
- BonesInfra provisioning requests and production `shared/.env` secrets remain
  separate from the narrow BonesRemote deployment configuration contract.
- Prepare remains server-side with its current runtime identity and shared-state
  access, but may not install application dependencies or compile the release.
- Compose images are built for `linux/amd64`, receive immutable release-specific
  tags, and are started remotely with pulls and builds disabled.
- Production support is limited to Debian 12+ and Ubuntu 24.04+; unsupported
  distributions or versions fail clearly rather than continuing best-effort.
- E2E tests are authored for human execution and are not run by an agent unless
  explicitly requested.

## Exclusions

- Automatic fallback to a server-side build or another local container engine.
- Compatibility guarantees, in-place migrations, or automatic repair for
  installations created by earlier releases.
- Private dependency credentials or any mechanism that exposes secrets to
  arbitrary application build scripts; `.env.build` is not a secret channel.
- Registry publishing or registry-mediated deployment; Compose images travel in
  the release artifact over the existing SSH transport.
- Production support for Arch Linux, macOS, Windows, or Linux distributions
  other than the specified Debian and Ubuntu releases.
- A broader native build-host support promise; Docker Desktop, WSL2, and native
  Windows CLI portability require their own validation and decisions.
- Artifact registries, signing, provenance services, SBOM generation, or
  reproducible-build certification.
