# Tasks

## Implementation

- [x] Remove the mistaken BonesRemote, sudoers, and embedded-wheel export path.
- [x] Stream the fixed archive command through the configured root SSH session.
- [x] Preserve local no-clobber, `0600`, sync, and cleanup behavior.

## Validation

- [x] Update focused tests for direct root SSH command construction and local
  archive publication.
- [x] Run the required non-E2E test, formatter, and lint commands.

## Completion

- [x] Update operator and architecture documentation to distinguish root export
  from the `git` deployment boundary.
- [x] Review the final diff for removed sudoers, wheel, and remote-command work.

## Completion notes

The clarification removed all server-side export work. The final implementation
adds only a local command, a binary SSH download helper, and direct root SSH
invocation of the fixed `zip -q -r -y - shared` archive command. It creates a
same-directory mode-`0600` temporary file, verifies the remote command, syncs
the file, then publishes with no-clobber atomic persistence.

Validation: `cargo test --workspace --exclude e2e`,
`cargo clippy --workspace --exclude e2e --all-targets`, `cargo fmt`,
`shfmt -w .`, `uv run ruff format .`, and `uv run ruff check .` pass. E2E tests
were not run.
