# Idea

## Request

Provide `bonesdeploy site delete` as the inverse of site setup: permanently
remove one site's remote resources without introducing a separate deleted-site
lifecycle.

## Problem

The current command persists a deletion plan, decommissioning state, and a
verified tombstone. Those states block mutations, remain registered after
successful deletion, and require setup reactivation. This machinery has caused
deletion retries and fresh setup to fail even after the site's resources were
removed.

## Definitions

**Site deletion:** An irreversible, idempotent teardown of resources owned
exclusively by the configured site, followed by removal of its BonesRemote
registration.

**Site registration:** The root-owned BonesRemote directory at
`/root/.config/bonesremote/sites/<site>`. Successful deletion removes it.

**Deletion inventory:** The current safety-validated, secret-free BonesInfra
manifest used to display and execute teardown. It is not persisted remotely.

## Desired Outcome

`bonesdeploy site delete` displays the current site inventory, requires exact
confirmation, removes the site's remote resources, removes its BonesRemote
registration, and reports completion. Rerunning after interruption safely
repeats missing-resource removals and finishes. A later `site setup` follows the
ordinary fresh-setup path.

## Scope

The change removes persisted deletion plans, decommissioning markers,
tombstones, deletion-specific mutation blocking, and setup reactivation. It
retains manifest validation, exact-name confirmation, ordered teardown,
idempotency, and the existing per-site lock while BonesRemote removes its own
registration.

## Constraints

Deletion must preserve local project files and shared server resources. Remote
registration removal must validate the site name, remain within the canonical
BonesRemote sites root, refuse unsafe path kinds, preserve the sites root and
sibling lock, and treat an absent registration as success. BonesRemote
registration is removed only after BonesInfra teardown succeeds.

## Exclusions

The change does not add archival, soft deletion, selective resource retention,
concurrent delete/deploy coordination, or E2E execution during local validation.
