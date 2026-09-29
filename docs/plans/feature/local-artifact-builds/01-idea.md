# Idea

## Request

Make local artifact builds the only native BonesDeploy build path in the next
breaking release. The local BonesDeploy CLI builds the committed application in
Docker and uploads the completed release artifact. BonesRemote does not retain a
server-side native build fallback.

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

**Local artifact build:** The sole native build path. BonesDeploy exports the
exact configured branch commit, runs its numbered build scripts in a local
Docker Linux container, packages the resulting tree, and uploads it. It does not
build the working tree or execute application build scripts on production.

**Build environment:** The fixed, non-secret metadata required by the build
contract, such as workspace and cache paths. It excludes the ambient host
environment, root `.env`, `.env.build`, production variables, credentials,
tokens, SSH agents, and Docker socket.

**Release artifact:** A versioned, checksummed transport containing the complete
post-build source tree. It includes committed deployment scripts needed by
server-side prepare and excludes Git metadata, environment files, production
shared state, server control-plane state, and the local build cache.

**Production host:** The machine running BonesRemote and application services.
A supported production host runs Debian 12 or newer or Ubuntu 24.04 or newer.
Other Linux distributions and older releases are unsupported.

**Prepare:** The existing server-side phase that runs numbered prepare scripts
against the promoted candidate as the runtime user after shared paths are wired.
Database migrations and environment-dependent runtime preparation remain
prepare work.

**Legacy remote build:** The server-side Git export, build-user, rootless-Podman,
and build-cache pipeline. It is removed rather than retained as a selectable mode
or automatic fallback.

## Desired Outcome

`bonesdeploy deploy` always builds the configured committed revision locally in
Docker before opening the deployment transaction. BonesRemote verifies the
artifact revision against the configured remote branch, safely imports it, and
uses the existing prepare, preflight, activation, restart, rollback, pruning,
and cleanup lifecycle.

Build or packaging failure leaves production untouched. Upload, validation,
prepare, or activation failure preserves existing cleanup and rollback
semantics. No failure can execute a build on production.

Fresh server provisioning and diagnostics accept only supported Debian and
Ubuntu versions. Production hosts contain no native application builder image,
per-site build user, build cache, or rootless Podman state.

## Scope

- Remove native build-mode configuration, selection, and compatibility behavior.
- Replace the local rootless-Podman runner and diagnostics with Docker.
- Keep the pinned Debian builder image, Linux `x86_64` target, clean Git revision
  provenance, complete-context artifact, and strict artifact protocol.
- Restrict build inputs to fixed non-secret contract metadata; remove
  `.env.build` injection from application builds.
- Remove native server checkout/build execution and its build users, caches,
  rootless container resources, image-store requirements, diagnostics, and tests.
- Retain one BonesRemote artifact receipt and release lifecycle.
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
- The local root `.env`, `.env.build`, decrypted production environment, backup
  settings, and other user-defined secret variables are not build inputs.
- Git remains the revision-provenance boundary. The artifact revision must equal
  the configured branch commit in the server bare repository.
- Artifact input remains untrusted at the privileged boundary. BonesRemote
  retains framing, size, digest, path, entry-type, symlink, file-count, and
  extracted-size enforcement before promotion.
- Prepare remains server-side with its current runtime identity and shared-state
  access.
- Production support is limited to Debian 12+ and Ubuntu 24.04+; unsupported
  distributions or versions fail clearly rather than continuing best-effort.
- E2E tests are authored for human execution and are not run by an agent unless
  explicitly requested.

## Exclusions

- Automatic fallback to a server-side build or another local container engine.
- Compatibility guarantees, in-place migrations, or automatic repair for
  installations created by earlier releases.
- Private dependency credentials or any mechanism that exposes secrets to
  arbitrary application build scripts.
- Docker Compose image builds, registry publishing, Docker image transport, or
  expansion of the existing native artifact format to Compose applications.
- Production support for Arch Linux, macOS, Windows, or Linux distributions
  other than the specified Debian and Ubuntu releases.
- A broader native build-host support promise; Docker Desktop, WSL2, and native
  Windows CLI portability require their own validation and decisions.
- Framework-specific artifact pruning, artifact registries, signing, provenance
  services, SBOM generation, or reproducible-build certification.
