# Local-Only Docker And Production Support Clarification

## Trigger

After the opt-in local artifact implementation was completed, the project chose
to make the next release a deliberate breaking change rather than preserve two
native build systems for a small installed base. The local build machine is an
acceptable place to depend on Docker when build containers receive no host or
production secrets. The project also narrowed its supported production platform
to the server environments it intends to validate and operate.

## Decision

Local artifact builds are the only native build path. Build mode is no longer a
configuration concept, and BonesRemote does not retain an explicit or automatic
server-side native build fallback.

Docker is the sole local build engine. Podman support, rootless enforcement, and
a generic engine-selection abstraction are removed. Docker runs the pinned
Linux `x86_64` builder without privileged mode, host credential mounts, the
Docker socket, ambient host variables, production variables, root `.env`, or
`.env.build`. Fixed non-secret workspace metadata remains part of the build
contract.

Supported production hosts are Debian 12 or newer and Ubuntu 24.04 or newer.
Provisioning and diagnostics reject older releases and other distributions
explicitly. This production support policy does not itself promise native
Windows or broader local build-host support.

The release does not guarantee in-place upgrades from earlier BonesDeploy
installations. Migration and repair behavior can be designed later from concrete
user needs instead of preserving the obsolete server build architecture now.

## Supersedes

This decision supersedes the original plan's opt-in `local|remote` build mode,
remote compatibility default, rootless Podman local runner, mixed-mode server
provisioning, server builder image requirement, and deferred default switch.

It does not weaken artifact receipt validation, Git revision provenance,
server-side prepare behavior, activation rollback, or the rule that failed local
builds never fall back to production execution.

## Required Authoritative Updates

- `01-idea.md` now defines local Docker artifacts as the only native build path,
  the no-secret build environment, the breaking compatibility boundary, and the
  supported production systems.
- `02-plan.md` now describes removal of build modes, remote native builds,
  Podman, server build resources, and user-defined build variables, plus Docker
  execution and production-version enforcement.
- `03-tasks.md` retains the completed artifact-foundation history and adds the
  pending breaking-release implementation, validation, and completion work.
