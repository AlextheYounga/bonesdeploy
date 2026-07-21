# BonesInfra Migration Scan — Required Rewrites

Based on reading the new security model (docs 01–03) and scanning every file in
`src/bonesinfra/`. This document maps the concrete rewrites needed in bonesinfra
to support the v1 target from `02_new_architecture_approach.md`:

> git = ingress only, foo = one per-site identity, releases/ = root:foo,
> shared/ = foo:foo, no permanent build/, Podman builds belong to bonesremote.

______________________________________________________________________

## 1. `domain/paths.py` — New constants, different repo default

### 1a. Repo location: `/home/git` → `/srv/git`

```
Line 4:  DEFAULT_REPO_PARENT = "/home/git"     → "/srv/git"
```

Affects `DeploymentPaths.new()` which defaults `repo_parent` from this constant.

### 1b. Add site registry constant

New constant needed:

```python
ETC_BONESDEPLOY_SITES = "/etc/bonesdeploy/sites"
```

And a `site_registry_path` field on `DeploymentPaths`. Bonesinfra may read this
for validation; `bonesremote init` is what writes it.

### 1c. Remove `BUILD_DIR` / `build_root` / `build_logs`

```
Lines 25-27: BUILD_DIR, WORKSPACE_DIR, LOGS_DIR constants
Lines 69-70:  build_root, build_logs fields on DeploymentPaths
Lines 127-128: path construction for build_root, build_logs
```

Builds are throwaway Podman containers owned by `bonesremote`. Bonesinfra should
not provision a permanent `build/` directory. Remove these path entries.

______________________________________________________________________

## 2. `domain/context.py` — Don't force deploy_user = "git"

### 2a. `DEPLOY_USER` constant

```
Line 10: DEPLOY_USER = "git"
```

Keep (git is still the global ingress user). But `deploy_user` is now only an
ingress identity — bonesinfra must stop using it for build/release/directory
ownership.

### 2b. `BonesConfig.deploy_user` forced to "git"

```
Line 55: deploy_user=DEPLOY_USER,
```

This bypasses whatever the caller might pass. OK for now since git is still the
only ingress user, but conceptually `deploy_user` should flow from config.
Low-priority to change for v1; just stop *using* it for site directory ownership.

______________________________________________________________________

## 3. `deploys/setup/users.py` — Critical identity changes

### 3a. Remove git from runtime_group (V2 fix)

```
Line 47: _ensure_group_membership(ctx.config.deploy_user, ctx.runtime.runtime_group)
```

**DELETE THIS LINE.** `git` must not be in any per-site group. This is the
highest-impact single-line change.

### 3b. Collapse the site identity to `runtime_group`

The v1 docs use only `runtime_user` and `runtime_group` for the site identity.
The release-read grouping concept becomes `runtime_group`:

```
Lines 43-45: server.group(name="Ensure release-read group exists", group=ctx.runtime.release_group)
Lines 59-65: groups=[ctx.runtime.runtime_group, ctx.runtime.release_group]
```

Simplifies to only `runtime_group`. Update any systemd templates that still
reference `SupplementaryGroups={{ release_group }}`.

### 3c. `git` identity harden (future)

```
Lines 29-32: shell="/bin/bash", ensure_home=True
```

02 says v1 defers git-shell. No change needed for v1, but the long-term direction
is `shell=/usr/bin/git-shell`.

______________________________________________________________________

## 4. `deploys/setup/directories.py` — Ownership model rewrite

This file needs the most changes. Every directory's owner:group:mode is wrong
for the new model.

### 4a. Bare repo path now `/srv/git/<project>.git`

```
Lines 17-20: mkdir path=paths["repo_parent"], user=deploy_user, group=deploy_user
Lines 23-27: git init --bare at paths["repo"]
```

Repo is still `git:git` (ingress-owned). The path changes because
`DeploymentPaths.repo_parent` defaults from `DEFAULT_REPO_PARENT` which becomes
`/srv/git`. No code change needed beyond §1a.

### 4b. project_root: `deploy_user:release_group 2751` → `root:<project> 2751`

```
Lines 44-48:
    mkdir(path=ctx.config.project_root, user=ctx.config.deploy_user,
          group=ctx.runtime.release_group, mode="2751")
```

Change to:

```
    mkdir(path=ctx.config.project_root, user="root",
          group=ctx.runtime.runtime_group, mode="2751")
```

