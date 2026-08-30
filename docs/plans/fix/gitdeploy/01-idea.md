# Git-Owned Deployments

## Request

Run deployments as the `git` deploy identity instead of root. Root must only
mediate narrowly defined, validated privileged transitions.

## Problem

The local deploy command currently opens a privileged SSH session, passes a
configuration directly to `bonesremote deploy`, and the remote deployment
command rejects non-root callers. Lifecycle state is also stored below root's
configuration directory. This violates the project's separation between the
deploy identity, runtime identity, and provisioning identity.

## Definitions

**Deploy coordinator:** The `bonesremote deploy` process running as `git`. It
loads the synchronized site snapshot, exports source, and coordinates lifecycle
operations.

**Privileged transition:** A small `bonesremote` operation authorized through
exact sudoers rules. It derives paths and identities from a validated site and
release name and never executes repository code as root.

**Control-plane snapshot:** The sanitized `RemoteDeploymentConfig` persisted by
the root-only config-sync command and read by the deploy coordinator.

## Desired outcome

`bonesdeploy deploy` connects as `git`, synchronizes the snapshot through an
exact root-only command, and runs `bonesremote deploy --site <site>` without
sudo. The coordinator and deployment state are git-owned; release namespaces
remain root-controlled; repository scripts run only as their designated build
or runtime users.

## Scope

This change covers deployment entrypoints, remote state and locking, privileged
release transitions, sudoers, provisioning, tests, and related documentation.

## Constraints

- Validate every site and release identifier at trust boundaries.
- Keep one lock across deploy, rollback, cancellation, recovery, and backup.
- Root must never execute repository-provided scripts.
- Preserve the shared `git` identity's documented single-operator limitation.
- Do not run E2E tests locally.

## Exclusions

Changing root password-login policy is a separate server-hardening change.
