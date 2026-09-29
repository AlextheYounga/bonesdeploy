# Tasks

## Implementation

- [x] Add named transient build-script units and inspectable systemd results so
      BonesRemote distinguishes `Result=timeout` from ordinary script exits.
- [x] Add root-owned build-user cgroup termination that writes `cgroup.kill`,
      requests nonblocking user-manager stop, and proves within a bounded
      deadline that the slice is absent or unpopulated.
- [x] Make `BuildContainer` kill the build-user cgroup on timeout and suppress
      Podman cleanup through the terminated session while preserving ordinary
      success and failure cleanup.
- [x] Replace release cancellation's build-user readiness and container-removal
      sequence with build-user cgroup termination before coordinator shutdown.
- [x] Preserve active deployment state and build artifacts when cgroup
      termination cannot be proven, without changing ordinary abort cleanup.
- [x] Add Next/Nuxt-specific Node `25.9.0` runtime defaults and render the same
      selected runtime pin into their generated `.env.build` files without
      changing other framework defaults or explicit overrides.
- [x] Make native build-user provisioning fail when the active dedicated slice
      does not expose cgroup-v2 `cgroup.kill`.
- [x] Update security invariants, context, and generated assertion inventory to
      describe cgroup containment and framework-specific Node defaults.

## Validation

- [x] Run focused BonesRemote tests proving named timeout detection, cgroup kill
      targeting, bounded success/failure verification, timeout cleanup
      suppression, fail-closed cancellation behavior, and coordinator state
      preservation after containment failure.
- [x] Run focused BonesDeploy tests proving Next/Nuxt build/runtime pins are
      `25.9.0`, an explicit selected version remains aligned, and
      non-Next/Nuxt runtime defaults remain `24.19.0`.
- [x] Run focused BonesInfra tests proving provisioning requires the build-user
      slice's cgroup-v2 kill control.
- [x] Run `cargo test --workspace --exclude e2e` and resolve every non-E2E test
      failure without running the E2E suite.
- [x] Run `cargo clippy --workspace --exclude e2e --all-targets -- -D warnings`,
      `cargo fmt --all -- --check`, `shfmt -w .`, and `git diff --check` with no
      warnings or errors.
- [x] Run `uv run pytest`, `ruff check .`, and `ruff format .` from
      `crates/bonesinfra/python` with no failures or formatting drift.

## Completion

- [x] Review the final diff for stale weak-kill claims, obsolete helpers,
      accidental framework default changes, and unrelated edits.
- [x] Record validation results, deviations, discoveries, and deliberately
      unfinished work in the completion notes.

## Completion notes

Implementation completed on 2026-09-29.

- Timed script units use a deterministic execution ordinal rather than the
  script filename so every accepted numbered filename remains compatible with
  systemd unit-name syntax.
- Containment errors are typed so the deployment coordinator preserves active
  state and build artifacts for operator recovery. This required the
  clarification recorded in `04-containment-error-clarity.md`.
- The committed BonesInfra wheel was regenerated after adding the provisioning
  check.
- Focused Rust tests passed for named-unit construction, timeout-result
  classification, cgroup targeting, missing controls, successful empty-cgroup
  verification, populated-cgroup failure, and typed containment errors.
- Focused framework tests passed for Next/Nuxt `25.9.0` defaults, explicit
  selected-version rendering, and the unchanged `24.19.0` fallback.
- All 510 BonesInfra Python tests passed, along with Ruff checks and formatting.
- `cargo test --workspace --exclude e2e`, strict workspace Clippy, Rust
  formatting, `shfmt -w .`, and `git diff --check` passed.
- The E2E suite and privileged mutation of a live systemd/cgroup hierarchy were
  deliberately not run, as required by the plan.
