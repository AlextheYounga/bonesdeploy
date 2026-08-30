# Git Coordinator Implementation Plan

This plan supersedes the temporary root-mode fallback recorded in
`03-tasks.md`. It restores and completes the original feature from
`02-plan.md`: deployments are coordinated by the unprivileged `git` identity,
and every root-required mutation crosses the sudo boundary through a narrow,
typed transition with an exact sudoers rule. It also deletes the unnecessary
first-pass code and the fallback code rather than layering compatibility.

## Target flow

```text
workstation
  -> SSH as git
  -> sudo -n /usr/local/bin/bonesremote config sync --site SITE --config-stdin
  -> /usr/local/bin/bonesremote deploy --site SITE   (no sudo)
       -> resolve revision, export source as git
       -> invoke typed root transitions through /usr/bin/sudo -n
       -> write coordination state as git
```

There is no root SSH fallback and no `sudo bonesremote deploy` grant.

## Design decisions

### Privileged API: five transactional transitions

The coordinator invokes dedicated deployment-transition leaves instead of
exposing existing general-purpose operator commands. Grouping by transaction
closes inter-command race and crash windows:

| Transition | Responsibility |
|---|---|
| `deploy begin --site --revision` | Validate site and full commit SHA, exclusively create the root-owned candidate, return the generated release ID |
| `deploy prepare --site --release` | Validate the git-exported context, hand it to the build identity, build, promote, wire shared data, run runtime preparation, seal, and preflight |
| `deploy commit --site --release` | Revalidate the sealed candidate, final nginx validation, activate, restart/verify services, restore and restart the previous release on failure |
| `deploy complete --site --release` | Remove the derived build context and prune using retention from the root-owned snapshot |
| `deploy abort --site --release` | Remove only the validated non-current candidate, derived context, and site-derived build container |

- No separate sudo commands for chown, promote, wire, restart, or prune.
- No paths, unit names, usernames, container names, shell commands,
  previous-release arguments, or prune counts are accepted from the caller.
- Revision is resolved by `git` in the coordinator; privileged code accepts
  only a full hexadecimal commit SHA.
- Rollback, backup, doctor, runtime management, manual prune, and service
  restart remain root/operator commands without git sudo grants.

### Path model

| Purpose | Path | Ownership/mode |
|---|---|---|
| Coordinator state | `/var/lib/bonesdeploy/state/<site>` | parent `root:git 0750`; site `git:git 0700`; `deployment.json` `git:git 0600` |
| Stable lock | `/var/lib/bonesdeploy/locks/.<site>.deployment.lock` | parent `root:git 0750`; file `root:git 0640` |
| Config snapshot | `/var/lib/bonesdeploy/config/<site>/bones.json` | site dir `root:git 0750`; file `root:git 0640` |
| Build context | `/srv/sites/<site>/tmp/build-<release>` | temp root `git:git 0700` |
| Secrets/logs | `/root/.config/bonesremote/sites/<site>` | `root:root 0700`; passphrase `root:root 0600` |
| Releases/current | Existing `/srv/sites/<site>` layout | root-controlled (unchanged) |

- Replace the ambiguous `bonesremote_sites_root()` with explicit state, lock,
  snapshot, and secret path functions.
- Borg passphrases and logs stay under the root-only secret path.
- Build-context paths become deterministic from site and release so the state
  record no longer stores an arbitrary context path.
- The commit helper derives the previous release from `current`; the state
  record no longer persists a previous-release path.

### Lock and state semantics

- Lock files are provisioned once by BonesInfra; `DeploymentLock::acquire`
  never creates them.
- The opener uses no-follow semantics, verifies a root-owned regular file, and
  takes `flock(LOCK_EX)` on a read-only descriptor. A missing or wrong lock
  fails with provisioning guidance.
- `atomic_write` sets `0600` explicitly instead of relying on umask.
- Git-written state is untrusted input to root helpers: revalidate site and
  release syntax, enforce legal phase transitions, verify path containment,
  symlink boundaries, and process identity before destructive operations.
- The chown traversal must stop following symlinks (current `chown` follows
  repository-controlled symlinks; use no-follow operations).

### Sudoers

- Grant only `config sync` and the five canonical transitions, each anchored:
  absolute `/usr/local/bin/bonesremote` path, `NOPASSWD`, canonical option
  order, anchored argument regexes (`^...$`), site `[a-z0-9-]+`, release
  matching the generated grammar (`[0-9]{8}_[0-9]{6}-[0-9a-f]{8}-[0-9a-f]{4}`),
  revision a full 40-hex SHA.
- Anchored sudo argument regexes require sudo `>= 1.9.10`. Establish this as a
  host prerequisite, verified during provisioning and doctor; do not silently
  install policy that passes only on the developer's distribution.
