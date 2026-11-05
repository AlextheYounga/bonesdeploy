# Git Deployment SSH Entry Point Tasks

## Implementation

- [x] Change only `bonesdeploy deploy` to connect through the `git` deploy SSH identity.
- [x] Remove `--config-stdin`; make config sync unconditionally consume stdin and deploy load `/srv/conf/<site>/bones.json`.
- [x] Invoke the two fixed config-sync and deploy commands directly through sudo.
- [x] Replace the wrapper policy with anchored direct BonesRemote sudoers rules.
- [x] Preserve existing deployment state, lock, snapshot, backup-secret, release, and lifecycle behavior.
- [x] Regenerate the embedded BonesInfra wheel after the sudoers template changes.
- [x] Keep control-plane directory permissions at the provisioned `0750` mode and update deployment identity and sudo-boundary documentation.

## Validation

- [x] Add Rust tests proving config sync and deploy have the fixed snapshot contract and local deploy uses both direct sudo commands.
- [x] Add Python tests pinning the complete direct sudoers allowed command set.
- [x] Add denial tests for arbitrary operations, missing/reordered arguments, optional deploy arguments, and trailing arguments.
- [x] Confirm existing tests still prove build scripts run as `<site>-build` and prepare scripts run as `<site>`.
- [x] Run `cargo test --workspace --exclude e2e`.
- [x] Run `cargo clippy`, `cargo fmt`, `ruff check .`, `ruff format .`, `uv run pytest`, regenerate the wheel, and run `shfmt -w .`.
- [x] Review the final diff and confirm no E2E tests were run locally.

## Completion notes

The earlier unprivileged-coordinator and typed-transition approach is
superseded. This change restricts the deployment SSH entry point to `git` while
retaining the existing root-executed BonesRemote lifecycle behind a root-owned
sudoers policy with two anchored direct command forms.

The wheel builder now cleans Python's generated `build/` directory before
packaging, so deleted assets cannot remain in the embedded wheel.

Validation passed: `cargo test --workspace --exclude e2e`, `cargo clippy`,
`cargo fmt`, `ruff check .`, `ruff format .`, `uv run pytest` (494 tests), wheel
regeneration, and `shfmt -w .`. E2E tests were not run.
