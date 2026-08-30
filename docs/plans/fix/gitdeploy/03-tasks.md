# Git-Owned Deployments Tasks

## Implementation

- [ ] Make local deploy connect as `git`, sync config through exact sudo, and invoke deploy without stdin config.
- [ ] Load synchronized snapshots in the remote coordinator and remove the root requirement from orchestration.
- [ ] Move state and locking to the git-readable state boundary with legacy migration and root-protected lock provisioning.
- [ ] Split lifecycle filesystem, service, cancellation, cleanup, and activation mutations into typed privileged operations.
- [ ] Replace sudoers with the complete exact allowlist and add denial tests.
- [ ] Update BonesInfra provisioning and regenerate its embedded wheel.
- [ ] Update architecture, security, context, and README documentation.

## Validation

- [ ] Run `cargo test --workspace --exclude e2e`.
- [ ] Run `cargo clippy`, `cargo fmt`, `ruff check .`, `ruff format .`, `uv run pytest`, and `shfmt -w .`.
- [ ] Review the final diff and confirm no E2E tests were run.

## Completion notes

The entrypoint now connects as `git`, config sync is the only sudoed deploy
entrypoint operation, snapshots are persisted outside `/root`, and deployment
state uses a git-owned root with pre-created root-owned locks. The lifecycle
helper split, migration of existing servers, expanded sudoers allowlist, and
transactional activation changes remain unfinished.
