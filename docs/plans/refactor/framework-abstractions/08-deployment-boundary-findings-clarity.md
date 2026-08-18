# Clarification

## Trigger

The deployment audit found strong existing primitives but important bypasses and
duplicated lifecycle helpers that must be preserved in the Deployment child plan.

## Decision

The Deployment child change will strengthen `SiteMutation`, `SiteState`,
`DeploymentSnapshot`, `DeploymentPhase`, and the existing lifecycle modules into a
clear public deployment boundary.

Concrete findings are:

- `release/commands/deploy/lifecycle.rs::run_staged_deployment()` coordinates phase
  advancement and several abort, activation-failure, cleanup, and rollback paths
  inline;
- `release/kill.rs` manually acquires a lock and adopts `Bones::for_site(site)`,
  bypassing normal validated site configuration;
- `release/recover.rs` accesses the lock and state store directly;
- `commands/doctor/site.rs` uses convention-derived `Bones::for_site()` rather
  than validated site configuration;
- `release::state` exposes crate-wide read/write functions and callers directly
  manipulate active and staged state;
- process-start parsing, systemd requirement parsing, account/group parsing,
  numbered-script listing, prepare-input streaming, and nginx path derivation are
  duplicated across lifecycle and command modules;
- status, service, release listing, and doctor independently derive release,
  service, account, and path information.

The child plan must preserve atomic state writes, confused-deputy validation,
mutation locking, preflight gating, activation semantics, revision consistency,
rollback behavior, and cleanup outcomes while closing these side doors.

## Supersedes

This adds the concrete deployment findings that were compressed out of the parent
summary and supersedes the generic phrase "strengthen deployment boundaries."

## Effect on the record

- `02-plan.md` identifies the exact deployment bypasses and duplicated helpers.
- `03-tasks.md` requires the Deployment child plan to cover mutation, state,
  lifecycle, and inspection relationships with these invariants intact.
