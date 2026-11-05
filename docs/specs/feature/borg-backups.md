# Borg Backups

<!-- Merged from feature/backups -->

## Idea

### Request

Add automated Borg backups to BonesDeploy through `bonesremote`, using the
`borgbackup` Rust crate. Backups remain on the deployment server so the user
can arrange external replication separately.

### Problem

BonesDeploy currently deploys application releases and shared runtime data but
does not provide a repeatable backup workflow for shared runtime data. Users
must independently design backup commands, scheduling, retention, and
passphrase handling for every server.

### Definitions

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

### Desired outcome

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

### Scope

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

### Constraints

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

### Exclusions

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


## Plan

### Current behavior

`bonesdeploy-core` owns the `Bones` configuration model, local managed `.env`
grammar, transport structs, and product paths. The current model has no backup
section or secret. `RemoteDeploymentConfig` is used for deploy-time settings,
while `ProvisioningRequest` carries typed site provisioning data.

`bonesinfra` owns server and site provisioning. Its base package list does not
currently guarantee Borg. Its Python systemd helpers render units and register
per-site services, but no backup scheduler exists. Existing infrastructure
updates use the version-gated patch registry.

`bonesremote` runs as root and owns remote deployment execution. Mutating site
operations use `SiteMutation` and the existing deployment lock. The crate has
no Borg integration or backup command. The `borgbackup` crate exposes
synchronous Borg operations including `init`, `create`, and `prune`, and passes
the passphrase through `BORG_PASSPHRASE`.

The current documentation describes releases and shared data but no backup
layout, schedule, retention, or secret handling.

### Intended behavior

The local configuration gains a backup section containing the cron expression,
retention days, and generated Borg passphrase. The schedule accepts validated
five-field crontab syntax and defaults to `0 0 * * *`; retention defaults to
30 days. The passphrase is generated by `bonesdeploy init` only when absent.
This feature applies only to newly initialized projects.

Site provisioning sends the passphrase only in the typed provisioning request.
BonesInfra writes it atomically to the root-owned per-site `.borg_passphrase`
file with mode `0600`, installs Borg, creates the root-owned repository
`/var/lib/bonesdeploy/backups/<site>.borg` using `repokey-blake2`, and installs
the root-owned cron file `/etc/cron.d/bonesdeploy-<site>-backup`.

Cron invokes one root-only `bonesremote backup run --site <site> --keep-days
<retention>` operation, so the cron entry carries the configured retention and
the remote process needs no additional backup configuration file.
The operation acquires the existing site mutation guard and invokes Borg to
archive only the site's `shared/` directory. Archive names contain the site
name and a UTC timestamp. It then prunes archives older than the configured
30-day retention window. Output is sent to journald by the cron command.

### Approach

1. Extend the shared configuration model with a typed backup section and
   validation for cron syntax, retention, and passphrase values. Add the
   backup fields to local `.env` serialization and the provisioning transport,
   while keeping the passphrase out of `RemoteDeploymentConfig` and the
   control-plane snapshot.
2. Add canonical backup paths to `bonesdeploy-core::paths` and the mirrored
   BonesInfra `DeploymentPaths` representation.
3. Add a BonesInfra backup provisioning module. Install Borg, write the
   passphrase securely, create the repository, and render the per-site cron
   file. Existing projects and existing-host migration are outside this
   feature.
4. Add a focused `bonesremote::backup` module and internal `backup run` CLI
   command. It reads the root-only passphrase file, derives paths from the
   canonical site configuration, accepts retention from the cron argument,
   calls the `borgbackup` synchronous API to archive `shared/`, names the
   archive with the site and UTC timestamp, and prunes by age.
5. Send cron output through `systemd-cat` and preserve actionable Borg
   failures in journald without exposing the passphrase.
6. Add focused Rust and Python tests, then update user and architecture
   documentation with the backup layout, configuration, limitations, and
   security rules.

### Responsibilities and boundaries

