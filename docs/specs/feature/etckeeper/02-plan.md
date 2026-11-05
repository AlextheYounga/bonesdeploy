# Plan

## Current behavior

`bonesinfra/cli/commands/server/packages.py` installs the host baseline package
list, and `deploy_server_setup` in `cli/commands/server/__init__.py` runs the
host-wide operations in order. The Debian package list does not include
etckeeper, and no operation initializes or commits `/etc`.

The remote Python apply commands are registered in `cli/app.py`. They use the
shared `pyinfra.runner.run` funnel: each command plans a deploy callable, then
executes its queued operations through `run_ops`. `runtime`, `services`, `ssl`,
and `helpers` are mutating flows; `manifest show` uses the same runner for
read-only fact inspection, and remote patches mutate only BonesDeploy marker
state.

`bonesremote` host doctor aggregates server baseline checks in
`commands/doctor/baseline.rs`, where package effects are represented by
root-owned artifacts rather than package database queries. Server doctor has no
etckeeper check.

## Intended behavior

The server package baseline installs `etckeeper` alongside the existing Git and
host packages. Server setup explicitly queues an idempotent `etckeeper init`
operation after package installation so `/etc` is initialized even when the
package's post-install initialization was skipped or setup is rerun.

Each mutating command appends the shared etckeeper final operation after its
existing provisioning operations. The operation inspects `/etc` for tracked,
deleted, or untracked changes and invokes `etckeeper commit` with a BonesInfra
provisioning message only when changes exist. A clean run succeeds without
creating a commit. Because it is an ordinary final PyInfra operation, any
earlier failure prevents it from running.

`manifest show` and both patch scopes do not append this operation. Host doctor
checks that the packaged `/usr/bin/etckeeper` executable exists as a secure
root-owned executable and reports a server-baseline issue when it does not.

## Approach

Create `services/linux/etckeeper.py` as the shared provisioning operation
boundary. It exposes initialization for server setup and a final commit
operation for mutating flows. The commit operation uses a checked-in native
shell asset for the change check and `etckeeper commit`, avoiding duplicated
inline shell logic and preserving real commit errors. Wire that operation as
the final call in each existing mutating orchestrator or framework deploy
composition, without changing the runner's post-run behavior.

Add `etckeeper` to `BASE_SYSTEM_PACKAGES`, call initialization in the server
orchestrator after package installation, and append the commit operation at the
end of server, site, services, SSL, helpers, and framework runtime plans.
Extend the existing host baseline doctor with the executable artifact check.
Add focused tests for operation ordering, clean/changed commit command
construction, and doctor artifact validation. Update the generated BonesInfra
wheel after Python changes.

## Responsibilities and boundaries

| Boundary | Responsibility |
| --- | --- |
| `services/linux/etckeeper.py` | Queue idempotent `/etc` initialization and final change recording operations. |
| `cli/commands/server/packages.py` | Declare etckeeper as a server baseline package. |
| Server/site/service/SSL/helper/runtime deploy plans | Place the final etckeeper operation after their own provisioning operations. |
| `assets/scripts/` | Store the native shell logic that checks `/etc` and invokes etckeeper. |
| `bonesremote commands/doctor/baseline.rs` | Verify the installed etckeeper executable as host-baseline evidence. |
| Python and Rust tests | Prove ordering, scope exclusions, clean-run behavior, and doctor reporting. |

## Affected areas

- `crates/bonesinfra/python/src/bonesinfra/services/linux/etckeeper.py`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/server/packages.py`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/server/__init__.py`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/site/__init__.py`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/site/services/__init__.py`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/site/ssl/__init__.py`
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/server/helpers/__init__.py`
- Framework runtime composition used by `runtime apply`
- A new shell asset under `crates/bonesinfra/python/src/bonesinfra/assets/scripts/`
- `crates/bonesremote/src/commands/doctor/baseline.rs`
- Focused BonesInfra and BonesRemote tests
- `crates/bonesinfra/assets/bonesinfra-*.whl`
- `CONTEXT.md`, `docs/security/invariants.md`, and relevant architecture or
  operator guidance

## Decisions

- Use one shared Linux service module rather than duplicating etckeeper shell
  calls in every flow; all mutating flows need the same final-step invariant.
- Keep the final step inside each PyInfra plan, not in `runner.run`, because the
  runner also serves read-only manifest inspection and patch flows and cannot
  express the requested per-flow operation ordering by itself.
- Use etckeeper's normal commit path with the package defaults. A preliminary
  `/etc` status check makes a no-change run successful while allowing actual
  etckeeper failures to fail the flow.
- Verify the installed executable in host doctor, matching the existing secure
  root-owned artifact checks and the explicit requirement that doctor checks
  etckeeper exists.
- Do not manage `.gitignore` content or custom configuration; etckeeper defaults
  remain the single configuration source.

## Risks

- A flow may accidentally append the commit before its true final operation;
  focused order tests must cover every mutating orchestrator.
- `/etc` contains secrets, so the repository and its history must remain root
  protected; the implementation must not add remote copies or broaden access.
- A malformed or missing `/etc` repository can make the final etckeeper step
  fail, correctly surfacing an unhealthy host rather than silently losing the
  audit record.
- Changes to the embedded Python source without wheel regeneration would leave
  the shipped binary running stale provisioning code.

## Validation

- Python tests assert etckeeper is in the server package set, initialization is
  ordered after package installation, and every mutating flow queues etckeeper
  last while manifest and patch flows do not.
- Python tests assert the native command checks clean and changed `/etc` states,
  and propagates commit failure.
- BonesRemote tests assert the baseline doctor reports a missing, wrong-kind, or
  insecure etckeeper executable and accepts a valid root-owned executable.
- Run targeted Python and Rust tests, regenerate and validate the embedded
  wheel, then run `cargo fmt`, `cargo clippy`, `shfmt -w .`, `ruff format .`, and
  `ruff check .`. Do not run E2E tests.
- Review the final diff and documentation for the default-only configuration
  policy and the final-step ordering.
