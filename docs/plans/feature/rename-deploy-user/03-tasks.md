# Tasks

## Implementation

- [ ] Change the canonical Rust and Python deploy-account values to
  `deploy`, then update routine SSH, BonesRemote doctor, and identity
  fixtures so every current consumer resolves the renamed account.
- [ ] Update BonesInfra fresh-server provisioning and authorized-key assertions
  to create `/home/deploy` and install the configured key without adding
  legacy-account detection, rename, copy, deletion, or fallback behavior.
- [ ] Render and test `/etc/sudoers.d/bonesdeploy` for the `deploy`
  principal while preserving the exact anchored config-sync and deploy command
  allowlist.
- [ ] Update security and site-doctor tests to prove `deploy` is required,
  remains outside runtime groups, and receives no Docker or broader sudo
  authority.
- [x] Update affected E2E harness expectations without running E2E scenarios,
  and refresh generated or embedded BonesInfra artifacts required by the
  repository after Python source and asset changes.
- [ ] Replace current deploy-account uses of `git`, `bonesdeploy`, and `/home/git`
  in user,
  architecture, security, context, and embedded skill documentation while
  preserving references to actual local Git behavior and historical plans.

## Validation

- [x] Run focused Rust tests for deploy-user defaults, SSH transport, and
  BonesRemote account/isolation diagnostics; confirm expected output names
  `bonesdeploy` and group-isolation failures remain enforced.
- [x] Run focused BonesInfra tests for user provisioning, authorized-key paths,
  sudoers rendering, and server provisioning order; confirm no migration or
  compatibility operation is emitted.
- [ ] Search current source, assets, tests, and operational documentation for
  `git`, `bonesdeploy`, and `/home/git`; classify every remaining match as genuine Git behavior
  or immutable historical context.
- [x] Run `cargo test --workspace --exclude e2e --no-fail-fast`, the complete
  BonesInfra Python test suite, and repository generated-artifact checks without
  executing E2E scenarios.
- [x] Run `cargo clippy`, `cargo fmt`, `shfmt -w .`, `ruff check .`, `ruff format
  .`, and `git diff --check`, addressing every warning, error, and formatting
  change.

## Completion

- [x] Review the final diff for legacy-account migration, SSH fallback,
  privilege drift, runtime-group access, unrelated Git terminology changes,
  stale generated content, and documentation that fails to state the breaking
  reprovisioning requirement.
- [x] Record validation evidence, meaningful deviations, and deliberately
  unfinished work in the completion notes.

## Completion notes

Implemented the breaking rename without migration or fallback behavior. The
canonical Rust and Python identities, provisioning, sudoers, diagnostics,
tests, E2E expectations, embedded wheel, and current documentation now use
`bonesdeploy`. The stale repository-era `REPO_PATH` test value was aligned with
the current empty production repository contract.

Focused Rust and Python tests passed, including rendered authorized-key and
sudoers boundaries. The complete 506-test BonesInfra suite, embedded artifact
checks, and `cargo test --workspace --exclude e2e --no-fail-fast` passed.
`cargo clippy --workspace --exclude e2e --all-targets`, `cargo fmt`, `shfmt -w
.`, Ruff checks and formatting, and `git diff --check` completed cleanly.

E2E expectations were updated to require `bonesdeploy` and reject `git`, but
E2E scenarios were deliberately not run as required by repository policy.
Remaining deploy-account references to `git` occur only in breaking-change
documentation and immutable historical plans; other matches are genuine Git
operations or metadata. There were no deviations from the approved plan and no
unfinished implementation work.
