# Tasks

## Implementation

- [ ] Change the canonical Rust and Python deploy-account values to
  `bonesdeploy`, then update routine SSH, BonesRemote doctor, and identity
  fixtures so every current consumer resolves the renamed account.
- [ ] Update BonesInfra fresh-server provisioning and authorized-key assertions
  to create `/home/bonesdeploy` and install the configured key without adding
  legacy-account detection, rename, copy, deletion, or fallback behavior.
- [ ] Render and test `/etc/sudoers.d/bonesdeploy` for the `bonesdeploy`
  principal while preserving the exact anchored config-sync and deploy command
  allowlist.
- [ ] Update security and site-doctor tests to prove `bonesdeploy` is required,
  remains outside runtime groups, and receives no Docker or broader sudo
  authority.
- [ ] Update affected E2E harness expectations without running E2E scenarios,
  and refresh generated or embedded BonesInfra artifacts required by the
  repository after Python source and asset changes.
- [ ] Replace current deploy-account uses of `git` and `/home/git` in user,
  architecture, security, context, and embedded skill documentation while
  preserving references to actual local Git behavior and historical plans.

## Validation

- [ ] Run focused Rust tests for deploy-user defaults, SSH transport, and
  BonesRemote account/isolation diagnostics; confirm expected output names
  `bonesdeploy` and group-isolation failures remain enforced.
- [ ] Run focused BonesInfra tests for user provisioning, authorized-key paths,
  sudoers rendering, and server provisioning order; confirm no migration or
  compatibility operation is emitted.
- [ ] Search current source, assets, tests, and operational documentation for
  `git` and `/home/git`; classify every remaining match as genuine Git behavior
  or immutable historical context.
- [ ] Run `cargo test --workspace --exclude e2e --no-fail-fast`, the complete
  BonesInfra Python test suite, and repository generated-artifact checks without
  executing E2E scenarios.
- [ ] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, `ruff check .`, `ruff format
  .`, and `git diff --check`, addressing every warning, error, and formatting
  change.

## Completion

- [ ] Review the final diff for legacy-account migration, SSH fallback,
  privilege drift, runtime-group access, unrelated Git terminology changes,
  stale generated content, and documentation that fails to state the breaking
  reprovisioning requirement.
- [ ] Record validation evidence, meaningful deviations, and deliberately
  unfinished work in the completion notes.

## Completion notes

Implementation has not started. Human approval of this planning record is
required before application files are changed.