User "root", group `<project>`. SetGID keeps new entries group-`<project>`.

### 4c. releases/: `deploy_user:release_group 2750` → `root:<project> 2750`

```
Lines 52-57:
    mkdir(path=paths["releases"], user=ctx.config.deploy_user,
          group=ctx.runtime.release_group, mode="2750")
```

Change to:

```
    mkdir(path=paths["releases"], user="root",
          group=ctx.runtime.runtime_group, mode="2750")
```

Releases are now root-owned, group-readable by foo. `bonesremote` promotes
artifacts here as root.

### 4d. Remove permanent build/ directory

```
Lines 60-63:
    mkdir(path=str(Path(ctx.config.project_root) / "build"),
          user=ctx.config.deploy_user, group=ctx.config.deploy_user, mode="0700")
```

**DELETE THIS BLOCK.** Builds are throwaway Podman containers owned by
bonesremote. No permanent `build/` directory under `project_root`.

### 4e. shared/: `runtime_user:runtime_group 2775` → `<project>:<project> 0750`

```
Lines 67-73:
    mkdir(path=paths["shared"], user=ctx.runtime.runtime_user,
          group=ctx.runtime.runtime_group, mode="2775")
```

Change to:

```
    mkdir(path=paths["shared"], user=ctx.runtime.runtime_user,
          group=ctx.runtime.runtime_group, mode="0750")
```

Drop setgid (2775 → 0750). `shared/` is runtime-exclusive; no group inheritance
needed. Remove the trailing 5 (world-readable) — shared has secrets like `.env`.

### 4f. Placeholder release: `deploy_user:release_group` → `root:<project>`

```
Lines 77-81:
    mkdir(path=paths["placeholder_web_root"], user=ctx.config.deploy_user,
          group=ctx.runtime.release_group, mode="0750")
```

Change to:

```
    mkdir(path=paths["placeholder_web_root"], user="root",
          group=ctx.runtime.runtime_group, mode="0750")
```

Matches the sealed release model: root-owned, group-readable by foo.

______________________________________________________________________

## 5. `deploys/setup/placeholder.py` — Ownership

### 5a. Placeholder HTML ownership

```
Lines 12-16:
    render(..., user=ctx.config.deploy_user, group=ctx.runtime.release_group, mode="0640")
```

Change to:

```
    render(..., user="root", group=ctx.runtime.runtime_group, mode="0640")
```

The `files.link` call (lines 20-24) creates `current` symlink pointing at the
placeholder release. This is fine — the symlink itself will later be flipped by
`bonesremote`. The initial symlink pointing at a root-owned placeholder is
correct.

______________________________________________________________________

## 6. `deploys/setup/plan.py` — No functional change

```
Line 12: from bonesinfra.deploys.setup import ...
```

No changes needed. The sub-modules change internally; the plan just calls them.

______________________________________________________________________

## 7. `deploys/runtime/doctor.py` — Who runs the doctor?

```
Lines 13-17:
    server.shell(commands=[_user_env_command(ctx.config.deploy_user,
                "/usr/local/bin/bonesremote doctor")],
                _sudo=True, _sudo_user=ctx.config.deploy_user)
```

Currently runs `bonesremote doctor` as `git`. Remove this step from
`bonesinfra`; it is not part of host provisioning.

______________________________________________________________________

## 8. Templates — `runtime_group` references

### 8a. `assets/nginx/site-nginx.service.j2:10`

```
SupplementaryGroups={{ release_group }}
```

Remove the extra group reference; the runtime user's primary group already
covers this.

### 8b. `runtimes/common/assets/app.service.j2:10`

```
SupplementaryGroups={{ release_group }}
```

Same as above.

______________________________________________________________________

## 9. `assets/apparmor/project-nginx-profile.j2` — Remove repo config access

### 9a. Remove `bones.toml` read grant (V3 fix)

```
Line 32: {{ paths.repo_bones_toml }} r,
```

**DELETE THIS LINE.** The per-site nginx profile currently grants read on the
repo's `bones.toml`. In the new model, privileged config comes from the
root-owned site registry (`/etc/bonesdeploy/sites/foo.toml`). The nginx runtime
has no reason to read git-owned config.

### 9b. `deploy_user` reference in comment

```
Line 46: # repo_path defaults to /home/{{ deploy_user }}/<project>.git...
```

