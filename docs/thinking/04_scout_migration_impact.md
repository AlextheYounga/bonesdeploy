# Scout Report: Migration Impact Analysis

**Target model** (from `02_new_architecture_approach.md` + `03_bonesdeploy_bonesremote_concerns.md`):

```text
git = ingress only
foo = one per-site user/group, no login, no sudo
podman build = temporary, disposable build environment
shared/ = persistent runtime state owned by foo
releases/ = promoted artifacts sealed as root:foo
bonesremote/root = privileged mediator
```

---

## Files to DELETE

These are obsolete under the new model:

| File | Reason |
|------|--------|
| `crates/bonesremote/src/commands/post_receive.rs` | Replaced: `git checkout -f` into permanent `build/workspace` becomes disposable `git archive` export into temporary podman build context. |
| `crates/bonesremote/src/commands/stage_release.rs` | Replaced: creates release dir + ensures permanent `build/workspace`. New model: release dir creation is part of promotion step; no permanent build dir exists. |
| `crates/bonesremote/src/commands/wire_release.rs` | Replaced: only wires `.env`. New model must wire all shared paths (`storage/`, `bootstrap/cache/`, `database/database.sqlite`, etc.) into the **promoted release**, not the build workspace. Different target, different phase. |
| `crates/bonesremote/src/release/scripts.rs` | Replaced: runs deployment scripts via `bash` in build workspace as `git` user. New model: build runs in podman container; this module becomes podman invocation instead. |
| `crates/bonesdeploy/kit/deployment/01_install_build_deps.sh` | Replaced: installs nvm/npm into `$HOME` of deploy user. New model: build deps are in container image, not installed per-deploy. |
| `crates/bonesdeploy/kit/deployment/02_run_build.sh` | Replaced: runs `composer install`/`npm run build` in workspace as deploy user. New model: these run inside podman container. |

---

## Files requiring MAJOR REWRITE (>50% changed)

| File | Current behavior | Required changes |
|------|-----------------|------------------|
| **`crates/bonesremote/src/commands/deploy.rs`** (260 lines) | Orchestrates: doctor → stage → post_receive → wire → deploy_scripts → publish → activate → restart → prune. Everything runs as `git`. | New pipeline: validate registry → export source → podman build → promotion hardening → wire shared paths → runtime prepare (as `foo`) → activate (as root) → restart service → prune → cleanup. Must load root-owned registry. Must invoke podman. Must `sudo -u foo` for runtime prepare. See §Pipeline below. |
| **`crates/bonesremote/src/commands/init.rs`** (86 lines) | Installs sudoers drop-in: `git ALL=(root) NOPASSWD: bonesremote service restart --config *`. | Must write `/etc/bonesdeploy/sites/<project>.toml` registry. Must install multi-rule sudoers: `git ALL=(root) NOPASSWD: bonesremote deploy --registry /etc/bonesdeploy/sites/*`, `bonesremote service restart --registry ...`, etc. |
| **`crates/shared/src/paths.rs`** (285 lines) | `DEPLOY_USER = "git"`, `DEFAULT_REPO_PARENT = "/home/git"`, permanent `BUILD_DIR`/`WORKSPACE_DIR`/`LOGS_DIR` in `Deployment` struct. | Remove `DEPLOY_USER` as a deploy identity, but keep `/home/git` as the bare repo parent. Remove `build_root`/`build_logs` from `Deployment` struct (build is disposable). Add registry/control-plane path constants. |
| **`crates/shared/src/config.rs`** (220 lines) | `Bones` struct with 13 fields. `Runtime` struct with 4 fields. `deployment_paths()` method. | Add `SiteRegistry` struct (root-owned, canonical). Remove `default_deploy_user()`. Add `registry_path_for(project_name)`. `deployment_paths()` must drop build workspace fields. |
| **`crates/bonesremote/src/privileges.rs`** (10 lines) | Only `ensure_root()`. | Add `ensure_root_or_die()`, `drop_privileges_to(user)`, maybe `setuid` wrapper. The mediator pattern needs both root operations and `sudo -u foo` sub-operations. |
| **`crates/bonesremote/src/release_state.rs`** (288 lines) | Staged release tracking via `.staged_release` file in `repo/bones/`. Lists releases, flips `current` symlink. | Remove staged-release tracking via git-owned file. Release state tracking moves to registry or temp file. Keep `point_symlink_atomically()`, `list_releases_sorted()`, `current_release_name()`/`current_release_dir()`. |
| **`crates/bonesdeploy/kit/hooks/hooks.sh`** (123 lines) | Calls `bonesremote deploy --config "$BONES_TOML"` directly as `git`. | Must call `sudo bonesremote deploy --registry /etc/bonesdeploy/sites/$PROJECT.toml`. Becomes thin trigger only. No config parsing, no branch resolution — all that moves to bonesremote. |
| **`crates/bonesdeploy/src/commands/secrets.rs`** (327 lines) | Pushes `.env` to `shared/.env` with `chown root:<runtime_group>` where `runtime_group` comes from git-owned `runtime.toml`. | Must validate `runtime_group` against root-owned registry. The `chown root:<group>` and destination path must use registry values, not repo-owned config. |
| **`crates/bonesremote/src/commands/service.rs`** (46 lines) | Loads `project_name` from `--config` (git-owned bones.toml). Trusts it after regex validation. | Must load `project_name` and `service_name` from root-owned registry. `--config` replaced with `--registry`. |