| Responsibility | Owner |
|---|---|
| Backup configuration, defaults, and validation | `bonesdeploy-core::config` |
| Local `.env` managed-key handling | `bonesdeploy-core::config::local_env` |
| Provisioning request transport | `bonesdeploy-core::config::transport` and BonesInfra request parsing |
| Product-owned backup paths | `bonesdeploy-core::paths` and mirrored `DeploymentPaths` |
| Borg installation, directories, passphrase file, and cron | BonesInfra site/server provisioning |
| Backup execution and Borg adapter | `bonesremote` |
| Site serialization | Existing `SiteMutation` and deployment lock |
| Journald integration | Cron entry and the backup process output boundary |
| Feature documentation | `README.md`, `CONTEXT.md`, Python context, architecture/security docs |

### Affected areas

- `crates/bonesdeploy-core/src/config/model.rs`
- `crates/bonesdeploy-core/src/config/local_env.rs`
- `crates/bonesdeploy-core/src/config/transport.rs`
- `crates/bonesdeploy-core/src/paths.rs`
- `crates/bonesinfra/python/src/bonesinfra/config/paths.py`
- `crates/bonesinfra/python/src/bonesinfra` provisioning and request parsing
- `crates/bonesinfra/python/src/bonesinfra/assets/cron/` or the established
  native provisioning asset location
- `crates/bonesremote/Cargo.toml` and `crates/bonesremote/src/backup/`
- `crates/bonesremote/src/cli/args.rs` and `dispatch.rs`
- Rust and Python integration tests for the affected boundaries
- `README.md`, `CONTEXT.md`, `crates/bonesinfra/python/CONTEXT.md`,
  `docs/ARCHITECTURE.md`, and `docs/security/invariants.md`

### Decisions

**Backups use a local per-site Borg repository.** This satisfies the current
requirement without silently choosing an external provider or transport. Users
remain responsible for copying the repository elsewhere.

**Cron is the sole trigger.** Linux cron is native to the supported Debian and
Ubuntu hosts and keeps scheduling independent from systemd service lifecycle
restarts. No alternate trigger is provisioned.

**The passphrase is a local managed secret and a root-only remote file.** The
local `.env` is already gitignored and is the source for provisioning. A
dedicated remote file keeps the secret out of command-line arguments, JSON
snapshots, and logs while allowing Borg to receive it through its supported
environment variable.

**Backups include only shared data.** Releases are reproducible from Git and
are deliberately excluded from v1. Database logical dumps are deferred to a
future feature.

**Retention is age-based with a 30-day default.** The requirement is expressed
in days rather than a second archive-count policy, so Borg prune arguments are
deterministic and easy to explain.

### Risks

- A passphrase accidentally appearing in a transport payload, process list, or
  journal would compromise all archives; secret-flow tests and review must
  enforce the boundary.
- Root-owned backups increase disk usage and can fill the server; retention
  must run after successful archive creation.
- Cron expressions and site names cross privileged provisioning boundaries and
  must be validated before being rendered into cron files or commands.
- A backup may overlap application writes, so the resulting contract is
  shared-file consistency, not filesystem-wide quiescence.

### Validation

- Core tests accept the default schedule and retention, reject malformed
  crontab expressions and invalid retention, round-trip backup configuration,
  and prove the passphrase is excluded from deployment/control-plane
  transport.
- BonesInfra tests verify Borg package installation, root-only directories and
  passphrase permissions, and cron rendering.
- BonesRemote tests verify path selection, archive naming, Borg argument
  construction, retention handling, and failure propagation without running
  e2e tests.
- `cargo test --workspace --exclude e2e` passes.
- `cargo clippy`, `cargo fmt`, `shfmt -w .`, and the required Python checks pass.
- Documentation and security review confirm the repository layout, secret
  flow, cron-only trigger, 30-day retention, shared-only archive contents, and
  new-project-only scope.


## Tasks

### Implementation

- [x] Add the typed backup configuration (`Backup` in `config/model.rs`), the
  managed `.env` keys (`BONES_BACKUP_SCHEDULE`, `BONES_BACKUP_RETENTION_DAYS`,
  `BONES_BORG_PASSPHRASE`), and the transport field, keeping the passphrase out
  of deployment and control-plane descriptors.
