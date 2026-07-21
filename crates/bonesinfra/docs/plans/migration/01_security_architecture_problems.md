Compression is rejecting ID order. I'll proceed directly to the synthesis — context is still manageable.

______________________________________________________________________

# Synthesis: BonesDeploy Ownership Model Redesign

I audited both repos end-to-end. Here's the grounded picture, the contradictions, and a proposed target model with explicit tradeoffs. **No final decisions baked in** — I flag every fork where I need your call.

______________________________________________________________________

## 1. What the code actually does today (the one-paragraph version)

There is **no `foo-build` user anywhere**. The global `git` user is the single deploy-side identity for every site: it owns the bare repo, owns `/srv/sites/<project>` and `releases/` and `build/`, runs `git checkout` into `build/workspace`, runs composer/npm/vite as itself with its own HOME as the cache root, runs `cp -a` into `releases/<id>` (preserving `git` ownership), flips `current`, prunes old releases, and drops failed ones. The runtime user (`<project>`, a system user with `nologin`) is enrolled in both `runtime_group` and `release_group`; `git` is **also** enrolled in `runtime_group` (`users.py:47`). `shared/` is `runtime_user:runtime_group` mode **2775** (group-writable + world-readable). The only sudoers grant is `git ALL=(root) NOPASSWD: <bonesremote> service restart --config *` (`init.rs:20-29`), and that command trusts git-supplied `project_name` with only a regex check. There is **no root-owned site registry** — `bones.toml`/`runtime.toml` live inside the git-owned bare repo and are trusted for privileged decisions (AppArmor grants read on `repo_bones_toml` at `project-nginx-profile.j2:32`; `service.rs` trusts `project_name` for `systemctl restart <x>-nginx`).

______________________________________________________________________

## 2. Violations against your desired model (ranked by severity)

### Severity: Critical (trust boundary / cross-site)

**V1. No `foo-build` user exists.** Build/release ownership is collapsed into the global `git`. One compromised `git` = every site's release code across the host.

- `crates/shared/src/paths.rs:11` `DEPLOY_USER = "git"`; `crates/shared/src/config.rs:91-103` has `runtime_user_for`/`runtime_group_for`/`release_group_for` but **no `build_user_for`**.
- `bonesinfra/.../setup/users.py:26-33` creates only `git` + runtime user.
- `bonesinfra/.../setup/directories.py:43-65` provisions `project_root`, `releases/`, `build/` all as `git:`.

**V2. `git` is in every site's `runtime_group`.** `users.py:47`. Combined with `shared/` mode 2775 (`directories.py:67-73`), `git` can read/write runtime mutable state (`.env`, sqlite, uploads) on every site. Tested-as-intended at `tests/test_deploy_structure.py:91-93` and `tests/test_shared_paths.py:33-35`.

**V3. No root-owned site registry; privileged code trusts git-owned config.**

- `bonesinfra/.../apparmor/project-nginx-profile.j2:32` grants runtime nginx read on `/home/git/<project>.git/bones/bones.toml`.
- `bonesinfra/.../domain/context.py:22-66` `DeployContext.from_files()` reads `project_name`/`repo_path`/`project_root`/`runtime_user`/`runtime_group`/`release_group` from git-owned TOML and those flow into root-owned systemd units, AppArmor profiles, nginx configs.
- `crates/bonesremote/src/commands/service.rs:12-26` trusts `project_name` from `--config *` (regex-only validation) → compromised `git` can restart any `<x>-nginx.service` on the host.
- `crates/bonesdeploy/src/commands/secrets.rs:120-143` trusts `runtime_group` from git-owned `runtime.toml` for `chown root:<group>` on secret files.

**V4. `runtime_group` from git-owned `runtime.toml` is trusted for `chown root:<group>`** in secrets push. A compromised `git` could set `runtime_group = "sudo"` and make secret files readable by a privileged group. (`secrets.rs:141-143`, `config.rs:151-164`.)

### Severity: High (release code ownership)