Change comment to reflect `/srv/git/<project>.git`. The `deploy_user` in the
comment is cosmetic but misleading.

______________________________________________________________________

## 10. Runtime modules — `runtime_write_paths` to `shared/...`

### 10a. `runtimes/rails/rails.py:38-42`

```
runtime_write_paths = [
    f"{paths['current']}/tmp",
    f"{paths['current']}/log",
    f"{paths['current']}/storage",
]
```

Change to:

```
runtime_write_paths = [
    f"{paths['shared']}/tmp",
    f"{paths['shared']}/log",
    f"{paths['shared']}/storage",
]
```

These paths are then used in AppArmor writable grants and systemd
`ReadWritePaths`. The `shared/` versions must actually exist (created by
bonesinfra or bonesremote). Bonesinfra should create the baseline shared dirs.

### 10b. `runtimes/django/django.py:45-48`

```
writable = [static_root, media_root]
# where static_root = f"{paths['current']}/{ctx.runtime.runtime_data.get('static_root', 'staticfiles')}"
#       media_root  = f"{paths['current']}/{ctx.runtime.runtime_data.get('media_root', 'media')}"
```

Change to:

```
writable = [f"{paths['shared']}/staticfiles", f"{paths['shared']}/media"]
```

### 10c. `runtimes/nuxt/nuxt.py:34-39`

Creates static placeholder output dirs as `deploy_user:release_group`. Change to
`root:runtime_group`. This is the placeholder for static Nuxt; same reasoning as
placeholder.py.

Lines 31-39: `user=ctx.config.deploy_user, group=ctx.runtime.release_group`

______________________________________________________________________

## 11. New shared directory provisioning

Frameworks create their writable leaves and files under `shared/`; bonesinfra
only provisions the `shared/` parent and permissions.

### 11a. New file or addition to directories.py

```
# Laravel baseline shared paths:
shared/.env           → bonesremote writes this
shared/storage/        → create as foo:foo 0750
shared/bootstrap/cache/ → create as foo:foo 0750
shared/database/       → create as foo:foo 0750 (database.sqlite created later)
```

These directories must exist before `bonesremote` wires symlinks into releases.

### 11b. Django baseline

```
shared/staticfiles/    → create as foo:foo 0750
shared/media/          → create as foo:foo 0750
```

### 11c. Rails baseline

```
shared/tmp/            → create as foo:foo 0750
shared/log/            → create as foo:foo 0750
shared/storage/        → create as foo:foo 0750
```

______________________________________________________________________

## 12. `deploys/setup/bonesremote.py` — Sudoers shape

```
Lines 15-17:
    server.shell(commands=["/usr/local/bin/bonesremote init"], _sudo=True)
```

`bonesremote init` should now write the site registry at
`/etc/bonesdeploy/sites/<project>.toml`. The registry is a bonesremote concern,
not bonesinfra's. But bonesinfra may need to ensure the parent directory exists:

### 12a. Ensure registry parent exists

Add to directories.py or a new step:

```
mkdir("/etc/bonesdeploy/sites", user="root", group="root", mode="0750")
```

### 12b. Sudoers file shape

The thinking docs (03) say:

> Good: `git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote deploy --site <project>`
> Bad: `git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote * --config *`

`bonesremote init` owns the sudoers installation step.
The command shape should remain narrow and registry-backed.

Currently the sudoers path is computed (`paths.py:151`) as the single global
drop-in at `/etc/sudoers.d/bonesdeploy`; `bonesinfra` should not add
per-project files or aggregate-file management without a real lifecycle need.

______________________________________________________________________

## 13. `deploys/setup/packages.py` — No functional change

The package list includes `git`, `nginx`, `apparmor`, etc. No changes needed,
though `podman` belongs in the bonesinfra base package set because host
provisioning owns the dependency.

______________________________________________________________________

## 14. `runtimes/common/paths.py` — Ensure shared/ baseline dirs

Currently only creates `/run/<project>/` socket dirs. Frameworks own their
writable leaves under `shared/`; bonesinfra only needs the `shared/` parent and
its permissions.

______________________________________________________________________

## 15. Tests that need updating

### 15a. `tests/test_deploy_structure.py`

```
Lines 91-93: assert ctx.config.deploy_user is in ctx.runtime.runtime_group
```

This test assertion that `git` is in `runtime_group` MUST be removed or
inverted.