- [x] Generate the Borg passphrase in `bonesdeploy init`; re-init preserves the
  existing backup configuration and only generates when absent.
- [x] Add canonical Rust and Python backup paths for the per-site `.borg`
  repository and remote passphrase file.
- [x] Add BonesInfra provisioning for Borg, root-owned backup directories,
  passphrase installation, and `/etc/cron.d` rendering; Python validates the
  cron schedule and retention at the provisioning boundary.
- [x] Implement the internal root-only `bonesremote backup run` using the
  `borgbackup` crate: `shared/` archive with UTC naming and age-based pruning.
- [x] Wire the cron entry to journald without exposing the passphrase.
- [x] Update command, architecture, and user documentation.

### Validation

- [x] Core tests: `.env` round-trip with defaults, provisioning JSON carries
  the backup section, `RemoteDeploymentConfig` excludes secrets.
- [x] Init tests: passphrase generated when absent, existing configuration
  preserved on re-init.
- [x] BonesInfra tests: invalid cron/retention rejected, provisioning
  operations and file modes, cron renders without the passphrase.
- [x] BonesRemote tests: archive naming, Borg command inputs, passphrase-file
  contract.
- [x] `cargo test --workspace --exclude e2e`, `cargo clippy`, `cargo fmt`,
  `shfmt -w .`, `ruff check .`, `ruff format .`, `uv run pytest`, and
  `git diff --check`; e2e tests are not run locally.

### Completion notes

Data flow: `.env` -> Rust config parsing -> provisioning JSON ->
BonesInfra/PyInfra (Borg package, repository, passphrase file, cron) -> cron
runs `bonesremote backup run` -> `borgbackup` crate archives `shared/` and
prunes by retention.

Ownership: BonesInfra (`services/linux/backup.py`) is the only provisioner;
BonesRemote (`commands/backup.rs`) is the only scheduled executor. Rust parses
the three `.env` values without backup-specific validation; cron-schedule and
retention safety is enforced once in Python immediately before rendering the
privileged cron file. The passphrase is an opaque string: generated by init,
transported in the site request, written to the root-only passphrase file, and
passed to Borg through its environment — never in argv, templates, logs, or
`RemoteDeploymentConfig`. File ownership and modes are a provisioning
contract, not a runtime check.

External replication, restore workflows, manual triggers, user-facing backup
commands, passphrase rotation, existing-project migration, and database dumps
remain deliberately out of scope.


## Clarification: Simplify Local Backups

### Trigger

The backup scope was narrowed after reviewing the planning record. Database
logical dumps are deferred, release trees can be recreated by deploying from
Git, and existing projects are outside v1.

### Decision

V1 creates a per-project Borg repository at
`/var/lib/bonesdeploy/backups/<project_name>.borg` and archives only the
project's `shared/` directory. Borg uses the `repokey-blake2` encryption mode,
and archive names contain the project name and a UTC timestamp. Cron is the
only trigger; the internal `bonesremote backup run` operation is not a
user-facing backup command.

The Borg passphrase is generated by `bonesdeploy init` only when the local
managed value is absent. Existing projects are not migrated or provisioned by
this feature. Passphrase rotation is not supported in v1.

### Supersedes

- The earlier requirement to archive `releases/`.
- The earlier requirement for PostgreSQL, MySQL, MariaDB, and MongoDB logical
  dumps.
- The earlier staging-directory design.
- The earlier requirement for an existing-host backup migration patch.
- Any implication that users can invoke a backup command manually.

### Effect On The Record

- `01-idea.md` now defines shared-data-only archives, the `.borg` repository
  path, `repokey-blake2`, new-project-only support, and no manual trigger.
- `02-plan.md` now provisions the repository and cron entry without database
  credentials, dump tools, staging, or an existing-host migration patch.
- `03-tasks.md` now tracks only the reduced implementation and validation work.