---

## Files requiring MODERATE EDIT (new structs/fields, removed dependencies)

| File | Changes |
|------|---------|
| **`crates/bonesremote/src/commands/activate_release.rs`** (30 lines) | Currently runs as `git` (no root check). Must run as root to flip `current` symlink. Add `privileges::ensure_root()`. Otherwise similar — reads staged release name, calls `point_symlink_atomically()`. |
| **`crates/bonesremote/src/commands/drop_failed_release.rs`** (30 lines) | Currently reads staged release from `repo/bones/.staged_release`. Must track failed release via temp state or registry. Otherwise similar cleanup logic. |
| **`crates/bonesremote/src/commands/post_deploy.rs`** (137 lines) | Pruning logic is largely correct — reads `current` symlink, lists releases, prunes oldest non-active. No identity change needed, but `releases_keep` should come from registry not repo config. |
| **`crates/bonesremote/src/commands/rollback.rs`** (33 lines) | Currently runs as `git` (no root check). Must run as root. `releases_keep` / config from registry. |
| **`crates/bonesremote/src/commands/doctor.rs`** | Adds checks for: podman available, registry exists, AppArmor profiles. |
| **`crates/bonesremote/src/config.rs`** | Must add `load_registry()` alongside existing `load()`. Registry loading from `/etc/bonesdeploy/sites/<project>.toml`. |
| **`crates/bonesremote/src/cli/args.rs`** | Add `--registry` arg alongside `--config`. Deploy subcommand takes `--registry`, `--revision`. Service restart takes `--registry`. |
| **`crates/bonesdeploy/src/commands/init_project.rs`** | Keep `/home/git/<name>.git` repo defaults. Template defaults change for the new deployment/control-plane model. |
| **`crates/bonesdeploy/src/commands/push_state.rs`** | Stop pushing `.bones/` to `bones/` in the bare repo. Publish the deployment dataset to `bonesremote` control-plane state instead. |
| **`crates/bonesdeploy/src/commands/pull_state.rs`** | Same path update. |
| **`crates/bonesdeploy/src/commands/doctor.rs`** | Remote checks change: verify `bonesremote` + podman available, registry exists. No longer checks `build/workspace`. |
| **`crates/bonesdeploy/src/commands/remote_setup.rs`** | Passes JSON to `bonesinfra setup apply`. JSON must include new fields: registry path, site user (`foo`), no `DEPLOY_USER` as deploy identity. |
| **`crates/bonesdeploy/src/commands/remote_runtime.rs`** | Passes JSON to `bonesinfra runtime apply`. Must reflect new identity model. |
| **`crates/bonesdeploy/src/infra/bonesinfra.rs`** | JSON payload construction for `setup apply` and `runtime apply` must use registry-aware fields. |
| **`crates/bonesdeploy/src/config.rs`** | Local config loading. Defaults stay on `/home/git`; privileged remote values are constrained by bonesremote. |
| **`crates/bonesdeploy/kit/bones.toml`** | Default repo_path template. |
| **`crates/bonesdeploy/kit/hooks/post-receive`** | Must become thin trigger: resolve ref, call `sudo bonesremote deploy --registry ...`. |
| **`crates/bonesdeploy/kit/hooks/pre-push`** | Local pre-push hook. Unchanged (still calls `bonesdeploy doctor --local`). |

---

## Files requiring MINOR EDIT (path constants, field removals)

| File | Changes |
|------|---------|
| `crates/bonesdeploy/src/infra/git.rs` | `RemoteConnectionDetails` parsing should continue to handle `/home/git/...` URLs. |
| `crates/bonesdeploy/src/infra/ssh.rs` | SSH user changes: `bonesdeploy deploy` may SSH as root (to run `bonesremote deploy --registry ...`) rather than as `git`. |
| `crates/bonesdeploy/src/infra/bootstrap_ssh.rs` | Bootstrap SSH user selection. |
| `crates/bonesdeploy/src/main.rs` | Version bump, no structural changes. |
| `crates/bonesremote/src/main.rs` | Version bump, no structural changes. |
| `crates/bonesremote/src/cli/dispatch.rs` | Wire new subcommands (promotion, source-export, runtime-prepare). |
| `crates/bonesdeploy/src/cli/dispatch.rs` | Update SSH target user for `deploy`/`rollback` commands. |
| `crates/bonesdeploy/src/ui/prompts.rs` | Prompt text updates for new default paths. |
| `crates/bonesdeploy/kit/runtime.toml` | Shared paths section becomes more explicit / validated. |

