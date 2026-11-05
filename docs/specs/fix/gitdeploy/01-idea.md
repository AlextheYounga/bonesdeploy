# Git Deployment SSH Entry Point

## Request

Run `bonesdeploy deploy` through the `git` SSH identity instead of opening a
root SSH session. Permit `git` to invoke only the exact BonesRemote commands
required for deployment through passwordless, non-interactive sudo.

## Problem

The deployment command currently connects to the server as the configured
privileged SSH user, normally root. This gives routine deployments a root SSH
entry point even though the server already has a dedicated `git` deploy
identity and BonesRemote already constrains deployment behavior.

## Definitions

**Deployment SSH identity:** The Unix account used by `bonesdeploy deploy` to
open the remote SSH session. For this change it is `git`.

**Privileged deployment command:** One complete BonesRemote command executed as
root through `sudo -n`: configuration sync or the existing full deployment
lifecycle.

**Deployment lifecycle:** The existing `bonesremote deploy` implementation,
including its current state, lock, release ownership, activation, rollback, and
cleanup behavior. This change does not split or replace that lifecycle.

## Desired outcome

`bonesdeploy deploy` connects as `git`, runs the exact configuration-sync and
deployment commands through `sudo -n`, and never opens a root SSH session.
BonesRemote continues to execute the existing deployment lifecycle as root,
while build scripts continue to run as `<site>-build` and prepare scripts
continue to run as `<site>`.

## Scope

This change covers the deploy SSH entry point, the two direct sudo command
invocations, the sudoers policy, denial tests, and documentation of the
resulting security boundary.

## Constraints

- Config sync always reads its descriptor from stdin and writes the
  site-derived `/srv/conf/<site>/bones.json` snapshot; deploy always loads that
  snapshot.
- Sudoers permits only direct config sync and deploy commands with one validated
  site argument. It denies arbitrary BonesRemote subcommands, optional extra
  arguments, reordered arguments, and trailing arguments.
- The deployment host requires sudo 1.9.10 or newer for anchored argument
  regular expressions.
- Deployment state, locks, backup credentials, control-plane snapshots, release
  ownership, and lifecycle sequencing remain unchanged.
- Build scripts run only as `<site>-build`.
- Prepare scripts run only as `<site>`.
- Repository-provided scripts never execute as root.
- The shared `git` identity retains its documented single-operator limitation.
- E2E coverage may be added but is not run locally.

## Exclusions

- Splitting the deployment lifecycle into privileged transition subcommands.
- Moving deployment state, locks, snapshots, or backup credentials.
- Changing root password-login policy or other server-hardening settings.
- Changing rollback, cancellation, recovery, backup, or release-management SSH
  entry points.
