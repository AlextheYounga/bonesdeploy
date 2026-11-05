# Idea

## Request

Address the Compose E2E error on a dedicated Git Flow bugfix branch and record
the error and repair path in durable planning documentation.

## Problem

The ignored Compose E2E scenario fails during `bonesdeploy site setup --yes`.
Provisioning correctly seeds the first-release placeholder, but remote site
doctor treats that placeholder as an active Compose release and rejects it
because it has no Compose file. Initial Compose setup therefore cannot complete
before the first deployment.

## Definitions

**Placeholder release:** The canonical `19700101_000000` release created during
site provisioning and targeted by `current` before an application deployment.
It provides a safe default site but is not an application release and does not
contain a Compose configuration.

**Active Compose release:** A non-placeholder release targeted by `current`
whose Compose configuration must be discovered and validated by remote doctor.

## Desired outcome

Remote doctor reports a first Compose deployment as pending while `current`
targets the placeholder release. `bonesdeploy site setup --yes` succeeds before
the first Compose deployment, and a non-placeholder active release still must
contain valid Compose configuration.

## Scope

- Make Compose doctor recognize the canonical placeholder release.
- Add a focused regression test for the placeholder Compose-doctor path.
- Validate the ignored Compose E2E scenario and preserve strict validation for
  deployed Compose releases.

## Constraints

- Work in a Git Flow bugfix branch with a linked worktree.
- Reuse the existing canonical placeholder recognition rather than duplicating
  release-name interpretation.
- Do not weaken active Compose configuration validation.
- Run E2E scenarios serially; nested Docker requires a real Incus guest.

## Exclusions

- Changing placeholder provisioning or creating a Compose file in the
  placeholder release is excluded.
- Docker installation, Compose service provisioning, and post-deployment
  Compose health classification are excluded.
- Other framework scenarios are excluded until this Compose setup blocker is
  resolved.
