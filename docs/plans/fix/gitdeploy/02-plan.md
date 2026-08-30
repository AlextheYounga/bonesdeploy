# Git-Owned Deployments Plan

## Current behavior

`crates/bonesdeploy/src/commands/deploy.rs` connects with
`ssh::connect_privileged`, sends a descriptor to `bonesremote deploy`, and the
remote command requires root. `config sync` is root-only and stores snapshots
under `/srv/conf/<site>`. Lifecycle modules independently require root, while
state and the deployment lock are currently below `/root/.config/bonesremote`.
Provisioning creates the remote site state directory as root-only and makes the
release namespace root-owned.

## Intended behavior

The local command connects with the configured deploy identity (`git`), invokes
`sudo -n bonesremote config sync --site <validated-site> --config-stdin` with
the sanitized descriptor, then invokes `bonesremote deploy --site <site>` with
no descriptor and no sudo. The remote command loads the synchronized snapshot.

Coordination and state writes run as `git`. The lock is a pre-provisioned file
in a root-owned directory writable by group `git`, so `git` can lock it but
cannot replace the lock inode. Root-only helpers perform candidate creation,
ownership handoff, sealing, activation, service restart/restoration, and
cleanup through exact sudoers entries.

## Approach

First change the CLI command contract and snapshot loading. Then separate the
state and lock paths from root secrets, retaining migration for legacy state.
Move root checks from orchestration into typed privileged lifecycle operations;
each helper derives all paths from validated identifiers and revalidates the
release before mutation. Make activation and failed-restart restoration one
privileged transaction. Finally update sudoers, provisioning, tests, and docs.

## Responsibilities and boundaries

- `bonesdeploy` owns local config serialization and the SSH session.
- `bonesremote config sync` owns root-only snapshot installation.
- `bonesremote deploy` owns unprivileged coordination and source export.
- `release::state` owns git-readable state and the shared lock.
- Lifecycle privileged helpers own root-controlled filesystem and service changes.
- BonesInfra owns ownership, mode, migration, and sudoers provisioning.
- Tests pin command contracts, path derivation, identity boundaries, and rollback.

## Affected areas

`crates/bonesdeploy`, `crates/bonesremote`, `crates/bonesdeploy-core`,
`crates/bonesinfra/python`, embedded-wheel generation, tests, `CONTEXT.md`,
`README.md`, and security/architecture documentation.

## Decisions

- The deploy command reads the persisted snapshot rather than accepting a
  second descriptor, preventing the coordinator from bypassing root-mediated
  config installation.
- Release directories remain root-controlled; git receives only an exclusive
  candidate created by a privileged operation.
- Sudoers allowlists exact typed subcommands and never allow `deploy`.

## Risks

Legacy root-owned state may prevent a deploy until migration is applied. A
mistake in candidate ownership or sudoers argument matching could enable release
replacement or arbitrary root execution. Activation must restore the previous
release if restart verification fails.

## Validation

Run focused Rust and Python tests for state migration, path and identity
validation, command allowlists, candidate sealing, activation restoration, and
CLI command construction. Run workspace tests excluding E2E, clippy, fmt,
ruff, pytest, wheel regeneration, and shfmt. Do not run E2E locally.
