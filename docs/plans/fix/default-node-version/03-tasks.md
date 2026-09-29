# Default Node Version Tasks

## Implementation

- [x] Replace empty Node declarations in all seven framework build-environment
  assets with the shared `{node_version}` placeholder.
- [x] Render `{node_version}` centrally from `Runtime::node_version` after each
  framework's existing build-environment rendering.
- [x] Update framework scaffolding coverage to require the canonical
  `NODE_VERSION=24.19.0`, a configured exact override, and preservation of an
  existing `.env.build`.
- [x] Update context and architecture documentation to state that fresh build
  environments receive the canonical exact Node default.

## Validation

- [x] Run the focused BonesDeploy asset and framework configuration tests and
  confirm all framework defaults and overrides render correctly.
- [x] Run the relevant crate and workspace test suites, excluding the end-to-end
  package, and address every failure.
- [x] Run wheel consistency validation and regenerate the embedded wheel only
  when the repository reports drift.
- [x] Run `cargo clippy`, `cargo fmt`, and `shfmt -w .` and address every warning
  or formatting change.

## Completion

- [x] Search for stale empty `NODE_VERSION=` scaffold expectations and update
  only contracts affected by the new default.
- [x] Review the final diff for duplicated Node defaults, weakened exact-version
  checks, unrelated edits, and accidental changes to pre-existing worktree
  documentation.
- [x] Run `git diff --check` and record the completed validation results below.

## Completion notes

Implemented shared Node-version rendering for all seven framework build
environments. Fresh scaffolds now pin the existing canonical `24.19.0` default,
and configured exact versions render without introducing another version
literal or weakening deployment validation. Existing `.env.build` files and
the intentionally blank custom-project fallback remain unchanged.

Focused asset and framework tests passed (22 tests). The full workspace suite
passed with the `e2e` package excluded, including 509 BonesInfra Python tests.
The BonesInfra build-time wheel consistency check passed, so the wheel was not
regenerated. Workspace Clippy, Rust formatting, shell formatting, stale-contract
searches, and `git diff --check` passed. The end-to-end suite was not run.

The manual setup report and completed SSL router task updates were already in
the worktree before this branch and were not modified as part of this fix.
