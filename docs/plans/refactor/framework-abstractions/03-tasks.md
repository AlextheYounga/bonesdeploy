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

## Child Plan Decomposition

- [ ] Create and review a child Acta plan for the project configuration boundary,
  including the `.env` parser/writer, project identity callers, duplicate parsers,
  and the dead `bonesinfra_input` contract.
- [ ] Create and review a child Acta plan for Git, SSH, and secrets boundaries,
  including the direct `git`/`ssh` callers, Python bare-repository setup, and
  protected delivery.
- [ ] Create and review a child Acta plan for infrastructure updates and migrations,
  including managed synchronization, `0003-project-infra`, patch markers, and the
  dead Rust migration.
- [ ] Create and review a child Acta plan for provisioning composition, including
  project infrastructure, language runtimes, runtime services, manifest, repeated
  framework runtime workflows, custom-hook overlap, runner access, and paths.
- [ ] Create and review a child Acta plan for the Rust/Python Framework contract,
  including centralized validation, defaults, permission defaults, identity,
  private modules, asset reach-through, and materialization boundaries.
- [ ] Create and review a child Acta plan for Deployment, SiteMutation, SiteState,
  and ReleaseLifecycle boundaries, including kill/recover/doctor bypasses, state
  visibility, duplicated inspectors, and lifecycle invariants.
- [ ] Create and review a child Acta plan for Doctor and shared inspection
  collaborators.
- [ ] Create and review a final child Acta plan for cross-boundary static checks and
  cleanup after the preceding child changes.

## Parent Validation

- [ ] Confirm the parent docs contain no unanswered questions, competing approaches,
  placeholder decisions, or speculative public APIs.
- [ ] Confirm every named child change has one responsibility, one owner, one
  dependency position, and a separate Acta plan deliverable.
- [ ] Confirm the parent scope explicitly excludes implementation of child changes.
- [x] Run `git diff --check`.
- [x] Review the final planning diff for consistency with the Acta skill.

## Completion

- [ ] Mark this parent record complete only after the architecture child plan and
  all required child-plan boundaries are recorded and reviewed.
- [ ] Record the settled child-change order and any deliberately deferred boundary
  in completion notes.

## Completion notes

- Initial implementation is complete; remaining child boundaries are deliberately
  unfinished and remain in the dependency order above.
- Validation passed: `cargo clippy --all-targets --all-features -- -D warnings`,
  `cargo fmt --all -- --check`, `shfmt -d .`, 70 `bonesdeploy` binary unit tests,
  `bonesdeploy-core` tests, and `bonesdeploy` CLI/init integration tests.
- E2E tests were not run by design.
