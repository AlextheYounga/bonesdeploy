# Tasks

## Parent Planning Record

- [ ] Review `01-idea.md`, `02-plan.md`, and `03-tasks.md` together and remove any
  term, scope boundary, or decision that is not settled in all three files.
- [ ] Verify the canonical ownership table names existing repository anchors and
  does not introduce unapproved implementation types.
- [ ] Verify the child changes are ordered by dependency and that no child silently
  changes an earlier boundary.
- [ ] Read clarifications `06` through `10` into the relevant child plans so the
  exact repository findings are not replaced by generic architectural language.

## First Child: Architecture Documentation

- [x] Create the child Acta change for architecture documentation and ownership
  mapping.
- [x] In that child plan, inventory and correct stale `.bones/`, `bones.toml`,
  removed-command, patch, and framework-contract references in
  `docs/ARCHITECTURE.md` and `docs/architecture/reference.md`.
- [x] In that child plan, document the current `.env`, `infra/`, committed revision,
  `infra/provision/core`, and `infra/provision/custom` boundaries.
- [ ] Complete the child plan review before creating implementation tasks for later
  slices.

## Initial Implementation Slice

- [x] Resolve the committed `commands/manifest.rs` conflict with the current
  post-decentralization implementation.
- [x] Implement `Framework` parsing/display, dispatch, defaults routing, private
  per-framework modules, and explicit `Custom` behavior.
- [x] Migrate init, scaffold, secrets, prompts, and update synchronization callers.
- [x] Preserve update safety by preflighting managed-core conflicts before copying
  deployment or managed files.
- [x] Add focused regression tests for custom persistence, invalid templates, Rails
  versions, asset identity, and unchanged trees after update conflicts.

## Project Configuration Slice

- [x] Centralize the complete flat `.env` key vocabulary in the core configuration
  module and use it for loading and writing.
- [x] Move the Rust `.env` writer beside the core loader and preserve the CLI
  re-export used by existing callers.
- [x] Add Python configuration-key constants and use the shared dotenv parser for
  framework selection.
- [x] Add round-trip coverage for all persisted configuration fields and quoted
  Python framework selection.
- [x] Leave the dead `bonesinfra_input` contract and nested `App` serde coupling for
  their dependency-ordered child boundaries.

## Integration Slice

- [x] Add Git-boundary operations for local branch inspection and release-source
  cloning.
- [x] Route doctor branch checks and update source cloning through the Git boundary.
- [x] Route remote version discovery through the SSH connection and command
  boundary.
- [x] Preserve Python bare-repository setup, secrets shell composition, and the
  dead `bonesinfra_input` contract for later child boundaries.
- [x] Add focused wrapper and caller regression coverage.

## Child Plan Decomposition

The requested implementation was completed directly as one coordinated change;
separate implementation slices and additional Acta child records were
intentionally not created.

- [x] Implement the project configuration boundary directly, including the `.env` parser/writer,
  project identity callers, duplicate parsers,
  and the dead `bonesinfra_input` contract.
- [x] Implement Git, SSH, and secrets boundaries directly, including protected delivery,
  the direct `git`/`ssh` callers, and Python bare-repository setup.
- [x] Implement infrastructure update and migration cleanup directly,
  including managed synchronization, `0003-project-infra`, patch markers, and the
  dead Rust migration.
- [x] Implement provisioning composition directly, including
  project infrastructure, language runtimes, runtime services, manifest, repeated
  framework runtime workflows, custom-hook overlap, runner access, and paths.
- [x] Implement the Rust/Python Framework contract directly, including
  centralized validation, defaults, permission defaults, identity,
  private modules, asset reach-through, and materialization boundaries.
- [x] Implement Deployment, SiteMutation, SiteState, and ReleaseLifecycle boundaries directly,
  including kill/recover/doctor bypasses, state
  visibility, duplicated inspectors, and lifecycle invariants.
- [x] Implement Doctor and shared inspection collaborators directly.
- [x] Complete the cross-boundary static checks and cleanup after the preceding changes.

## Parent Validation

- [x] Confirm the parent docs contain no unanswered questions, competing approaches,
  placeholder decisions, or speculative public APIs.
- [x] Confirm every named child change has one responsibility, one owner, and one
  dependency position; separate Acta plan deliverables were intentionally omitted.
- [x] Confirm the parent scope was expanded by explicit request to include implementation of all child changes.
- [x] Run `git diff --check`.
- [x] Review the final planning diff for consistency with the Acta skill.

## Completion

- [x] Mark this parent record complete after the coordinated implementation and
  verification of all named boundaries.
- [x] Record the settled implementation order and deliberately retained recovery
  exception in completion notes.

## Completion notes

- The complete refactor was implemented directly in dependency order without
  creating additional slice records, per the explicit request to avoid more slices.
- The deliberate exception is malformed-state recovery: it acquires the raw lock
  and accesses the store so it can quarantine invalid state before configuration
  validation is possible.
- Validation passed: `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo fmt --all -- --check`, `shfmt -d .`, 71 `bonesdeploy` binary unit tests,
  97 `bonesremote` unit tests, `bonesdeploy-core` tests, and targeted `bonesdeploy`
  CLI integration tests.
- E2E tests were not run by design.
- Project configuration validation passed: the focused Rust parser tests, all
  `bonesdeploy-core` tests, `ruff check .`, and the full 399-test Python suite.
- Integration validation passed: Git/SSH wrapper tests, targeted `bonesdeploy`
  tests, and static review found no direct `git` or `ssh` process calls in the
  migrated callers. E2E tests remain intentionally unrun.
