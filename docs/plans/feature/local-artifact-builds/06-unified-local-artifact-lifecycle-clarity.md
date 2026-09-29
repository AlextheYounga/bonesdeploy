# Unified Local Artifact Lifecycle Clarification

## Trigger

Moving native application builds to the local machine exposed two remaining
parts of the production-build model. Native deployment still requires a pushed
commit in a production bare repository solely to compare revision identifiers,
and Docker Compose deployment still exports source from that repository and
builds application images on production.

The local build container is also no longer a production tenant-isolation
boundary. Its purpose is to provide a clean, reproducible Linux `x86_64` build
environment compatible with production. The privileged boundary begins when
BonesRemote accepts deployment input on the production host.

## Decision

All application releases build locally before the deployment transaction.
Native projects produce a complete runtime filesystem artifact. Docker Compose
projects run Compose validation, pulls, and builds locally and produce an
artifact containing the committed release tree, a generated BonesDeploy Compose
override, an image inventory, and the exact service images required to run the
release.

Local Git is the sole source-selection boundary. BonesDeploy resolves and
exports the configured branch commit and records that commit as release audit
metadata. Production does not maintain application bare repositories, require a
first Git push, resolve remote branch refs, or compare the artifact revision to
a second repository controlled by the same deploy credential.

Compose builds use the same fixed `linux/amd64` target as native builds. They
use `.env.build` for explicit public build-time interpolation and do not receive
the production `.env` or ambient host variables from BonesDeploy. The project
Compose definition may not depend on production secrets to select services,
build contexts, images, or other build topology. Private dependency and registry
credentials remain unsupported.

Every Compose service image is tagged with an immutable release-specific name
derived from the site, service, and revision. BonesDeploy writes a generated
override that selects those tags. BonesRemote validates and loads the image
archive, materializes the release tree and override, and starts the stack with
builds and pulls disabled. Rollback therefore selects the previous release's
Compose override and already-loaded image tags rather than a mutable shared tag.
Failed-release cleanup and release pruning remove release-specific image tags;
Docker may retain layers still referenced by another release.

Framework build scripts must leave a complete runnable artifact. Dependency
installation, native-extension compilation, and asset compilation are local
build work. Remote prepare is limited to operations that require production
state, such as shared-path wiring, database migrations, production-secret-based
configuration, and runtime validation. Build-only content such as root
`node_modules`, framework caches, and numbered build scripts is removed when the
selected runtime does not require it.

The local Docker runner is a compatibility and reproducibility mechanism, not a
hardened sandbox. It retains the pinned target and image, explicit environment,
secret exclusion, narrow project/cache inputs, timeout, output, and reliable
cleanup. It does not preserve production-oriented user, namespace, or
container-lifecycle complexity that provides no practical local-build value.
BonesDeploy never adds privileged mode, the Docker socket, host home, SSH state,
credential stores, production state, or ambient host variables to a build.

BonesRemote continues to treat every artifact as untrusted privileged input. It
retains framing, digest, resource bounds, path, entry-type, symlink, conflict,
and expanded-size validation before promotion. Manifest fields describe facts
that BonesRemote can enforce; a client-supplied builder-image value is not build
attestation and is not a production trust proof.

BonesInfra remains responsible for production machine policy and runtime
provisioning: operating-system validation, server hardening, deploy and runtime
identities, BonesRemote, constrained sudo, release/shared/configuration paths,
language runtimes, Nginx, systemd, AppArmor, Docker Engine and Compose runtime,
backups, SSL, tunnels, manifests, deletion, and infrastructure patches. It no
longer provisions application source repositories or any application build
facility. The deploy identity remains as the narrow SSH transport principal even
though it no longer serves Git repositories.

## Supersedes

This clarification supersedes mandatory server-side pushed-revision comparison,
production bare repositories, first-push readiness, Compose source deployment,
production Compose builds and pulls, the exclusion of Compose artifacts,
deferred framework artifact pruning, and any definition of prepare that permits
application dependency installation.

It does not remove friendly public commands, production runtime provisioning,
placeholder behavior, strict artifact receipt, release locking, shared-state
wiring, production preparation, sealing, preflight, activation, restart,
rollback, cancellation, pruning, recovery, or operational inspection.

## Required Authoritative Updates

- `01-idea.md` defines one local-build/artifact lifecycle for native and Compose
  runtimes, local Git provenance, complete artifacts, and the stricter prepare
  boundary.
- `02-plan.md` describes native filesystem artifacts, Compose image artifacts,
  immutable image tagging, repository removal, local-runner simplification,
  BonesInfra's runtime-only boundary, and lifecycle state changes.
- `03-tasks.md` adds the implementation, validation, and completion work needed
  to apply this clarification.