---

## NEW files needed

| File | Purpose |
|------|---------|
| `crates/bonesremote/src/commands/promote.rs` | **Promotion hardening**: copies podman build output into release dir. Sets `root:foo` ownership, `0750` dirs, `0640` files. Rejects setuid/setgid, device files, FIFOs, sockets, unsafe symlinks. |
| `crates/bonesremote/src/commands/source_export.rs` | **Source export**: `git --git-dir=<repo> archive <rev> \| tar -x -C <tmp>` into disposable temp dir. Validates repo/rev against registry. |
| `crates/bonesremote/src/commands/runtime_prepare.rs` | **Runtime prepare**: runs as `foo` via `sudo -u foo`. Executes framework commands (e.g. `php artisan migrate --force`, `php artisan optimize`). Sees `.env`/SQLite/storage via shared symlinks. |
| `crates/bonesremote/src/commands/build.rs` | **Podman build**: creates/disposes container, mounts source+cache, runs build scripts in `/workspace/source`, returns the mutated source tree for promotion. |
| `crates/bonesremote/src/commands/wire_shared.rs` | **Shared path wiring** (replaces old `wire_release.rs`): symlinks all shared paths (`.env`, `storage/`, `bootstrap/cache/`, `database/database.sqlite`, etc.) from `shared/` into the promoted release directory. |
| `crates/shared/src/registry.rs` | **Site registry schema** (`SiteRegistry` struct): canonical `project_name`, `repo_path`, `project_root`, `runtime_user`, `runtime_group`, `shared_root`, `releases_root`, `service_name`, `framework`. Load from `/etc/bonesdeploy/sites/<project>.toml`. |
| `crates/shared/src/hardening.rs` | **Artifact hardening logic**: validate symlinks, reject dangerous file types, normalize modes. Shared between promote and any future artifact import. |

---

## The new deploy pipeline (replaces `deploy.rs:23-44`)

```text
validate_registry
  └─ resolve_revision
       └─ source_export          (git archive → temp dir)
            └─ podman_build      (container: source → artifact)
                 └─ promote       (artifact → releases/<id>, root:foo, hardened)
                      └─ wire_shared  (symlink shared/ paths into release)
                           └─ runtime_prepare  (sudo -u foo: migrate, optimize)
                                └─ activate     (root: flip current symlink)
                                     └─ restart_service  (root: systemctl restart)
                                          └─ post_deploy  (prune old releases)
                                               └─ cleanup_temp (rm temp build dir)
```

On failure: delete temp dir, delete staged release dir (if promoted), clear any state files.

---

## Key identity shifts by file

| Concern | Old identity | New identity |
|---------|-------------|--------------|
| `git checkout` into workspace | `git` user | `root` exports archive |
| Run build scripts | `git` user | Podman container |
| Copy build → release | `git` user (`cp -a`, preserves git ownership) | `root` promotes (hardens, sets `root:foo`) |
| Wire `.env` / shared paths | `git` user | `root` creates symlinks (targets `foo:foo` shared) |
| Runtime prepare (migrate) | N/A (didn't exist) | `foo` user |
| Flip `current` symlink | `git` user | `root` |
| Restart service | `root` via sudo (same) | `root` via sudo (same, but registry-backed) |
| Prune old releases | `git` user | `root` or `foo` (TBD) |
| Push secrets to `shared/.env` | `git` user over SSH | `root` over SSH, validated against registry |

---

## Summary counts

| Category | Count |
|----------|-------|
| Files to DELETE | 6 |
| Files to MAJOR REWRITE | 10 |
| Files to MODERATE EDIT | 17 |
| Files to MINOR EDIT | 13 |
| NEW files | 7 |
| **Total files touched** | **~53** (of ~70 source files) |

The highest-risk changes are:
1. `deploy.rs` — complete pipeline rewrite (260 lines → ~150, but entirely new logic)
2. `paths.rs` / `config.rs` — shared crate schema changes cascade to every consumer
3. `release_state.rs` — removing repo-owned staged state tracking
4. `hooks.sh` — changing the git hook trigger boundary

Files that do NOT change:
- `crates/bonesdeploy/src/commands/version.rs` / `config.rs` / `manage.rs` / `guide.rs` / `update.rs` / `status.rs`
- `crates/bonesremote/src/commands/version.rs` / `config.rs` / `status.rs`
- Most `ui/` helpers, `infra/embedded.rs`, `infra/rsync.rs`
- Test infrastructure patterns (but test data must update)