**V5. Entire deploy pipeline runs as `git`.** No `sudo -u foo-build` anywhere. `kit/hooks/hooks.sh:81,86` invokes `bonesremote deploy` directly; `deploy.rs:23-45` runs doctor→stage→post_receive→wire→publish→activate→restart→post_deploy all as `git`.

- `stage_release.rs:32-33` creates `releases/<id>` as `git`.
- `post_receive.rs:19-30` runs `git checkout` as `git` into `git:git 0700` build/workspace.
- `deploy.rs:74-85` runs build scripts (composer/npm/vite) as `git` with `git`'s HOME.
- `deploy.rs:116-134` `publish_release_tree` uses `cp -a`, preserving `git` ownership into the release tree.

**V6. Runtime user writes into release tree (not shared).**

- Rails (`bonesinfra/.../runtimes/rails/rails.py:38-42`): `runtime_write_paths = [current/tmp, current/log, current/storage]` → systemd `ReadWritePaths` + AppArmor `rwk` grant inside the release tree.
- Django (`django.py:45-47`): `current/staticfiles`, `current/media`.
- Laravel `runtime.toml`: `bootstrap/cache`, `database/database.sqlite` mode 770/660 inside the release tree (only `storage` and `.env` are shared via `[shared]`).
  This contradicts your invariant "foo-run should not own release code... runtime compromise not to persist by rewriting code."

### Severity: Medium (docs/functional)

**V7. `wire_release` wires ONLY `.env`.** `wire_release.rs:18-48`. The test `storage_is_not_symlinked` (`wire_release.rs:240-263`) explicitly asserts `storage` is NOT symlinked. The `[shared] paths` block in Laravel's `runtime.toml` declares `storage` as shared, but the wire code doesn't implement it. → Laravel apps cannot write to `storage/` at runtime; either they crash or an operator hand-fixes permissions (exactly the recursive chown you want to eliminate).

**V8. `wire_release` `.env.example` fallback creates `shared/.env` as `git:git 0600`** → runtime_user cannot read it. (`wire_release.rs:27-34`.) Only the `secrets push` path does the correct `chown root:<runtime_group> 0640`.

**V9. Docs contradict code on modes and "activation hardening".**

- `docs/security/02-filesystem-layout.md:70` claims releases become "service-owned after activation hardening" — **no such step exists** (`activate_release.rs:9-30` only flips the symlink).
- `:84-90` claims modes `0750/0711`; actual modes are `2751/0700/2750/2775`.
- `:129-132` claims `storage` is wired as a shared symlink — it is not.
- `docs/PROJECT.md:52` claims `shared/` is `0711`; actual is `2775`.
- `docs/security/03-identity-and-user-policy.md:6` and `22-desired-end-state-summary.md:10` describe a 2-identity model (deploy + service), not your desired 3-identity (git + build + run).

**V10. `git` has `/bin/bash` + HOME + SSH authorized_keys** (`users.py:30-31, 74-90`). Desired: ingress-only (git-shell or nologin, no interactive shell).

**V11. Bare repo lives in `git`'s HOME** (`paths.py:4` `DEFAULT_REPO_PARENT = "/home/git"`). `git`'s HOME is simultaneously: bare-repo parent, build-tool cache root (`$HOME/.nvm`, `$HOME/.npm`), and SSH authorized_keys location.

**V12. Build tool caches are global, not per-site.** `01_install_build_deps.sh`/`02_run_build.sh` (quoted at `docs/references/bonesdeploy.md:793-853`) use `NVM_DIR="${NVM_DIR:-$HOME/.nvm}"`, `npm install -g`, `corepack enable` — all writing into `/home/git/.nvm`, `/home/git/.npm`, shared across every site.

**V13. `www-data` is a de facto global cross-site read key.** Every `/run/<project>/` and `/run/<project>/nginx/` is `0711` so system nginx can traverse; per-site nginx socket is `0666`-ish (`site-nginx.conf.j2:21-24` ponytail comment acknowledges this). Accepted for single-tenant hosts but violates multi-site isolation.

______________________________________________________________________

## 3. Proposed target ownership model

### Roles (per site `foo`)

