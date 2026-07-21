# BonesInfra Migration Plan

This plan turns the findings in `01_security_architecture_problems.md` through
`04_bonesinfra_migration_scan.md` into a sequenced migration path for
`bonesinfra`.

Decisions are fixed in `06_bonesinfra_migration_decisions.md`.

The v1 target is the simpler model from `02_new_architecture_approach.md` and
`03_bonesinfra_security_responsibilities.md`:

```text
git = global ingress only
<project> = one per-site runtime identity, no login, no sudo
podman build = disposable build environment owned by bonesremote
shared/ = persistent runtime state owned by <project>:<project>
releases/ = sealed promoted artifacts owned by root:<project>
bonesremote/root = privileged deploy mediator
```

The earlier multi-identity model in `01_security_architecture_problems.md` is
useful background, but it is not the v1 implementation target.

______________________________________________________________________

## Migration Goals

1. Make `git` ingress-only.
2. Make the per-site runtime user the only site identity.
3. Remove repo-owned config from privileged trust paths.
4. Stop provisioning permanent build state under `/srv/sites/<project>`.
5. Provision filesystem ownership so `bonesremote` can promote sealed releases.
6. Move runtime-writable paths to `shared/` and away from release code.
7. Keep deploy-time lifecycle behavior out of `bonesinfra`.

______________________________________________________________________

## Non-Goals For BonesInfra V1

These belong to `bonesremote`, `bonesdeploy`, or a later hardening pass:

- Running Podman builds.
- Exporting source from bare repositories.
- Promoting build artifacts into releases.
- Normalizing artifact ownership and modes after builds.
- Wiring shared symlinks into each release.
- Running migrations, cache warm-up, or framework prepare commands.
- Flipping `current` after real deployments start.
- Restarting services during deploy.
- Rollback and pruning.
- Secret push semantics.

______________________________________________________________________

## Phase 0: Resolved Contract

These are fixed for v1:

- `runtime_user` and `runtime_group` are the only per-site identity fields.
- `bonesinfra` provisions the `shared/` parent and host permissions.
- Frameworks create their own writable leaves and files under `shared/`.
- `runtime doctor` is not part of `bonesinfra`.
- Podman is required and is installed by `bonesinfra`.

______________________________________________________________________

## Phase 1: Update Domain Paths

Purpose: make path derivation match the new host layout.

Changes:

- Change the default repo parent from `/home/git` to `/srv/git`.
- Add a constant for `/etc/bonesdeploy/sites`.
- Add `site_registry_path` to `DeploymentPaths` so app/deploy code can validate
  or reference the trusted registry location.
- Remove permanent build path constants and fields from `DeploymentPaths`:
  `BUILD_DIR`, `WORKSPACE_DIR`, `LOGS_DIR`, `build_root`, and `build_logs`.

Expected result:

```text
/srv/git/<project>.git
/srv/sites/<project>/shared
/srv/sites/<project>/releases
/srv/sites/<project>/current
/etc/bonesdeploy/sites/<project>.toml
```

Do not add new build-user concepts in this phase.

______________________________________________________________________

## Phase 2: Fix Identity Provisioning

Purpose: remove the cross-site trust break caused by `git` being a site group
member.

Changes:

- Keep creating the global `git` ingress user.
- Keep creating one runtime user/group per site.
- Delete the membership that adds `git` to the runtime group.
- Ensure the runtime user has no login shell and no sudo grant.

Target shape:

```text
git:git
  owns /srv/git/<project>.git
  not a member of <project>

<project>:<project>
  owns shared runtime state
  reads sealed releases through root:<project> group ownership
  does not own ordinary release code
```

Deferred hardening:

- Change `git` from `/bin/bash` to `git-shell` after the deploy hook contract is
  registry-backed and tested.

______________________________________________________________________

## Phase 3: Rewrite Setup Directory Ownership

Purpose: make provisioned directories safe before `bonesremote` starts handling
real releases.

Changes:

- Ensure `/srv/git` exists as `git:git` with non-world-writable permissions.
- Create `/srv/sites/<project>` as `root:root`.
- Create `releases/` as `root:<project>` with group-readable, non-runtime-writable
  permissions.
- Delete provisioning of permanent `/srv/sites/<project>/build`.
- Create `shared/` as `<project>:<project>` with mode `0750`.
- Create only the `shared/` parent and permissions; frameworks create their own
  writable leaves and files under that parent.
- Create placeholder release content as `root:root`, not `git:*`.
- Ensure `/etc/bonesdeploy/sites` exists as `root:root` before registry-backed
  commands become mandatory.

Target ownership baseline:

```text
/srv/git/<project>.git              git:git
/srv/sites/<project>                root:root
/srv/sites/<project>/releases       root:<project>
/srv/sites/<project>/shared         <project>:<project>
/srv/sites/<project>/current        root:root symlink
/etc/bonesdeploy/sites              root:root
```

