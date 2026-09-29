# Tasks

## Implementation

- [ ] Add named transient build-script units and inspectable systemd results so
      BonesRemote distinguishes `Result=timeout` from ordinary script exits.
- [ ] Add root-owned build-user cgroup termination that writes `cgroup.kill`,
      requests nonblocking user-manager stop, and proves within a bounded
      deadline that the slice is absent or unpopulated.
- [ ] Make `BuildContainer` kill the build-user cgroup on timeout and suppress
      Podman cleanup through the terminated session while preserving ordinary
      success and failure cleanup.
- [ ] Replace release cancellation's build-user readiness and container-removal
      sequence with build-user cgroup termination before coordinator shutdown.
- [ ] Add Next/Nuxt-specific Node `25.9.0` runtime defaults and render the same
      selected runtime pin into their generated `.env.build` files without
      changing other framework defaults or explicit overrides.
- [ ] Make native build-user provisioning fail when the active dedicated slice
      does not expose cgroup-v2 `cgroup.kill`.
- [ ] Update security invariants, context, and generated assertion inventory to
      describe cgroup containment and framework-specific Node defaults.

## Validation

- [ ] Run focused BonesRemote tests proving named timeout detection, cgroup kill
      targeting, bounded success/failure verification, timeout cleanup
      suppression, and fail-closed cancellation behavior.
- [ ] Run focused BonesDeploy tests proving Next/Nuxt build/runtime pins are
      `25.9.0`, an explicit selected version remains aligned, and
      non-Next/Nuxt runtime defaults remain `24.19.0`.
- [ ] Run focused BonesInfra tests proving provisioning requires the build-user
      slice's cgroup-v2 kill control.
- [ ] Run `cargo test --workspace --exclude e2e` and resolve every non-E2E test
      failure without running the E2E suite.
- [ ] Run `cargo clippy --workspace --exclude e2e --all-targets -- -D warnings`,
      `cargo fmt --all -- --check`, `shfmt -w .`, and `git diff --check` with no
      warnings or errors.
- [ ] Run `uv run pytest`, `ruff check .`, and `ruff format .` from
      `crates/bonesinfra/python` with no failures or formatting drift.

## Completion

- [ ] Review the final diff for stale weak-kill claims, obsolete helpers,
      accidental framework default changes, and unrelated edits.
- [ ] Record validation results, deviations, discoveries, and deliberately
      unfinished work in the completion notes.

## Completion notes

Implementation has not started. The planning record awaits human approval.
