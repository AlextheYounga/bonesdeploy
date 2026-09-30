# Idea

## Request

Rename the historical `git` deployment user to `bonesdeploy` now that local
artifact builds and SSH artifact transport have replaced Git-push deployment.
Treat the rename as a breaking host contract and do not migrate accounts from
past releases.

## Problem

The production account named `git` no longer hosts repositories, accepts Git
pushes, exports source, or builds applications. It is the narrow SSH transport
identity for routine deployment, but its old name describes removed behavior
and obscures its current security role in code, diagnostics, provisioning, and
documentation.

## Definitions

**Deploy identity:** The global production Unix account named `bonesdeploy`
that accepts authorized-key SSH connections for routine artifact deployment.
It may invoke only the exact root-owned BonesRemote config-sync and deploy
commands granted by the BonesDeploy sudoers policy. It is not an application
runtime identity, build identity, or administrative SSH identity.

**Administrative SSH identity:** The configured privileged account used by
server and site administration. It remains separate from the deploy identity
and is not renamed by this change.

**Runtime identity:** The dedicated per-site Unix account that runs an
application and owns its mutable shared state. Runtime identities remain
isolated from the global deploy identity.

## Desired outcome

Fresh BonesDeploy server provisioning creates `bonesdeploy` as the deploy
identity, installs its authorized keys, grants its existing narrow sudo policy,
and does not create a `git` account. Routine deployment, remote version checks,
and SSH connectivity checks connect as `bonesdeploy`. BonesRemote diagnostics
and security checks recognize `bonesdeploy` as the deploy identity and continue
to reject membership in site runtime groups.

Code, tests, generated assets, architecture descriptions, security guidance,
and user documentation consistently call this account `bonesdeploy` or the
deploy identity. References to Git remain only where they describe local source
selection or actual Git operations.

## Scope

- Change the canonical Rust and Python deploy-account values from `git` to
  `bonesdeploy`.
- Provision `/home/bonesdeploy`, copy the configured administrative authorized
  keys into it, and render the existing sudoers allowlist for `bonesdeploy`.
- Make routine BonesDeploy SSH transport use `bonesdeploy`.
- Update BonesRemote identity collection, site isolation checks, doctor output,
  tests, and fixtures to use the renamed account.
- Update embedded guidance and repository documentation that describe the
  production deploy identity or its home directory.
- Remove stale deploy-account terminology and test data exposed by the rename.

## Constraints

- This is a breaking production-host contract. Existing `git` accounts,
  authorized keys, home contents, UIDs, GIDs, and sudoers entries are not
  migrated or retained for compatibility.
- Existing hosts must be reprovisioned to satisfy the new contract.
- The deploy identity's privilege boundary remains unchanged: it receives no
  runtime-group or Docker-group membership and may sudo only exact BonesRemote
  config-sync and deploy command forms.
- The account continues to use authorized-key SSH and a shell capable of
  executing the existing remote commands.
- Per-site runtime identities and the configured administrative SSH identity
  remain unchanged.
- E2E scenarios may be updated but are not run by an agent unless explicitly
  requested.

## Exclusions

- In-place account rename, UID/GID preservation, home-directory movement,
  authorized-key migration, legacy sudoers cleanup, or update patches for hosts
  provisioned by past releases.
- Automatic fallback from `bonesdeploy` to `git` during SSH connection.
- Changes to the two-command sudoers authority or the root-owned BonesRemote
  deployment lifecycle.
- Per-site deploy identities, multi-operator tenant isolation, forced-command
  SSH, or broader SSH-account hardening.
- Renaming local Git concepts, commands, repositories, branches, or revision
  provenance that still represent actual Git behavior.
