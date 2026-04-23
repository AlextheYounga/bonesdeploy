# Symlink-Based Releases and Rollbacks

## Context

The legacy model checked out directly into a live worktree path and overwrote files in place. The release model introduced here deploys into timestamped release directories, points `current` at the active release, and performs rollback by repointing that symlink.

No backward compatibility is needed — no one else is using bonesdeploy yet.

## Target Directory Structure

```
/srv/deployments/lawsnipe/
├── releases/
│   ├── 20260323_183800/
│   ├── 20260323_192100/
│   └── 20260323_201500/  ← latest
├── shared/
│   ├── .env
│   ├── storage/
│   └── node_modules/
└── current -> releases/20260323_201500/
```

`/var/www/lawsnipe` should be a symlink to `/srv/deployments/lawsnipe/current`.

## New Config Fields

Add `[releases]` section to bones.toml:

```toml
[releases]
keep = 5
shared_paths = [".env", "storage", "node_modules"]
```

## New Deploy Flow

1. **pre-receive**: `sudo bonesremote doctor`, then `sudo bonesremote release stage --config ...`
    - Creates `releases/` and `shared/` dirs if missing
    - Creates `releases/{YYYYMMDD_HHMMSS}/`
    - Chowns new release dir + shared dir to deploy user
    - Writes release name to `{git_dir}/bones/.staged_release` (hook state file)
2. **post-receive** runs three remote hook commands in order:
   - `sudo bonesremote hooks post-receive --config ...`
   - `sudo bonesremote hooks deploy --config ...`
   - `sudo bonesremote hooks post-deploy --config ...`
3. **bonesremote hooks post-receive**: `git checkout -f` into `releases/{timestamp}` and wires shared paths.
4. **bonesremote hooks deploy**: runs deployment scripts, then calls `sudo bonesremote release activate --config ...`
    - Symlinks each shared_path from release dir → `shared/`
    - Atomically swaps `current` symlink (create tmp link, then `rename`)
    - Prunes old releases beyond `keep` count
5. **bonesremote hooks post-deploy**: hardens permissions on the release dir pointed to by `current` + `shared/`
   - Hardens permissions on the release dir pointed to by `current` + `shared/`

If a deployment script fails at step 4, `current` still points to the previous release. The site stays up.

## New Commands

### bonesremote (server-side)

| Command | Description |
|---------|-------------|
| `release stage --config` | Create release dir, chown, write state file |
| `release activate --config` | Symlink shared paths, swap `current`, prune old releases |
| `release rollback --config` | Re-point `current` to previous release |
| `hooks post-receive --config` | Checkout into staged release and wire shared paths |
| `hooks deploy --config` | Run deployment scripts and activate/drop failed release |
| `hooks post-deploy --config` | Harden permissions on active release + shared paths |

### bonesdeploy (local CLI)

| Command | Description |
|---------|-------------|
| `rollback` | SSH in, run `sudo bonesremote release rollback` |

## Implementation Order

### Step 1: Config structs (both crates)
Add `Releases` struct and `releases` field to `BonesConfig`.

**Files:**
- `crates/bonesdeploy/src/config.rs`
- `crates/bonesremote/src/config.rs`

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Releases {
    #[serde(default = "default_keep")]
    pub keep: u32,
    #[serde(default)]
    pub shared_paths: Vec<String>,
}

fn default_keep() -> u32 { 5 }
```

Add to `BonesConfig`:
```rust
#[serde(default)]
pub releases: Option<Releases>,
```

### Step 2: Refactor permissions.rs
Extract `harden` logic so it can accept explicit target paths. This allows hardening just the current release + shared dir.

**File:** `crates/bonesremote/src/permissions.rs`

- Rename existing `harden(cfg)` → `harden_paths(cfg, paths: &[&str])`
- Add `harden_release(cfg)` that resolves `current` target + `shared/` and calls `harden_paths`
- Same for `chown_to_deploy_user` — add variant that chowns specific dirs

### Step 3: New remote commands
**New files:**
- `crates/bonesremote/src/commands/stage_release.rs`
- `crates/bonesremote/src/commands/activate_release.rs`
- `crates/bonesremote/src/commands/rollback.rs`

**Modified:** `crates/bonesremote/src/commands/mod.rs` — register new subcommands

#### stage_release.rs
1. Load config
2. Create `{deploy_root}/releases/` and `{deploy_root}/shared/` if missing
3. Generate timestamp: `chrono::Local::now().format("%Y%m%d_%H%M%S")`
4. Create `{deploy_root}/releases/{timestamp}/`
5. Chown new release dir + shared dir to deploy user
6. Write timestamp to `{git_dir}/bones/.staged_release`

Requires adding `chrono` dep to `crates/bonesremote/Cargo.toml`.
The `--config` arg gives us `bones.toml` path. Derive `git_dir` as the parent of `bones/bones.toml`.

#### activate_release.rs
1. Load config, read release name from `.staged_release`
2. For each `shared_paths` entry:
   - If path exists in release dir but not in shared: move it to shared (first deploy seeds shared from checkout)
   - If path doesn't exist in shared: create it (mkdir for dirs, touch for files)
   - Remove path from release dir, create symlink `release/{path}` → `../../shared/{path}`
3. Atomic symlink swap: `symlink(target, "current.tmp")` then `rename("current.tmp", "current")`
4. Prune: list `releases/`, sort, remove oldest beyond `keep` (never remove what `current` points to)
5. Clean up `.staged_release` state file

#### rollback.rs
1. Load config
2. Read `current` symlink target to get active release name
3. List `releases/`, sort, find the one before current
4. Atomic symlink swap to previous release
5. Print old → new release names

### Step 4: Update hook templates
**Files in `kit/hooks/`:**

**pre-receive** — run `sudo bonesremote release stage --config "$BONES_TOML"`

**post-receive** — orchestrate all deployment stages via remote hook commands:
```bash
sudo bonesremote hooks post-receive --config "$BONES_TOML"
sudo bonesremote hooks deploy --config "$BONES_TOML"
sudo bonesremote hooks post-deploy --config "$BONES_TOML"
```

### Step 5: Local rollback command
**New file:** `crates/bonesdeploy/src/commands/rollback.rs`

Same pattern as `redeploy.rs`: load config, SSH in, `stream_cmd` to run `sudo bonesremote release rollback --config ...`.

**Modified:** `crates/bonesdeploy/src/commands/mod.rs` — add `Rollback` variant

### Step 6: Update bones.toml template and prompts
- `kit/bones.toml` — add `[releases]` section
- `crates/bonesdeploy/src/prompts.rs` — add prompts for `keep` and `shared_paths` during `bonesdeploy init`

## Verification

1. `cargo check` — both crates compile
2. `cargo build --release` — build both binaries
3. Deploy `bonesremote` to server, run `bonesdeploy push` to sync new hooks
4. Update `bones.toml` with `[releases]` section, run `bonesdeploy push`
5. Update nginx document root to `/var/www/lawsnipe/current/public`, reload nginx
6. Move `.env` and `storage/` into `/var/www/lawsnipe/shared/` manually (first time only)
7. `git push production master` — verify:
   - Release dir created in `releases/`
   - `current` symlink points to new release
   - Shared paths are symlinked into release dir
   - Site works
8. Push again — verify pruning keeps only `keep` releases
9. `bonesdeploy rollback` — verify `current` swaps to previous release, site serves old code
10. `bonesdeploy redeploy` — verify it works with the new release flow
