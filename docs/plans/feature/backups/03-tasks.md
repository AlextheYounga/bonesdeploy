# Tasks

## Implementation

- [x] Add the typed backup configuration, defaults, validation, managed `.env`
  keys, and transport fields while keeping the passphrase out of deployment
  and control-plane descriptors.
- [x] Add canonical Rust and Python backup paths for the per-site `.borg`
  repository and remote passphrase file.
- [x] Add BonesInfra provisioning for Borg, root-owned backup directories,
  secure passphrase installation, and `/etc/cron.d` rendering with the default
  midnight schedule and configured retention passed to `bonesremote`.
- [x] Add the `borgbackup` dependency and implement the internal root-only
  `bonesremote backup run --site <site>` operation using `SiteMutation`.
- [x] Implement Borg archive creation for `shared/`, UTC project-based archive
  naming, and age-based pruning.
- [x] Add cron/journald command wiring without exposing the Borg passphrase in
  arguments or logs.
- [x] Update command, architecture, security, and user documentation with the
  backup layout, secret handling, cron configuration, retention, and limits.

## Validation

- [x] Add core integration tests for configuration defaults, cron validation,
  retention validation, transport serialization, and passphrase exclusion.
- [x] Add BonesInfra tests for package/layout provisioning, file modes, and
  cron output.
- [x] Add BonesRemote tests for archive path and naming, Borg command inputs,
  retention, and failure behavior.
- [x] Run `cargo test --workspace --exclude e2e` and address all failures.
- [x] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, `ruff check .`,
  `ruff format .`, and `uv run pytest` in the applicable Python directory;
  do not run e2e tests.

## Completion

- [x] Review the final diff for secret leakage, path drift between Rust and
  Python, privileged command injection, and files exceeding project limits.
- [x] Confirm all implementation and validation tasks are complete before
  recording completion notes.

## Completion notes

Implemented in `crates/bonesdeploy-core/src/config/backup.rs` (typed `Backup`
section, five-field crontab validation restricted to safe characters, positive
retention, printable-ASCII passphrase rules, and `ensure_backup_passphrase`
which preserves an existing managed passphrase before generating one), wired
through `model.rs`, `local_env.rs` (`BONES_BACKUP_SCHEDULE`,
`BONES_BACKUP_RETENTION_DAYS`, `BONES_BORG_PASSPHRASE`), and
`transport.rs::BackupFields`. `RemoteDeploymentConfig` and the control-plane
snapshot structurally cannot carry the passphrase; core tests assert the JSON
exclusion. `bonesdeploy init` calls `ensure_backup_passphrase` before writing
the managed block. Canonical paths live in `bonesdeploy-core::paths`
(`BACKUPS_ROOT`, `site_backup_repository_path`, `bonesremote_site_passphrase_path`)
and the mirrored `DeploymentPaths` fields (`backup_repository`,
`backup_passphrase_file`, `backup_cron_file`).

BonesInfra gained `services/linux/backup.py` plus `assets/cron/backup.cron.j2`.
Provisioning installs Borg, creates `/var/lib/bonesdeploy/backups` (0700) and
the site state directory (0700), writes the passphrase via `files.put` from an
in-memory source (0600, no Jinja interpolation), creates the
`repokey-blake2` repository through a shell guard whose command line contains
only file paths, and renders the cron file (0644). Python request parsing
re-validates the whole backup section at the trust boundary. The cron entry
runs `bonesremote backup run --site <site> --keep-days <days> 2>&1 |
systemd-cat -t bonesdeploy-backup` and carries no secret.

BonesRemote gained `commands/backup.rs`: root check, `SiteMutation` lock, the
root-only passphrase file (mode re-verified), a
`<site>_<YYYYMMDD_HHMMSS>` UTC archive of `shared/` via the `borgbackup` crate
(passphrase delivered through `BORG_PASSPHRASE`), and age-based pruning with
`--keep-within <days>d`. Retention is passed by the cron entry, so no remote
backup configuration file exists.

External replication, restore workflows, manual triggers, user-facing backup
commands, passphrase rotation, existing-project migration, and database dumps
remain deliberately out of scope.