| Role | Identity | Created by | Shell | HOME | sudo | Groups |
|---|---|---|---|---|---|---|
| Ingress | `git` (global, one per host) | bonesinfra | `git-shell` | `/home/git` (or `/srv/git` — see Q5) | none | none per-site |
| Build | `foo-build` (per-site) | bonesinfra | `/usr/sbin/nologin` | `/var/lib/bonesdeploy/build/foo` | none | `foo-release` |
| Runtime | `foo-run` (per-site, system user) | bonesinfra | `/usr/sbin/nologin` | `/nonexistent` | none | `foo-run` (primary), `foo-release` (supplementary, read) |
| Mediator | `root` / `bonesremote` | — | — | — | — | — |

### Directory ownership target

| Path | owner:group:mode | Notes |
|---|---|---|
| `/home/git` (or `/srv/git`) | `git:git 0750` | Ingress home; no build caches here |
| `/home/git/foo.git` (bare repo) | `git:git` | Ingress repo; git-shell push only |
| `/srv/sites` | `root:root 0711` | Traverse-only (unchanged) |
| `/srv/sites/foo` | `foo-build:foo-release 2751` | Setgid so children inherit `foo-release` |
| `/srv/sites/foo/releases` | `foo-build:foo-release 2750` | Setgid; `foo-build` owns entries, `foo-run` reads via group |
| `/srv/sites/foo/build` | `foo-build:foo-build 0700` | Private build workspace |
| `/srv/sites/foo/shared` | `foo-run:foo-run 0750` | **Runtime-exclusive; `git` NOT in group** |
| `/srv/sites/foo/current` | `root:root` (symlink) | Flipped only by root/bonesremote |
| `/run/foo` | `foo-run:foo-run 0711` | Unchanged (www-data traverse) |
| `/var/lib/bonesdeploy/build/foo` | `foo-build:foo-build 0750` | HOME + composer/npm/cargo caches |
| `/etc/bonesdeploy/sites/foo.toml` | `root:root 0640` | **NEW: root-owned site registry** |
| `/srv/conf/foo` | `root:foo-run 0750` | nginx config, runtime.env |

### Trust boundary

- **Git-owned** (untrusted for privileged decisions): bare repo, `bones/bones.toml`, `runtime.toml` inside the repo. These drive **non-privileged deploy orchestration only** (stage, checkout, build scripts, wire, publish, prune).
- **Root-owned** (trusted for privileged decisions): `/etc/bonesdeploy/sites/foo.toml` — canonical `project_name`/`repo_path`/`project_root`/`build_user`/`runtime_user`/`runtime_group`/`release_group`/`service_name`. Written once by `bonesremote init` (root); `bonesinfra` reads it and refuses to provision if the git-owned `bones.toml` values conflict.
- **Privileged operations** (`service restart`, `current` flip if we move it, any future `release publish` if we go that route) read identity from the registry, not from `--config`.

### Runtime-writable paths (the shared-symlink model)

Every runtime-mutable path lives under `shared/` and is symlinked into the release at wire time. `runtime_write_paths` in systemd/AppArmor reference **`shared/...` only, never `current/...`**.

| Framework | shared paths | release symlink targets |
|---|---|---|
| Laravel | `.env`, `storage/`, `bootstrap/cache/`, `database/database.sqlite` | `release/.env`, `release/storage`, `release/bootstrap/cache`, `release/database/database.sqlite` |
| Rails | `.env`, `storage/`, `tmp/`, `log/` | `release/.env`, `release/storage`, `release/tmp`, `release/log` |
| Django | `.env`, `media/`, `staticfiles/` (if runtime-generated) | `release/.env`, `release/media`, `release/staticfiles` |
| Next/Nuxt/SvelteKit/Vue | `.env` only | `release/.env` |

`wire_release` creates these symlinks as `foo-build` (so the symlink entries are build-owned, pointing at foo-run-owned shared leaves). The shared leaves themselves are created as `foo-run:foo-run 0750` — either by `bonesinfra` at provisioning or by a root-mediated `bonesremote shared init` step.

______________________________________________________________________

## 4. Open design questions (need your call)

