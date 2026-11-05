# Plan

## Current behavior

Administrative site commands use `ssh::connect_privileged()` with the configured
root SSH identity. Deployment alone uses the `git` identity and its restricted
sudo command forms. The SSH module has text streaming but no binary download
helper.

## Intended behavior

The local command resolves `--output`, creates a mode-`0600` temporary file
beside its destination, connects as the configured root user, and streams
`zip -q -r -y - shared` from the canonical project root into that file. It
flushes, syncs, and atomically publishes the archive only after the SSH command
succeeds.

## Approach

Keep the implementation entirely in `bonesdeploy`: a focused site command and a
binary-safe SSH download helper. Build the remote archive command locally from
the configured canonical project root and quote it for the SSH shell. Remove the
previous remote command, sudoers, test, documentation, and wheel changes.

## Responsibilities and boundaries

| Boundary | Responsibility |
| --- | --- |
| `commands/site/export.rs` | CLI orchestration, destination handling, and local publication. |
| `infra/ssh.rs` | Raw binary stdout transfer and concurrent stderr draining. |
| Existing root SSH connection | Runs the fixed archive command on the configured host. |

## Affected areas

- BonesDeploy CLI, focused export module, SSH integration, and tests.
- README, context, skill command reference, and architecture documentation.

## Decisions

- Use root SSH because export is an administrative local-machine operation and
  root is already the configured administrative connection.
- Do not add a server command or sudo policy because the deployment user is not
  part of this workflow.
- Use `zip -y` so links cannot cause the archive to read outside `shared/`.
- Refuse overwrite and stage in the destination directory to avoid publishing a
  partial archive.

## Risks

- Mixing diagnostics with stdout corrupts the ZIP stream.
- Local or remote failure must not publish partial output.
- Exports contain secrets and concurrent application writes are not snapshot
  consistent.

## Validation

- Test CLI parsing, destination resolution, permissions, no-clobber behavior,
  cleanup, and remote command construction.
- Run affected Rust tests, `cargo fmt`, `cargo clippy`, `shfmt -w .`, and the
  required Python formatting, lint, and test checks without E2E tests.