- The coordinator invokes transitions via `std::process::Command` with argv
  `/usr/bin/sudo -n /usr/local/bin/bonesremote deploy <transition> --site S
  --release R`. Never a shell, never PATH lookup at the privilege boundary.
- Prove denials: `deploy`, backup, runtime, doctor, status, extra/reordered/
  `--name=value`/repeated options, trailing arguments, shell wrappers, other
  executables.

### Migration

One explicit, versioned, root-run BonesInfra patch (not lazy runtime probing):

1. Validate site names; provision the new state, snapshot, and lock roots;
   create the lock inode conditionally.
2. Acquire legacy and new locks in a fixed order.
3. Refuse ambiguous destinations: if both old and new state exist and are not
   verified equivalent, stop; never overwrite either.
4. Move only `deployment.json`, `active-deployment.json`, `staged-release`,
   and `recovery/`. Never move `.borg_passphrase`, logs, or unknown files.
5. Copy to a temporary file, fsync, atomic rename, verify, then remove the
   source (the two roots may be on different filesystems).
6. Apply `git:git 0700` to site state directories, `0600` to state files.
7. Leave the old lock inode as an inert root-only artifact.
8. Write the patch marker only after verified success.

## Implementation steps

### 1. Normalize the current working tree

- Keep the single-descriptor contract: `bonesremote deploy` loads only the
  installed snapshot.
- `crates/bonesdeploy/src/commands/deploy.rs`: restore `ssh::connect(&cfg)`.
- `crates/bonesdeploy/src/infra/mod.rs`: restore
  `sudo -n /usr/local/bin/bonesremote config sync --site 'S' --config-stdin`
  for sync and plain `/usr/local/bin/bonesremote deploy --site 'S'` for deploy.
- `crates/bonesremote/src/commands/deploy/lifecycle.rs`: remove
  `ensure_root("bonesremote deploy")`.
- Replace the fallback test `deploy_requires_root_without_a_config_descriptor`
  with the unprivileged-coordinator contract.
- Do not revive first-pass environment path overrides
  (`BONESREMOTE_SITES_ROOT`) or incomplete sudoers rules.

### 2. Separate trust-domain paths

- `crates/bonesdeploy-core/src/paths.rs`: add explicit
  `bonesremote_state_root/site_state_root`, `bonesremote_lock_root/
  deployment_lock_path`, `bonesremote_snapshot_root/snapshot_path`,
  and secret-path accessors; delete the ambiguous generic root usage.
- `crates/bonesremote/src/control_plane.rs`: snapshot root becomes
  `/var/lib/bonesdeploy/config/<site>/bones.json`; `store()` installs
  `root:git 0640` into the provisioned `root:git 0750` directory.
- `crates/bonesremote/src/release/state/mod.rs`: state under
  `/var/lib/bonesdeploy/state/<site>`; lock at
  `/var/lib/bonesdeploy/locks/.<site>.deployment.lock`; harden `acquire`
  per the lock semantics above.
- `crates/bonesremote/src/release/state/atomic.rs`: explicit `0600`.
- Build context: deterministic
  `/srv/sites/<site>/tmp/build-<release>` created by `git`; `ensure_build_context`
  moves to the coordinator and no path is persisted in the record.

### 3. Harden root-side validation

- Site: `validate_site_name` plus provisioned-root checks in every helper.
- Release: exact generated grammar, one normal path component, no symlink,
  destination contained in `/srv/sites/<site>/releases`.
- Context: derived only from site/release; require expected prefix, no symlink
  components, expected owner/mode.
- Config sync: descriptor size limit, full schema validation (services,
  branch/ref syntax, relative normalized `web_root`, bounded `releases_keep`
  and timeout, reject privilege-affecting runtime settings).
- Fix recursive ownership changes to no-follow semantics.

### 4. Add the five privileged transitions

- New leaves in `crates/bonesremote/src/cli/args.rs` under `Deploy`
  (`begin`, `prepare`, `commit`, `complete`, `abort`), dispatched in
  `crates/bonesremote/src/cli/dispatch.rs`.
- Reuse existing functions inside each transaction:
  - `begin`: `stage.rs` exclusive creation + `set_staged_release`.
  - `prepare`: `checkout` context validation, `build_user` handoff,
    `build::run`, `promote`, `wire_shared`, `prepare`, `finalize`,
    `preflight::validate_ready` + `run_nginx_test`.
  - `commit`: revalidate seal, `activate`, `service::run_for_release`,
    restoration transaction (moved from `coordinator.rs`), staged cleanup.
  - `complete`: context removal + `prune` with retention from snapshot.
  - `abort`: consolidated kill/drop-failed behavior against validated paths.