### Q1. Keep a single global `git` user as ingress?

**My recommendation: yes, but downgrade to `git-shell`.** One global ingress user is operationally simpler and the blast radius is contained if (a) `git` is removed from all runtime groups, (b) `git`'s shell is `git-shell` (no interactive commands, only `git-receive-pack`/`git-upload-pack`), (c) the post-receive hook invokes `bonesremote` via a narrow sudoers rule rather than `git`'s own shell.

**Tradeoff:** if you want per-site ingress isolation (separate SSH keys per site, separate ingress users), that's `foo-ingress` per site — more users to manage, but a compromised key for site A cannot push to site B. For a single-admin-host model, global `git` + `git-shell` is the lazy-correct choice.

### Q2. Per-site `foo-build` user?

**My recommendation: yes, this is the core of the redesign.** Without it, V1/V5/V12 cannot be fixed. `foo-build` owns `build/`, `releases/`, runs checkout + build scripts, has its own HOME for tool caches.

**Tradeoff:** +1 system user per site. For N sites that's N+3 users (git + foo-build + foo-run + foo-release group) vs the current N+2 (git + foo + foo-release). The isolation gain (per-site build identity, per-site cache isolation, no global `git` owning all release code) is worth it.

### Q3. Per-site `foo-run` user?

**Already the case** (runtime_user defaults to project_name). Keep. Just rename conceptually to `foo-run` in docs and make `runtime_group` distinct from `runtime_user` (currently both default to `project_name`, so the runtime user's primary group == runtime_group, and adding `git` to runtime_group bleeds everywhere).

**My recommendation:** `runtime_user = "foo-run"`, `runtime_group = "foo-run"` (same name, but explicitly a group), `release_group = "foo-release"`. Drop `git` from `runtime_group` (V2 fix).

### Q4. `foo-release` group composition?

**My recommendation:** `foo-release` contains `foo-build` (primary or supplementary) and `foo-run` (supplementary, read-only intent). `git` is **not** a member. `foo-build` is the owner of release dirs/files; `foo-run` reads via group. Mode `2750` on `releases/` (setgid) ensures new release subdirs inherit `foo-release` as group.

### Q5. Bare repo location: `/home/git` vs `/srv/git` vs `/var/lib/bonesdeploy/repos`?

**My recommendation: `/srv/git/<project>.git`, owned `git:git 0750`.** Rationale: keeps `git`'s HOME (`/home/git`) clean for SSH/authorized_keys only, separates bare repos from HOME clutter, and `/srv` is the conventional location for site-served data. `git` still owns the repos (ingress). `foo-build` does **not** have direct repo access — see Q6.

**Tradeoff:** migrating existing repos from `/home/git/<project>.git` to `/srv/git/<project>.git` is a one-time `mv` + `bones.toml` `repo_path` update + remote URL update on dev machines. Low cost.

**Alternative (lazy):** keep `/home/git/<project>.git` but set `git`'s shell to `git-shell` and move build caches to `foo-build`'s HOME. Cheaper migration, but `git`'s HOME still mixes SSH identity with repo storage.

### Q6. Does `foo-build` get direct read access to the bare repo, or does `bonesremote` export a source archive?

**My recommendation: archive export via root mediator (your option A).** The post-receive hook (running as `git`) calls `sudo bonesremote release checkout --config <registry>` which as root:

1. Validates the repo/revision against the root-owned registry.
1. `git --git-dir=/srv/git/foo.git archive <rev> | sudo -u foo-build tar -x -C /srv/sites/foo/build/workspace`

This gives `foo-build` no direct access to `/srv/git/foo.git`, no traversal into `git`'s space, and the privileged step validates the rev before exporting.

**Tradeoff (option B):** give `foo-build` read-only group access to exactly one bare repo via a `foo-repo-read` group. Simpler (no archive step) but `foo-build` can traverse into `/srv/git` and read the full repo history (including `.git` metadata). Option A is tighter.

**Tradeoff (option C):** move repos under per-site paths like `/srv/sites/foo/repo.git` owned `foo-build:foo-build`. Then `foo-build` owns ingress too — but this collapses ingress and build identities for that site, weakening the isolation you want between push-access and build-access.

### Q7. Should `bonesremote` run as root and drop privileges to `foo-build` for checkout/build?

**My recommendation: yes, for the privileged boundary steps only.** `bonesremote` runs as root (via sudo from `git`'s post-receive hook), reads identity from `/etc/bonesdeploy/sites/foo.toml`, validates, then `setuid`/`sudo -u foo-build` for the actual checkout + build script execution. Build scripts run with `foo-build`'s HOME, no sudo, AppArmor-confined (future).

**Tradeoff:** this adds a privileged `release checkout` and `release build` command to `bonesremote`. Simpler alternative: keep the post-receive hook as `git`, but have it invoke `sudo -u foo-build bonesremote deploy --config <registry>` for the build steps. The hook itself stays unprivileged; only the `sudo -u` crosses the boundary. Narrower sudoers: `git ALL=(foo-build) NOPASSWD: /usr/local/bin/bonesremote deploy --config /etc/bonesdeploy/sites/*`.

### Q8. Build scripts run as `foo-build`?

**Yes, non-negotiable for the model.** `deploy.rs:74-85` currently runs `build_workspace/deployment/*.sh` as `git`. Change to run as `foo-build` (via the sudoers rule in Q7 or the setuid in Q7). HOME must be `/var/lib/bonesdeploy/build/foo` so composer/npm/cargo caches are per-site.

### Q9. How do build tools get HOME/cache without login?

**My recommendation:** `foo-build` is a system user with `home=/var/lib/bonesdeploy/build/foo`, `shell=/usr/sbin/nologin`, `create_home=True`. `bonesinfra` provisions `$HOME/.composer`, `$HOME/.npm`, `$HOME/.cargo`, `$HOME/.cache` as `foo-build:foo-build 0750`. Build scripts invoked via `sudo -u foo-build` set `HOME=/var/lib/bonesdeploy/build/foo`.

### Q10. Which paths are code artifacts vs runtime mutable?

See the table in §3. **Code artifacts** (build-owned, read-only at runtime): everything except the shared-symlink targets. **Runtime mutable** (foo-run-owned, under `shared/`): `.env`, `storage/`, `bootstrap/cache/`, `database/database.sqlite`, `tmp/`, `log/`, `media/`, `uploads/`, `staticfiles/` (if runtime-generated).

### Q11. Should `.env` be readable by `foo-build` during build?

**My recommendation: no, with a carve-out for Laravel.** `.env` lives at `shared/.env` owned `root:foo-run 0640` (via secrets push). `foo-build` cannot read it. For Laravel `php artisan config:cache` which needs `.env` at build time, run that specific command as `foo-run` via a narrow `bonesremote release warm --config <registry>` step, or accept that `config:cache` runs at first-request runtime (slower first boot, no secret leak to build).

**Tradeoff:** allowing `foo-build` to read `.env` simplifies Laravel builds but leaks secrets to the build identity (which has no login but could be compromised via the build scripts). The lazy-secure default: don't let build read secrets; run secret-dependent warm-up as `foo-run`.

### Q12. `bootstrap/cache` shared or per-release?

**My recommendation: shared.** Laravel's `bootstrap/cache` (compiled config, routes, views, events) should persist across releases for fast boot. Symlink `shared/bootstrap/cache` → `release/bootstrap/cache`, owned `foo-run:foo-run 0750`. Rebuilt by `php artisan config:cache` etc. running as `foo-run` (per Q11) on each deploy.

### Q13. How does `current` symlink activation work?

**My recommendation: root/bonesremote flips it.** `current` is `root:root` (already the case post-provisioning, `placeholder.py:18-23`). A new `bonesremote release activate --config <registry>` command (root-only, sudoers-allowed) atomically flips the symlink via `rename(2)`. `foo-build` stages the release but does not flip `current`.

**Tradeoff (lazy alternative):** let `foo-build` flip `current` (it owns `project_root` parent at `2751`). Simpler (no new privileged command) but gives the build identity control over which release is live — a compromised `foo-build` could point `current` at a malicious release. The root-mediated flip is one extra sudoers line and ~15 lines of Rust; worth it for the blast-radius reduction.

### Q14. Rollback?

**My recommendation: `bonesremote release rollback --config <registry>` (root-only).** Reads the registry, finds the previous release under `releases/`, flips `current` to it. No rebuild, no re-checkout. Since `current` is root-owned (Q13), rollback must be privileged.

### Q15. Service restart?

**Keep the current narrow sudoers, but tie it to the registry.** `git ALL=(root) NOPASSWD: <bonesremote> service restart --config /etc/bonesdeploy/sites/*` (note: registry path, not arbitrary `--config *`). `service::run` reads `project_name` from the registry, validates it matches an allowed pattern, and runs `systemctl restart <project>-nginx`. The `--config *` wildcard is replaced with a fixed registry path glob so `git` cannot point at an arbitrary file.

### Q16. AppArmor/systemd hardening?

**My recommendation: keep the existing per-site nginx + per-runtime app profiles, but:**

- Remove the `repo_bones_toml r,` grant from `project-nginx-profile.j2:32` (V3 fix — nginx config is at `/srv/conf/<project>/nginx.conf`, doesn't need the repo's `bones.toml`).
- Change `runtime_write_paths` for Rails/Django to point at `shared/...` symlinks, not `current/...` (V6 fix).
- Add an AppArmor profile for `foo-build`'s build scripts (future hardening; not v1).

### Q17. What does nginx need access to?

**System nginx (`www-data`):** only needs to traverse `/run/<project>/` (`0711`) and connect to the per-site nginx socket. Does **not** need to read release code or `bones.toml`. The per-site nginx (running as `foo-run`) reads `current/public/**` via AppArmor grant `r`.

**Per-site nginx (`foo-run`):** reads `/srv/conf/<project>/nginx.conf` (root:foo-run 0750), `current/public/**` (foo-build:foo-release, foo-run reads via group), `/run/<project>/*.sock` (rw). No access to `shared/` secrets, no access to `bones.toml`.

### Q18. How to avoid `www-data` as universal cross-site read key?

**My recommendation for v1: accept the documented ceiling** (`site-nginx.conf.j2:21-24` ponytail). The upgrade path (per the ponytail comment): set per-site nginx group to `www-data` and use `0660` socket mode, so only `www-data` can connect — but `www-data` is still global. The real fix is per-site nginx master processes running as distinct users, which is a larger refactor. For v1, `0711` on `/run/<project>/` + `0660` sockets (group `www-data`) is the lazy-correct ceiling.

### Q19. Minimum viable safe v1?

**My recommendation for the smallest change that fixes the critical violations:**

1. **Add `foo-build` user** in `bonesinfra/users.py` (per-site, system, nologin, HOME at `/var/lib/bonesdeploy/build/foo`).
2. **Re-provision `project_root`, `releases/`, `build/` as `foo-build:foo-release`** in `bonesinfra/directories.py` (replace `ctx.config.deploy_user` with `ctx.config.build_user`).
3. **Remove `git` from `runtime_group`** (`users.py:47` delete).
4. **Tighten `shared/` to `0750`** (`directories.py:72`), drop setgid (only `foo-run` writes).
5. **Add `/etc/bonesdeploy/sites/<project>.toml`** root-owned registry, written by `bonesremote init`, read by `bonesinfra` and privileged `bonesremote` commands.
6. **Sudoers:** replace `--config *` with `--config /etc/bonesdeploy/sites/*` for `service restart`; add `git ALL=(foo-build) NOPASSWD: <bonesremote> deploy --config /etc/bonesdeploy/sites/*` so the build runs as `foo-build`.
7. **`deploy.rs`:** run checkout + build scripts + publish as `foo-build` (via the sudoers rule). `cp -a` now preserves `foo-build` ownership into the release tree — correct.
8. **`wire_release`:** implement the `[shared] paths` symlinks (storage, bootstrap/cache, etc.), created as `foo-build`-owned symlinks pointing at `foo-run`-owned shared leaves.
9. **Move Rails/Django `runtime_write_paths` to `shared/...`** symlinks (V6 fix).
10. **Reconcile docs** (`02-filesystem-layout.md`, `PROJECT.md`, `03-identity-and-user-policy.md`, `22-desired-end-state-summary.md`) to the 3-identity model and the actual modes.

**Explicitly deferred to v2:** `git-shell` for `git` (Q1 deeper hardening), per-site AppArmor for build scripts, `www-data` cross-site fix (Q18), root-mediated `current` flip (Q13 — lazy v1 lets `foo-build` flip since it owns the parent), rollback command (Q14).

### Q20. What code/docs need to change?

**bonesinfra (Python):**

- `domain/context.py`: add `build_user` field, default `f"{project_name}-build"`.
- `deploys/setup/users.py`: create `foo-build`, remove `git` from `runtime_group` (line 47).
- `deploys/setup/directories.py`: owner `foo-build` for `project_root`, `releases/`, `build/`; `shared/` mode `0750`.
- `runtimes/rails/rails.py`, `runtimes/django/django.py`: `runtime_write_paths` → `shared/...`.
- `assets/apparmor/project-nginx-profile.j2:32`: remove `repo_bones_toml r,`.

**bonesdeploy (Rust):**

- `crates/shared/src/config.rs`: add `build_user_for()`, add registry-loading helpers.
- `crates/shared/src/paths.rs`: add `/etc/bonesdeploy/sites/` registry path constants.
- `crates/bonesremote/src/commands/init.rs`: write `/etc/bonesdeploy/sites/<project>.toml`; tighten sudoers to registry paths + `foo-build` deploy rule.
- `crates/bonesremote/src/commands/deploy.rs`: invoke checkout/build/publish as `foo-build` via `sudo -u`.
- `crates/bonesremote/src/commands/wire_release.rs`: implement `[shared] paths` symlinks; fix `.env.example` fallback (chown to `root:foo-run 0640` or fail loudly if secrets not pushed).
- `crates/bonesremote/src/commands/service.rs`: load `project_name` from registry, not `--config`.
- `crates/bonesdeploy/src/commands/secrets.rs`: validate `runtime_group` against registry.
- `crates/bonesdeploy/kit/hooks/hooks.sh:81,86`: invoke `bonesremote deploy` via `sudo -u foo-build`.

**Docs:**

- `docs/security/02-filesystem-layout.md`: fix modes table, delete "activation hardening" myth, document shared-symlink model.
- `docs/security/03-identity-and-user-policy.md`: 3-identity model.
- `docs/security/22-desired-end-state-summary.md`: 3-identity model.
- `docs/PROJECT.md`: fix `shared/` mode (0750 not 0711/2775), ownership table (foo-build not git for releases), wire_release coverage.

______________________________________________________________________

## 5. The one-sentence version

The redesign's spine is: **introduce `foo-build` per site, move every deploy-side ownership from `git` to `foo-build`, remove `git` from `runtime_group`, tighten `shared/` to `foo-run`-exclusive, add a root-owned site registry that privileged commands read instead of git-owned config, and wire shared symlinks for every runtime-mutable path so the runtime user never writes inside the release tree.**

______________________________________________________________________

## What I need from you

Before I draft an implementation plan, I need your calls on:

1. **Q1** — global `git` (git-shell) vs per-site `foo-ingress`?
2. **Q5** — move bare repos to `/srv/git` or keep `/home/git`?
3. **Q6** — archive-export (option A) vs `foo-build` read access to bare repo (option B)?
4. **Q7** — `bonesremote` runs as root and drops to `foo-build`, or `git` invokes `sudo -u foo-build bonesremote deploy`?
5. **Q11** — does `foo-build` read `.env` during build, or do secret-dependent warm-up commands run as `foo-run`?
6. **Q13** — root-mediated `current` flip (tighter) or `foo-build` flips (lazier)?
7. **Q19** — is the v1 scope right, or do you want `git-shell`/root-flip/rollback in v1 too?

Once I have those, I'll write the migration path as a sequenced checklist with exact file:line edits for both repos.
