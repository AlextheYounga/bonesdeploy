# Idea

## Request

Add the ability to delete a site and all of its resources after it has been set
up. Use the existing manifest as the canonical inventory so setup inspection and
site deletion do not maintain separate definitions of site ownership.

## Problem

`bonesdeploy site setup` provisions a complete remote site, but BonesDeploy has
no inverse operation. Removing a site currently requires an operator to discover
and delete resources manually across deployment state, application data, Git,
systemd, nginx, AppArmor, users, databases, caches, certificates, tunnels, and
backups. Manual removal can leave data behind or accidentally damage shared
server infrastructure.

The current manifest provides the beginning of an ownership inventory, but it is
read-only and models only filesystem artifacts and systemd services. It does not
declare every resource that must be removed, and its project-defined artifact
paths are not validated strongly enough for use by a destructive command.

## Definitions

**Site deletion:** An irreversible remote operation that removes every resource
owned exclusively by the configured site. Site deletion does not delete the
local project or shared server infrastructure.

**Site-owned resource:** A remote resource created or managed exclusively for
one site. This includes application and shared data, releases, the bare Git
repository, deployment state, site users and groups, site-specific service data
and identities, systemd units, nginx configuration, AppArmor profiles, runtime
containers and images, logs, certificates, Quick Tunnel state, and backups.

**Shared server resource:** A package, daemon, account, configuration, image,
certificate authority account, filesystem root, or other host resource used by
the server baseline or by more than one site. Shared server resources remain
installed and unchanged except for removing the deleted site's membership or
configuration.

**Deletion manifest:** The typed BonesInfra manifest declarations that identify
site-owned resources and the resource kind needed to inspect and remove each
one. It is the same canonical inventory used by `bonesdeploy site manifest`; it
does not contain secrets or executable deletion commands supplied by manifest
entries.

**Deletion plan:** The safety-validated, secret-free resolved deletion manifest
persisted when deletion first begins. Every retry uses this snapshot rather than
recomputing ownership from potentially changed local configuration.

**Decommissioning:** A persisted remote state entered under the site's existing
deployment lock before teardown starts. While a site is decommissioning,
deployments and other site mutations are rejected. The state remains after a
partial failure so deletion can be run again safely.

**Deletion tombstone:** Minimal root-owned control-plane metadata recording that
a site's mutable state was deleted. It is not application data or a runnable site
resource. An unverified tombstone records that state cleanup occurred but final
inspection did not finish; a verified tombstone records successful final
inspection. Both prevent delayed or manually invoked deployment commands from
recreating state. A later successful setup of the same site name removes only a
verified tombstone after provisioning is complete. The tombstone retains the
deletion plan so a repeated delete can verify the original inventory.

## Desired outcome

Running `bonesdeploy site delete` from an initialized project displays the
configured site and its deletion inventory, then requires the operator to type
the project name. `--yes` skips the interactive confirmation for automation.

After confirmation, BonesDeploy prevents further deployments, removes every
declared site-owned remote resource, verifies that the resources are absent,
and reports completion. The command is safe to run again after an interrupted or
partially failed deletion. A failure leaves the site decommissioning rather than
allowing deployments to resume against partially removed infrastructure.

Successful deletion removes the site's mutable deployment state and retains only
its deletion tombstone. Re-running setup for the same configured site keeps
deployments blocked throughout provisioning and reactivates the site after setup
has completed successfully.

The local project, local Git repository and remotes, `.env`, `infra/`, encrypted
secrets, local GPG keys, and project-scoped BonesInfra cache remain available so
the operator retains the source and configuration that identified the deleted
site.

## Scope

This change includes:

- A public `bonesdeploy site delete` command with typed project-name
  confirmation and `--yes` automation support.
- Extension of the BonesInfra manifest to declare and inspect every site-owned
  resource needed for complete teardown, including resources that are not
  filesystem paths or systemd services.
- Destructive manifest validation that permits only known resource kinds,
  project-derived identities, and approved site-owned path boundaries.
- A persisted deletion plan and a BonesRemote coordinator session that holds the
  existing deployment lock throughout remote teardown.
- Deployment serialization and persisted decommissioning state in BonesRemote.
- A minimal deletion tombstone and safe reactivation when the same site is set
  up again.
- Ordered, idempotent teardown of ingress, processes, runtime resources,
  service data and identities, certificates, backups, users, files, Git, and
  deployment state.
- Two-phase post-deletion inspection that proves provisioning-owned resources
  are absent before BonesRemote state cleanup and proves every declared resource
  is absent before coordinator completion.
- Focused tests for inventory completeness, deletion safety, ordering,
  idempotency, partial failure, and CLI confirmation.

## Constraints

The command is destructive and must default to no. `--yes` may bypass the
interactive prompt, but it must not bypass configuration validation, manifest
validation, deployment locking, or post-deletion verification.

Manifest declarations are data, not arbitrary teardown callbacks. Deletion code
must select a trusted handler for each supported resource kind and reject
unknown kinds, unsafe paths, malformed identities, shared resources, and
declarations that escape the configured site's ownership boundaries before any
remote mutation begins.

Filesystem deletion must remain inside those boundaries at execution time, not
only during preflight. Handlers must use symlink-resistant operations anchored to
trusted directory descriptors, must not cross mount points, and must remove
links themselves rather than following their targets.

Deletion must preserve shared packages, shared daemons and database servers,
the `git` deployment account, global sudoers and SSH configuration, shared image
stores, default-deny nginx assets, Certbot account state, global backup roots,
other sites, and their data.

The operation must be rerunnable. Missing resources are successful no-ops, and
the order must stop public traffic and processes before deleting their data or
identities.

The validated deletion plan must be persisted before the first destructive
operation. Retries must use the persisted plan even when local framework,
service, domain, SSL, backup, or custom manifest configuration has changed.

Non-trivial behavior must have runnable tests. Normal local validation excludes
execution of the E2E suite.

## Exclusions

This change does not delete or edit local project files, local Git configuration,
encrypted secrets, local GPG keys, or local BonesInfra caches.

It does not delete external DNS records, third-party provider resources not
created by BonesDeploy, shared server software, package repositories, global
daemon configuration, Certbot registration accounts, shared base images, or
etckeeper history. Historical etckeeper commits may continue to record files
that existed before deletion.

It does not add site archival, soft deletion, backup retention, resource
selection flags, or restoration. Site deletion always means complete removal of
all declared site-owned remote resources. The deletion tombstone is deliberately
retained control-plane safety metadata, not a retained site resource or backup.