The placeholder release is only for first-time service validation. Future release
promotion belongs to `bonesremote`.

______________________________________________________________________

## Phase 4: Provision Shared Runtime State

Purpose: make runtime-writable state explicit and durable.

`bonesinfra` provisions the `shared/` parent and its permissions.

Frameworks create their own writable leaves and files under `shared/`.

Rules:

- `shared/` is owned by `<project>:<project>`.
- Directory mode should start at `0750`.
- Do not create or populate `.env` from `bonesinfra`; secret material belongs to
  the deploy/remote secret path.
- Do not wire release symlinks here; that is per-release behavior and belongs to
  `bonesremote`.

______________________________________________________________________

## Phase 5: Update Runtime Write Policies

Purpose: make service sandboxing match the sealed-release model.

Changes:

- Rails writable paths should point at `shared/tmp`, `shared/log`, and
  `shared/storage`, not `current/tmp`, `current/log`, or `current/storage`.
- Django writable paths should point at `shared/staticfiles` and `shared/media`,
  not `current/staticfiles` or `current/media`.
- Static runtime placeholder directories, including Nuxt placeholder output, must
  use `root:<project>` ownership instead of git-owned or runtime-group-writable
  placeholders.
- Systemd `ReadWritePaths` and AppArmor writable grants should only include
  shared runtime state.

Invariant after this phase:

```text
runtime can write shared/
runtime can read current release code
runtime cannot write ordinary release files
```

______________________________________________________________________

## Phase 6: Remove Repo Config From Runtime Policy

Purpose: stop runtime services from reading git-owned config files.

Changes:

- Remove the AppArmor grant that allows reading repo-owned `bones.toml`.
- Update comments and templates that still assume repo paths live below
  `/home/git`.
- Ensure nginx, php-fpm, and app services use rendered config and the active
  release path, not git-owned project config.

This does not by itself implement the root-owned registry contract. It prepares
`bonesinfra` so that registry-backed `bonesremote` commands can become the only
privileged source of truth.

______________________________________________________________________

## Phase 7: Align Sudoers And Registry Hooks

Purpose: ensure the `bonesinfra` sudoers setup stays narrow and
registry-backed.

Changes:

- Keep the single historical sudoers drop-in at `/etc/sudoers.d/bonesdeploy`.
- Install sudoers during `bonesinfra` host provisioning.
- Keep the allowed commands narrow and registry-backed/site-literal.
- Do not install rules shaped like `bonesremote * --config *`.

Good shape:

```text
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote deploy --site <project>
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote service restart --site <project>
```

Bad shape:

```text
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote * --config *
```

`bonesremote` defines the command contract. `bonesinfra` installs the single
validated drop-in and should not add per-project sudoers files or
aggregate-file management without a real lifecycle requirement.

______________________________________________________________________

## Phase 8: Update Tests

Purpose: prevent the repo from drifting back to the current unsafe model.

Required test changes:

- Invert any assertion that `git` is a member of the runtime group.
- Assert `git` is not granted site group access.
- Assert `shared/` is `0750` and runtime-owned.
- Assert `releases/` and placeholder content are `root:<project>`.
- Assert no permanent `build/` path is derived or provisioned.
- Assert the default repo parent is `/srv/git`.
- Assert runtime writable paths point at `shared/`.
- Assert AppArmor templates do not grant runtime access to repo-owned
  `bones.toml`.
- Keep boundary tests from `PROJECT.md`: CLI/domain/app do not import pyinfra in
  the wrong layers, runtime registry imports are explicit, and no `sys.path`
  mutation returns.

Useful new regression tests:

```text
git_not_in_runtime_group
shared_is_runtime_owned_private_state
releases_are_root_owned_group_readable
deployment_paths_do_not_include_build_root
runtime_write_paths_are_shared_paths
apparmor_does_not_read_repo_config
```

______________________________________________________________________

## Phase 9: Update Documentation

Purpose: make docs match the actual model once code changes land.

Update:

- `docs/PROJECT.md`
- `docs/security/02-filesystem-layout.md`
- `docs/security/03-identity-and-user-policy.md`
- `docs/security/22-desired-end-state-summary.md`

Documentation must say:

- BonesInfra is the hidden provisioning engine, not public UX.
- `git` is ingress only.
- One per-site runtime identity owns `shared/`.
- Releases are sealed as `root:<project>`.
- Builds are disposable Podman work owned by `bonesremote`.
- Runtime services never need to read git-owned `bones.toml`.
- Shared symlink wiring, runtime prepare, activation, restart, rollback, and
  pruning belong to `bonesremote`.

Documentation must not say:

- Users should run `bonesinfra` directly.
- `git` owns releases or shared state.
- `shared/` is group-writable by `git`.
- Runtime writes inside ordinary release code.
- Activation hardening exists in `bonesinfra`.

