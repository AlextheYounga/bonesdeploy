# Git-Owned Deployments Tasks

## Implementation

- [ ] Make local deploy connect as `git`, sync config through exact sudo, and invoke deploy without stdin config.
- [x] Load synchronized snapshots in the remote coordinator; `bonesremote deploy` accepts no second descriptor and no longer requires `--config-stdin`.
- [ ] Move state and locking to the git-readable state boundary with legacy migration and root-protected lock provisioning.
- [ ] Split lifecycle filesystem, service, cancellation, cleanup, and activation mutations into typed privileged operations.
- [x] Remove the dead sudoers allowlist; the deploy user receives no sudo grants until typed transitions exist.
- [x] Update BonesInfra provisioning and regenerate its embedded wheel.
- [x] Update architecture, security, context, and README documentation.

## Validation

- [ ] Run `cargo test --workspace --exclude e2e`.
- [ ] Run `cargo clippy`, `cargo fmt`, `ruff check .`, `ruff format .`, `uv run pytest`, and `shfmt -w .`.
- [ ] Review the final diff and confirm no E2E tests were run.

## Completion notes

A first pass moved the deploy entrypoint to `git` and the state/lock/snapshot
paths out of `/root`, but review found deployment non-operational: the
coordinator still called root-only lifecycle steps in-process with no elevation
mechanism, no `config sync` sudoers rule, a root-owned unreadable snapshot
directory, a git-replaceable lock directory, a Borg passphrase path split
between provisioner and reader, and no legacy state migration.

The entrypoint, root gate, and state/lock/snapshot root changes were therefore
reverted until the typed privileged-transition split exists. What remains and
works:

- `bonesdeploy deploy` connects as the configured SSH user, synchronizes the
  snapshot to `/srv/conf/<site>/bones.json`, and runs `bonesremote deploy
  --site <site>`, which requires root and loads the synchronized snapshot. The
  coordinator no longer accepts a second descriptor over stdin.
- Deployment state, locks, and the Borg passphrase stay under the root-owned
  `/root/.config/bonesremote/sites/<site>` tree, matching provisioning and the
  doctor baseline; no migration is required.
- The sudoers drop-in grants nothing: review confirmed no invocation path uses
  `sudo bonesremote` as the deploy user. The file documents the trust model and
  the bar for adding future typed transitions.

Git-owned deployments remain planned work: they require the full typed
privileged-transition split (sudoers allowlist, filesystem re-permissioning,
per-step revalidation, and legacy state migration) to land together before the
coordinator can run as `git`.