- Each helper derives all identities and paths from the validated site and
  root-owned snapshot; nothing is accepted from argv beyond site/release/SHA.

### 5. Refactor the coordinator

- `crates/bonesremote/src/commands/deploy/coordinator.rs`: keep lock
  ownership, phase recording, error reporting, and transition invocation;
  replace direct privileged calls with the five transitions.
- Move activation restoration into `commit`; delete
  `finish_failed_activation` and `restore_previous_release` from the
  coordinator.
- Preserve `CleanupPending` semantics: `complete` failures are maintenance
  warnings after successful activation.
- Every pre-commit failure invokes `abort`.

### 6. Provision and migrate (BonesInfra)

- `crates/bonesinfra/python/src/bonesinfra/config/paths.py`: state, lock,
  snapshot roots.
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/server/__init__.py`:
  global root creation.
- `crates/bonesinfra/python/src/bonesinfra/cli/commands/site/directories.py`:
  per-site state (`git:git 0700`), snapshot dir (`root:git 0750`), lock file
  (conditional `root:git 0640`), build-context tmp root (`git:git 0700`).
- `crates/bonesinfra/python/src/bonesinfra/patches/`: one-way versioned
  migration per the Migration section (extend the remote patch framework to
  apply real migration logic).
- Sudo version prerequisite check in provisioning and doctor.

### 7. Install the exact sudoers policy

- `crates/bonesinfra/python/src/bonesinfra/assets/sudoers/bonesdeploy.j2`:
  six anchored rules (config sync + five transitions), no `deploy` grant.
- Rendering/validation stays in
  `crates/bonesinfra/python/src/bonesinfra/cli/commands/server/sudoers.py`
  with `visudo -c -f`.

### 8. Doctor

- `crates/bonesremote/src/commands/doctor/`: validate each trust domain
  separately — secrets root-only and git-inaccessible; snapshots root-owned,
  git-readable, git-non-writable; lock parent not git-writable, lock inode
  root-owned regular file; state git-writable, inaccessible to runtime/build
  identities; releases/current root-controlled. Keep site discovery on the
  root-owned provisioned-site root, not mutable state. Give the coordinator
  state tree its own evaluator rather than folding it into the generic
  privileged-path list.

### 9. Tests

Rust:

- Command construction (`crates/bonesdeploy/src/infra/mod.rs` tests):
  SSH identity `git`, sync through sudo, deploy without sudo/descriptor.
- Transition argv builders: program `/usr/bin/sudo`, exact canonical argv.
- CLI contract (`crates/bonesremote/tests/cli.rs`): canonical forms accepted;
  malformed sites/releases/revisions rejected at parse and runtime.
- Lifecycle: transition validation, symlink rejection, sealing, activation
  restoration, abort safety, prune exclusions.
- Lock: missing, replaced, symlinked, wrong-ownership lock files fail.
- State migration: collision refusal, secret preservation, artifact fidelity.

Python:

- Sudoers template allow/deny rendering tests (exact rules, no `deploy`, no
  wildcards, no `ALL`).
- Provisioning expectations in `test_setup_directories.py`, `test_paths.py`.
- Migration patch tests in `test_patches.py`.

CI (Linux only — bonesremote does not compile on macOS):

- Real sudo-policy matrix on a supported distribution with sudo `>= 1.9.10`:
  allowed commands execute; all denial examples fail before execution.

### 10. Documentation and cleanup

- Update `CONTEXT.md`, `docs/ARCHITECTURE.md`,
  `docs/architecture/reference.md`, `docs/security/invariants.md`,
  `crates/bonesinfra/python/CONTEXT.md`, `README.md`, and rewrite
  `03-tasks.md` to describe this completed state.
- Fix `docs/security/invariants.md` to keep arbitrary git revisions out of
  privileged arguments (resolved 40-hex SHA only).
- Delete dead code exposed by the change: root checks on git-owned functions,
  the coordinator's in-process privileged calls, fallback tests, first-pass
  leftovers.
- Treat `docs/session.md` as archival, not a source of truth.

## Execution order

Steps 1-2 first (paths and entrypoint), then 3-5 (transitions and
coordinator), then 6-7 (provisioning, migration, sudoers), then 8-10. Steps 1
and 2 can land together; do not deploy between steps until sudoers and
provisioning exist, since the coordinator becomes non-operational the moment
root gates are removed without transitions.

## Validation

- `cargo test --workspace --exclude e2e` on Linux; focused tests for
  bonesdeploy/bonesdeploy-core on macOS.
- `cargo clippy`, `cargo fmt`, `shfmt -w .`.
- `ruff check .`, `ruff format .`, `uv run --frozen pytest -q`.
- Regenerate the embedded wheel after Python changes stabilize.
- No local E2E runs; bonesremote and sudo authorization validated on Linux CI.