```
Lines 224,228-229: tests referencing runtime_user, runtime_group
```

### 15b. `tests/test_shared_paths.py`

```
Lines 33-35: assert deploy_user in runtime_group for shared path access
```

Same — this enforces the V2 violation. Must be updated.

### 15c. `tests/test_context.py`

```
Lines 27-29,33-34: Tests for runtime_user and runtime_group defaults
Lines 51-52,60-61: More runtime_user/runtime_group tests
```

Update defaults in tests to use only `runtime_user` and `runtime_group`.

### 15d. `tests/test_templates_j2.py`

```
Lines 34,110-120,172-176: Template tests with deploy_user, runtime_user, runtime_group
```

Update for new template variables and removed `repo_bones_toml` grant.

______________________________________________________________________

## 16. Summary: What changes, what doesn't

### Must change (v1 blockers):

| File | Change |
|------|--------|
| `domain/paths.py:4` | `DEFAULT_REPO_PARENT` → `/srv/git` |
| `domain/paths.py:25-27` | Remove `BUILD_DIR`/`WORKSPACE_DIR`/`LOGS_DIR` |
| `domain/paths.py:69-70,127-128` | Remove `build_root`/`build_logs` fields + construction |
| `domain/paths.py` | Add `ETC_BONESDEPLOY_SITES` + `site_registry_path` |
| `deploys/setup/users.py:47` | Delete `_ensure_group_membership(deploy_user, runtime_group)` |
| `deploys/setup/directories.py:44-48` | `project_root` owner → `root:<project>` |
| `deploys/setup/directories.py:52-57` | `releases/` owner → `root:<project>` |
| `deploys/setup/directories.py:60-63` | **Delete** permanent `build/` directory |
| `deploys/setup/directories.py:67-73` | `shared/` mode `2775` → `0750` |
| `deploys/setup/directories.py:77-81` | placeholder → `root:<project>` |
| `deploys/setup/placeholder.py:12-16` | render owner → `root:<project>` |
| `assets/apparmor/project-nginx-profile.j2:32` | Delete `repo_bones_toml r,` |
| `runtimes/rails/rails.py:38-42` | `runtime_write_paths` → `shared/...` |
| `runtimes/django/django.py:45-48` | `writable` → `shared/...` |
| `runtimes/nuxt/nuxt.py:31-39` | placeholder dirs → `root:runtime_group` |
| Tests: `test_deploy_structure.py`, `test_shared_paths.py` | Remove git-in-runtime-group assertions |

### Resolved:

| Topic | Decision |
|-------|----------|
| `release_group` | Use `runtime_group` only for v1. |
| `doctor.py` | Remove from `bonesinfra`. |
| `shared/` baseline dirs | Frameworks create writable leaves; bonesinfra provisions the parent. |
| Podman package | Install it in `bonesinfra`.

### Does not change (bonesinfra scope):

| What | Why |
|------|-----|
| Podman build execution | Belongs to bonesremote |
| Source export from repo | Belongs to bonesremote |
| Wire symlinks into releases | Belongs to bonesremote |
| Runtime prepare (migrations, optimize) | Belongs to bonesremote |
| Current symlink flip | Belongs to bonesremote |
| Service restart during deploy | Belongs to bonesremote |
| Rollback / pruning | Belongs to bonesremote |
| Secret push semantics | Belongs to bonesdeploy/bonesremote |
| Site registry content | Belongs to bonesremote (might just ensure parent dir) |
| Sudoers content | Belongs to bonesremote (bonesinfra may install the file) |

______________________________________________________________________

## 17. File count estimate

| Category | Files changed | Lines touched (approx) |
|----------|---------------|----------------------|
| Domain (paths + context) | 2 | ~15 lines |
| Setup deploys (users, dirs, placeholder, plan) | 4 | ~30 lines |
| Runtime deploys (doctor, nginx) | 1-2 | ~3 lines |
| Templates (j2) | 2-3 | ~5 lines |
| Runtime modules (rails, django, nuxt) | 3 | ~12 lines |
| App/cli/infra | 0 | 0 |
| Tests | 3 | ~15 lines |
| New shared dirs provisioning | 1 new or +existing | ~20 lines |
| **Total** | **~16 files** | **~100 lines** |

This is a small, focused change set. Most of the migration is in bonesremote,
not bonesinfra.
