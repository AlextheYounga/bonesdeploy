# Site Shared Export

<!-- Merged from feature/export -->

## Idea

### Request

Add a local command that exports a site's remote `shared/` directory as a ZIP.

### Problem

Persistent runtime data is stored remotely, but retrieving it currently requires
manual SSH archive creation and copying.

### Definitions

**Shared export:** A live ZIP of one site's remote `shared/` directory,
including hidden files. Nested symbolic links are stored rather than followed.

### Desired outcome

`bonesdeploy site export [--output <path>]` streams the remote `shared/`
directory over the existing root SSH connection into a local `0600` ZIP file.
Without an explicit filename, it uses `<site>-shared-<YYYYMMDD_HHMMSS>.zip` in
UTC. Existing files are not overwritten.

### Scope

- Local CLI arguments, destination handling, and binary SSH streaming.
- A direct, root-run remote `zip` command over the existing privileged SSH
  connection.
- Focused tests and operator documentation.

### Constraints

- Export is local administrative functionality; it uses the existing configured
  root SSH connection, not the `git` deployment identity.
- The archive is a live best-effort view and must not stop the site.
- `shared/.env` is included and the resulting local archive is sensitive.
- Do not run E2E tests.

### Exclusions

- No `bonesremote` command, sudoers rule, BonesInfra change, or wheel update.
- No import, restore, Borg operation, remote archive staging, filtering,
  encryption, progress display, or forced overwrite.


## Plan

### Current behavior

Administrative site commands use `ssh::connect_privileged()` with the configured
root SSH identity. Deployment alone uses the `git` identity and its restricted
sudo command forms. The SSH module has text streaming but no binary download
helper.

### Intended behavior

The local command resolves `--output`, creates a mode-`0600` temporary file
beside its destination, connects as the configured root user, and streams
`zip -q -r -y - shared` from the canonical project root into that file. It
flushes, syncs, and atomically publishes the archive only after the SSH command
succeeds.

### Approach

Keep the implementation entirely in `bonesdeploy`: a focused site command and a
binary-safe SSH download helper. Build the remote archive command locally from
the configured canonical project root and quote it for the SSH shell. Remove the
previous remote command, sudoers, test, documentation, and wheel changes.

### Responsibilities and boundaries

| Boundary | Responsibility |
| --- | --- |
| `commands/site/export.rs` | CLI orchestration, destination handling, and local publication. |
| `infra/ssh.rs` | Raw binary stdout transfer and concurrent stderr draining. |
| Existing root SSH connection | Runs the fixed archive command on the configured host. |

### Affected areas

- BonesDeploy CLI, focused export module, SSH integration, and tests.
- README, context, skill command reference, and architecture documentation.

### Decisions

- Use root SSH because export is an administrative local-machine operation and
  root is already the configured administrative connection.
- Do not add a server command or sudo policy because the deployment user is not
  part of this workflow.
- Use `zip -y` so links cannot cause the archive to read outside `shared/`.
- Refuse overwrite and stage in the destination directory to avoid publishing a
  partial archive.

### Risks

- Mixing diagnostics with stdout corrupts the ZIP stream.
- Local or remote failure must not publish partial output.
- Exports contain secrets and concurrent application writes are not snapshot
  consistent.

### Validation

- Test CLI parsing, destination resolution, permissions, no-clobber behavior,
  cleanup, and remote command construction.
- Run affected Rust tests, `cargo fmt`, `cargo clippy`, `shfmt -w .`, and the
  required Python formatting, lint, and test checks without E2E tests.


## Tasks

### Implementation

- [x] Remove the mistaken BonesRemote, sudoers, and embedded-wheel export path.
- [x] Stream the fixed archive command through the configured root SSH session.
- [x] Preserve local no-clobber, `0600`, sync, and cleanup behavior.

### Validation

- [x] Update focused tests for direct root SSH command construction and local
  archive publication.
- [x] Run the required non-E2E test, formatter, and lint commands.

### Completion

- [x] Update operator and architecture documentation to distinguish root export
  from the `git` deployment boundary.
- [x] Review the final diff for removed sudoers, wheel, and remote-command work.

### Completion notes

The clarification removed all server-side export work. The final implementation
adds only a local command, a binary SSH download helper, and direct root SSH
invocation of the fixed `zip -q -r -y - shared` archive command. It creates a
same-directory mode-`0600` temporary file, verifies the remote command, syncs
the file, then publishes with no-clobber atomic persistence.

Validation: `cargo test --workspace --exclude e2e`,
`cargo clippy --workspace --exclude e2e --all-targets`, `cargo fmt`,
`shfmt -w .`, `uv run ruff format .`, and `uv run ruff check .` pass. E2E tests
were not run.


## Clarification

### Trigger

The export is a local administrative operation. The configured root SSH
connection already serves setup and administrative commands; `git` exists only
for deployment.

### Decision

`bonesdeploy site export` connects as the configured root user and runs the
fixed archive command directly over SSH. It does not invoke `bonesremote`, use
the deployment identity, or require sudo. This reuses the existing
administrative boundary and avoids adding server-side authority for a
local-machine operation.

### Supersedes

The prior export plan's decision to stream through a root-backed BonesRemote
`export` subcommand authorized for the `git` deployment identity by a new
sudoers rule.

### Effect on the record

`01-idea.md` now defines export as root-SSH local administration and excludes
BonesRemote, sudoers, and BonesInfra changes. `02-plan.md` assigns the remote
archive invocation to the existing privileged SSH connection. `03-tasks.md`
tracks removal of the mistaken server-side path and validation of the simplified
local implementation.
