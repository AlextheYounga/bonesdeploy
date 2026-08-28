# Tasks

## Implementation

- [ ] Add the typed backup configuration, defaults, validation, managed `.env`
  keys, and transport fields while keeping the passphrase out of deployment
  and control-plane descriptors.
- [ ] Add canonical Rust and Python backup paths for the per-site `.borg`
  repository and remote passphrase file.
- [ ] Add BonesInfra provisioning for Borg, root-owned backup directories,
  secure passphrase installation, and `/etc/cron.d` rendering with the default
  midnight schedule and configured retention passed to `bonesremote`.
- [ ] Add the `borgbackup` dependency and implement the internal root-only
  `bonesremote backup run --site <site>` operation using `SiteMutation`.
- [ ] Implement Borg archive creation for `shared/`, UTC project-based archive
  naming, and age-based pruning.
- [ ] Add cron/journald command wiring without exposing the Borg passphrase in
  arguments or logs.
- [ ] Update command, architecture, security, and user documentation with the
  backup layout, secret handling, cron configuration, retention, and limits.

## Validation

- [ ] Add core integration tests for configuration defaults, cron validation,
  retention validation, transport serialization, and passphrase exclusion.
- [ ] Add BonesInfra tests for package/layout provisioning, file modes, and
  cron output.
- [ ] Add BonesRemote tests for archive path and naming, Borg command inputs,
  retention, and failure behavior.
- [ ] Run `cargo test --workspace --exclude e2e` and address all failures.
- [ ] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, `ruff check .`,
  `ruff format .`, and `uv run pytest` in the applicable Python directory;
  do not run e2e tests.

## Completion

- [ ] Review the final diff for secret leakage, path drift between Rust and
  Python, privileged command injection, and files exceeding project limits.
- [ ] Confirm all implementation and validation tasks are complete before
  recording completion notes.

## Completion notes

Implementation has not started. This record captures the approved feature
scope and implementation direction; external replication, restore workflows,
manual triggers, release archives, database dumps, existing-project migration,
and passphrase rotation remain deliberately unfinished scope.
