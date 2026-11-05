# Idea

## Request

Add automated Borg backups to BonesDeploy through `bonesremote`, using the
`borgbackup` Rust crate. Backups remain on the deployment server so the user
can arrange external replication separately.

## Problem

BonesDeploy currently deploys application releases and shared runtime data but
does not provide a repeatable backup workflow for shared runtime data. Users
must independently design backup commands, scheduling, retention, and
passphrase handling for every server.

## Definitions

**Backup repository:** A per-site Borg repository stored on the deployment
server under `/var/lib/bonesdeploy/backups/<project_name>.borg`. It is separate
from application releases, shared data, Git repositories, and BonesRemote
control-plane state.

**Backup run:** One root-owned, cron-triggered invocation of `bonesremote` that
creates a Borg archive containing the site's `shared/` directory, then applies
the configured retention policy.

**Backup schedule:** A five-field Linux crontab expression that controls when
the site's backup run starts. The default schedule is daily at midnight:
`0 0 * * *`.

## Desired outcome

For each newly initialized project:

- BonesDeploy stores a randomly generated Borg passphrase in the local,
  gitignored `.env` as `BONES_BORG_PASSPHRASE`.
- Provisioning installs Borg, creates a root-owned per-site repository, and
  writes the passphrase remotely as
  `/root/.config/bonesremote/sites/<site>/.borg_passphrase` with mode `0600`.
- Provisioning installs a root-owned `/etc/cron.d/bonesdeploy-<site>-backup`
  entry using the configured crontab expression.
- The scheduled run archives only `shared/`.
- Borg retention keeps 30 days of backups by default.
- Backup output and failures are available through journald.
- No backup trigger outside cron is provided.
- Borg repositories use `repokey-blake2` encryption and archive names contain
  the project name and a UTC timestamp.

## Scope

This change includes:

- Backup configuration for schedule and retention, with defaults of midnight
  daily and 30 days.
- Local passphrase generation and managed `.env` storage.
- Secure passphrase transport during provisioning and root-only remote storage.
- Borg repository creation and archive/prune execution in `bonesremote` using
  the `borgbackup` crate.
- Per-site cron provisioning.
- Tests and documentation for configuration, permissions, scheduling, archive
  contents, retention, and failure reporting.

## Constraints

- Backups are stored on the deployment server; external replication is the
  user's responsibility and is not automated.
- Linux cron is the only scheduler. No systemd timer, user-facing manual trigger, or
  alternate scheduler is part of this feature.
- The passphrase must not be written to deployment descriptors, control-plane
  snapshots, command arguments, logs, build environments, or generated project
  files.
- The Borg repository is root-owned and must not be
  writable by the site's runtime user.
- Journald is the error-reporting destination.
- Existing projects are not migrated or provisioned by this feature.
- Passphrase rotation is not supported in v1.

## Exclusions

This change does not include:

- External backup replication, off-site storage, cloud backends, or backup
  monitoring and notification services.
- Backups of releases, the bare Git repository, BonesRemote control-plane
  state, build caches, runtime sockets, or the Borg repository itself.
- Database logical dumps, including Redis and Valkey.
- User-facing backup commands, restore commands, archive browsing, or mounts.
- A backup-specific deployment lock policy beyond the existing site mutation
  serialization required to avoid concurrent site mutations.
- Existing-project migration and passphrase rotation.
