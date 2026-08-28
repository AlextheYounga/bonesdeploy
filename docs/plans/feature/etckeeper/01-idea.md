# Idea

## Request

Add and configure etckeeper for BonesDeploy hosts. Server setup must install
etckeeper and initialize `/etc` using its package defaults. Every successful
mutating BonesInfra flow must finish by checking and committing its `/etc`
changes with etckeeper.

## Problem

BonesInfra changes host configuration across server setup, site setup, runtime,
service, SSL, and helper provisioning, but those changes are not consistently
recorded. Administrators cannot reliably inspect which provisioning flow changed
`/etc` or recover the prior configuration from local host history.

## Definitions

**Mutating BonesInfra flow:** A remote BonesInfra apply command that queues
provisioning operations: `server apply`, `site apply`, `runtime apply`,
`services apply`, `ssl apply`, or `helpers apply`. It excludes read-only
`manifest show` and local or remote patch bookkeeping.

**Etckeeper final step:** The last operation queued by a mutating BonesInfra
flow. It checks whether `/etc` has changes and invokes etckeeper's normal commit
path when changes exist. It is not runner cleanup and is not executed after a
failed preceding operation.

**Etckeeper defaults:** The Debian package's standard Git configuration and
ignore behavior. BonesInfra does not add project-specific ignore rules or
replace the package's configuration policy.

## Desired outcome

`bonesdeploy server setup` installs etckeeper and leaves `/etc` initialized as
the package's default Git-backed etckeeper repository. Every successful
mutating BonesInfra flow records its resulting `/etc` state in a final etckeeper
commit, while a flow with no `/etc` changes completes without a spurious commit
failure. Read-only and patch flows do not create etckeeper commits. Server
doctor reports a missing etckeeper installation as a baseline issue.

## Scope

- Install etckeeper as part of the host-wide server package baseline.
- Ensure `/etc` is initialized with etckeeper defaults during server setup.
- Add one reusable final etckeeper operation to each mutating BonesInfra flow.
- Add server-doctor evidence that etckeeper is installed.
- Add focused Python and Rust tests and update relevant operator documentation.

## Constraints

- Etckeeper commits are queued as the final PyInfra operation, so preceding
  provisioning failures prevent the commit operation from running.
- Existing PyInfra runner and operation boundaries remain in use; no separate
  post-run cleanup hook is introduced.
- Only Debian and Ubuntu hosts are supported.
- `/etc` remains root-protected; etckeeper commands run with the same privileged
  provisioning mechanism used by existing host configuration operations.
- No new ignore rules or custom etckeeper configuration is introduced.
- E2E tests must not be run during implementation.

## Exclusions

- No remote backup repository, push policy, or workstation-side etckeeper setup.
- No BonesDeploy CLI command for browsing, reverting, or managing `/etc` history.
- No changes to the existing deployment lifecycle owned by `bonesremote`.
- No etckeeper commit for read-only manifest inspection or patch bookkeeping.
