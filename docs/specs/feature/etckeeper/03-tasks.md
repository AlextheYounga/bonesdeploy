# Tasks

## Implementation

- [x] Add the shared `services/linux/etckeeper.py` operation and native shell
  asset for idempotent initialization and final changed-state commits.
- [x] Add etckeeper to the server baseline package list and queue initialization
  immediately after package installation.
- [x] Append the shared etckeeper final operation to server, site, services,
  SSL, helpers, and runtime provisioning plans, leaving manifest and patch
  flows unchanged.
- [x] Extend host-mode BonesRemote baseline doctor to validate the secure
  etckeeper executable.
- [x] Regenerate the embedded BonesInfra wheel and update focused source tests
  for the new operation boundary.

## Validation

- [x] Test server package membership and initialization ordering.
- [x] Test final-step ordering and exclusions for all mutating, manifest, and
  patch flows.
- [x] Test clean `/etc`, changed `/etc`, and etckeeper failure command paths.
- [x] Test host doctor acceptance and rejection of etckeeper executable states.
- [x] Run targeted Python and Rust tests without running E2E tests.
- [x] Run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff format .`, and
  `ruff check .`, addressing all warnings and errors.

## Completion

- [x] Update `CONTEXT.md` and related security/architecture/operator
  documentation where the server baseline and doctor behavior are described.
- [x] Review the final diff, generated wheel, public behavior, and documentation
  for accidental ignore rules, remote backups, or commits in excluded flows.

## Completion notes

The etckeeper record script is a checked-in `assets/scripts/etckeeper-record.sh.j2`
rendered per flow with a flow-specific commit message (server setup, site setup,
service/SSL/helper/runtime provisioning). It fails clearly when etckeeper is
missing or `/etc` is not a git-backed etckeeper repository, exits successfully
without committing a clean tree, and propagates `etckeeper commit` failures.
`ETCKEEPER_DIR` (honored by etckeeper itself) is the only indirection; it exists
so tests can run the rendered script against a temporary tree with an etckeeper
shim on `PATH`.

The runtime flow wraps the loaded framework/custom deploy through
`etckeeper.commit_changes_after` in `runtime apply`, while the other mutating
orchestrators call `commit_changes` directly as their last statement; both shapes
are covered by ordering tests. The existing SSL handoff test gained the final
commit step. Wheel `bonesinfra-0.3.4` was regenerated via `cargo build-wheel`.

Validation results: 448 Python tests pass (`uv run pytest`), `bonesremote` and
`bonesdeploy` crate tests pass including the new `doctor_baseline` integration
tests, `cargo build-wheel` rebuilds the wheel cleanly, and `cargo fmt`,
`cargo clippy --all-targets`, `shfmt -w .`, `ruff format`, and `ruff check` are
clean. E2E tests were not run, as instructed.