______________________________________________________________________

## Safe Implementation Order

This order minimizes half-migrated states.

1. Add tests for the new invariants that can be expressed without changing code.
2. Change path constants and remove permanent build paths.
3. Remove `git` from runtime group provisioning.
4. Rewrite setup directory ownership and modes.
5. Add shared runtime directory provisioning.
6. Move runtime writable paths to `shared/`.
7. Remove repo config grants from AppArmor templates.
8. Align sudoers installation with the registry-backed command contract.
9. Update docs to match the implemented behavior.
10. Run `ruff check .`, `ruff format .`, and `uv run pytest`.

If `bonesremote` is not ready for root-owned release promotion yet, stop before
deploying this to production hosts. The `bonesinfra` side can be implemented and
tested in isolation, but production deployment needs the matching
`bonesremote` lifecycle changes.

______________________________________________________________________

## Cross-Repo Dependencies

`bonesinfra` can prepare the host, but the migration is not complete until
`bonesremote` and `bonesdeploy` stop trusting git-owned config for privileged
actions.

Required `bonesremote` work:

- Write `/etc/bonesdeploy/sites/<project>.toml` during init.
- Read the registry for privileged deploy, activation, restart, rollback, and
  pruning commands.
- Export source from `/srv/git/<project>.git` into a disposable build context.
- Run Podman builds without mounting `shared/`, `.env`, `current`, `releases/`,
  `/srv/git`, or `/etc/bonesdeploy`.
- Promote artifacts as root into `releases/<release_id>`.
- Reject unsafe artifact entries before promotion.
- Normalize release ownership to `root:<project>`.
- Wire shared symlinks before runtime prepare.
- Run runtime prepare as `<project>`.
- Flip `current` as root only after prepare succeeds.
- Restart services through registry-backed commands.

Required `bonesdeploy` work:

- Stop deriving privileged secret ownership from repo-owned `runtime.toml`.
- Push secrets using the registry-backed runtime identity.
- Update any local assumptions about repo paths moving from `/home/git` to
  `/srv/git`.
- Keep public UX in `bonesdeploy`; do not expose `bonesinfra` as user-facing API.

______________________________________________________________________

## Rollout Strategy

For existing hosts, use a deliberate migration rather than relying on idempotent
setup alone.

Host migration outline:

1. Stop deploy hooks or put the site into maintenance mode.
2. Install updated `bonesremote` that supports the root-owned registry and sealed
   promotion path.
3. Create `/etc/bonesdeploy/sites/<project>.toml` from trusted operator input,
   not from repo-owned config alone.
4. Move bare repos from `/home/git/<project>.git` to `/srv/git/<project>.git`, or
   keep a compatibility symlink only for a short transition window.
5. Remove `git` from all per-site runtime groups.
6. Change `shared/` to `<project>:<project> 0750` after confirming runtime secret
   and writable paths are present.
7. Convert existing release trees to `root:<project>` and remove runtime write
   access from ordinary code paths.
8. Wire mutable paths into `shared/` using the same logic `bonesremote` will use
   for new releases.
9. Update hooks to call registry-backed `bonesremote` commands.
10. Run provisioning and deploy smoke checks.

Rollback plan for the migration itself:

- Keep a backup of the old bare repo location before moving it.
- Keep the previous `current` target recorded before changing ownership or
  symlinks.
- Do not delete old releases during the ownership migration.
- If deploy hooks fail after migration, disable hooks and invoke the new
  registry-backed `bonesremote` command manually for diagnosis.

______________________________________________________________________

## Acceptance Criteria

The migration is complete for `bonesinfra` when all of these are true:

- `git` owns ingress repos only and is not in per-site runtime groups.
- Default bare repo paths are under `/srv/git`.
- No permanent `/srv/sites/<project>/build` directory is provisioned.
- `shared/` is runtime-owned and private to the site identity.
- `releases/` and placeholder release content are root-owned.
- Runtime write paths point only at `shared/`.
- AppArmor and service templates do not require runtime access to git-owned
  config.
- `/etc/bonesdeploy/sites` exists for the trusted registry.
- Sudoers rules remain narrow and are installed by `bonesinfra`.
- Tests encode the new ownership and boundary rules.
- Documentation describes `bonesinfra` as provisioning only, not deployment
  lifecycle ownership.

______________________________________________________________________

## Deferred Hardening

After v1 is working, revisit these items deliberately:

- Change `git` to `git-shell`.
- Add per-site ingress users if one global `git` is no longer acceptable.
- Add per-site build users if disposable Podman builds need host-level identity
  separation.
- Add AppArmor or bubblewrap-style confinement for build containers beyond
  Podman defaults.
- Reduce the `www-data` cross-site socket/traversal ceiling.
- Add migration-aware rollback for database changes, if the product chooses to
  support it.
