# bonesremote hooks post-receive

## Overview

Checks out the configured branch or revision into the deployment build workspace. This is one of the building blocks of the deployment pipeline — typically you would run `bonesremote deploy --config <path>` instead, which orchestrates the full lifecycle (doctor, stage, checkout, wire, scripts, activate, restart, prune) in a single call.

This subcommand is useful when you need only the checkout step, or when composing a custom pipeline from individual building blocks.

## Command Signature

```bash
bonesremote hooks post-receive --config <path> [--revision <ref>]
```

**Flags:**
- `--config <path>`: Path to `bones.toml` configuration file (required)
- `--revision <ref>`: Optional Git revision to checkout (default: configured branch)

**Note:** Must NOT be run as root (runs as deploy user).

---

## Detailed Execution Steps

### 1. Verify Not Running as Root

**Source:** `post_receive.rs:13`

```rust
privileges::ensure_not_root("bonesremote hooks post-receive")?;
```

Ensures the hook runs as the deploy user, not root.

---

### 2. Load Configuration

**Source:** `post_receive.rs:15`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration to determine:
- Git directory path
- Build workspace path
- Default branch
- Shared paths configuration

---

### 3. Define Build Workspace Path

**Source:** `post_receive.rs:16`

```rust
let build_root = release_state::build_root(&cfg);
```

**Path:** `/srv/deployments/myapp/build/workspace`

---

### 4. Verify Build Workspace Exists

**Source:** `post_receive.rs:18-20`

```rust
if !build_root.exists() {
    bail!("Build workspace does not exist: {}", build_root.display());
}
```

Validates that `release stage` was run and the build workspace was created.

**If missing:** Indicates staging didn't happen or failed.

---

### 5. Determine Checkout Target

**Source:** `post_receive.rs:22`

```rust
let checkout_target = revision.unwrap_or(cfg.data.branch.as_str());
println!("Checking out {checkout_target} to {}...", build_root.display());
```

**Priority:**
1. `--revision` flag if provided
2. Configured branch from `bones.toml` (default: `master`)

**Example:**
- Flag provided: `--revision feature/new-api` → checks out `feature/new-api`
- No flag: Uses `branch: master` from config → checks out `master`

---

### 6. Execute Git Checkout

**Source:** `post_receive.rs:24-40`

```rust
let status = Command::new("git")
    .arg("--work-tree")
    .arg(&build_root)
    .arg("--git-dir")
    .arg(&cfg.data.repo_path)
    .arg("checkout")
    .arg("-f")
    .arg(checkout_target)
    .status()
    .with_context(|| {
        format!("Failed to run git checkout for target '{checkout_target}' into {}", build_root.display())
    })?;

if !status.success() {
    bail!("git checkout failed for target '{checkout_target}': status {status}");
}
```

#### 6.1 Git Command Breakdown

**Full Command:**
```bash
git \
  --work-tree /srv/deployments/myapp/build/workspace \
  --git-dir /home/git/myapp.git \
  checkout \
  -f \
  master
```

**Flags:**
- `--work-tree`: Where to check out files (build workspace)
- `--git-dir`: Path to bare repository
- `checkout`: Switch to branch/commit
- `-f`: Force checkout (overwrite uncommitted changes)
- `{target}`: Branch name or commit SHA

#### 6.2 Checkout Behavior

**For branch:**
- Checks out the latest commit on that branch
- All files from that commit are placed in `build_root`

**For commit SHA:**
- Checks out specific commit
- Detached HEAD state
- Used for precise deployment control

#### 6.3 Force Flag (-f)

**Why force?**
- Overwrites any existing files in `build_root`
- Handles case where `build_root` has leftover files
- Ensures clean checkout

---

### 7. Wire Shared Paths (Separate Step)

**Note:** `post_receive.rs` only handles git checkout. Shared path wiring is a separate step — `wire_release::run(config_path)` — called by `bonesremote deploy` (the unified command) after checkout. In the old hook pipeline, wiring happened as a distinct subcommand after post-receive.

---

### 8. Print Success (Implicit)

The command succeeds silently unless an error occurs. The `wire_release` command prints progress messages.

---

## Execution Context

### When This Runs

#### Automatic (via Git Hook)

The `post-receive` hook in the bare repository calls the unified `bonesremote deploy --config <path> --revision <rev>` command. Internally, that calls `hooks post-receive` (checkout) as one step in its pipeline:

```
git push production master
  ↓
/home/git/myapp.git/hooks/post-receive
  ↓
bonesremote deploy --config <path> --revision <newrev>
  ├─ doctor
  ├─ stage_release
  ├─ hooks post-receive (checkout)  ← you are here
  ├─ release wire
  ├─ hooks deploy (scripts + activate)
  ├─ service restart (sudo)
  └─ hooks post-deploy (prune)
```

#### Manual (via bonesdeploy deploy)

`bonesdeploy deploy` runs `bonesremote deploy --config <remote_bones_toml>` directly over SSH — without git hooks. That in turn calls all the same building blocks internally.

#### Standalone

You can also run this subcommand directly if you only need the checkout step:

```bash
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.toml --revision main
```

**Use case:** Custom pipelines, debugging, manual checkout without running deploy scripts.

---

## Environment Variables

This subcommand does not use any specific environment variables. Git hook environment variables (`GIT_DIR`, etc.) are handled by the `post-receive` shell script that calls `bonesremote deploy --config <path>` — they are not passed through to this subcommand.

---

## Checkout Targets

### Branch Checkout

**Default behavior:** Checkout configured branch

```toml
[data]
branch = "master"
```

**Command:** `git checkout -f master`

**Result:** Latest commit on `master` branch

### Specific Commit

**Usage:** `--revision <sha>`

**Command:** `git checkout -f abc123`

**Result:** Specific commit checked out

**Use case:** Deploy a specific version or tag

### Tag Checkout

**Usage:** `--revision v1.2.3`

**Command:** `git checkout -f v1.2.3`

**Result:** Tag checked out (detached HEAD)

---

## Directory State After post-receive

### Before (empty build workspace)

```
/srv/deployments/myapp/
├── build/
│   └── workspace/           # Empty
├── releases/
│   └── 20260507_150432/     # Staged, empty
└── shared/
    ├── .env
    └── storage/
```

### After (checked out and wired)

```
/srv/deployments/myapp/
├── build/
│   └── workspace/
│       ├── .env -> ../../shared/.env
│       ├── storage -> ../../shared/storage
│       ├── src/
│       ├── public/
│       ├── package.json
│       └── ... (all project files)
├── releases/
│   └── 20260507_150432/     # Still empty, waiting for deployment
└── shared/
    ├── .env
    └── storage/
```

---

## Error Scenarios

### Build Workspace Missing

```
Build workspace does not exist: /srv/deployments/myapp/build/workspace
```

**Cause:** `release stage` not run

**Solution:** Run staging first:
```bash
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.toml
```

### Git Checkout Failed

```
git checkout failed for target 'master': status 1
```

**Possible causes:**
- Branch doesn't exist
- Bare repository is empty
- Git repository corrupted
- Permission denied

### Wiring Failed

```
Failed to create shared symlink ...
```

**Possible causes:**
- Permission denied
- Disk space
- Shared directory doesn't exist

---

## Integration with Deployment Pipeline

### Full Pipeline (Recommended)

```
git push production master
  ↓
pre-push (local doctor) → pre-receive (inert) → post-receive
  ↓
bonesremote deploy --config <path> --revision <rev>
  ├─ doctor
  ├─ stage_release
  ├─ hooks post-receive (checkout)  ← you are here
  ├─ release wire
  ├─ hooks deploy (scripts + activate)
  ├─ service restart (sudo)
  └─ hooks post-deploy (prune)
```

### Manual Deployment (Recommended)

```bash
bonesdeploy deploy
  ↓ (SSH → bonesremote deploy --config <path>)
```

### Standalone Usage (Building Blocks)

These subcommands are available for custom pipelines:

```bash
# All in one (recommended)
bonesremote deploy --config /home/git/myapp.git/bones/bones.toml

# Or individual building blocks:
bonesremote release stage --config /path/to/bones.toml
bonesremote hooks post-receive --config /path/to/bones.toml
bonesremote release wire --config /path/to/bones.toml
bonesremote hooks deploy --config /path/to/bones.toml
sudo bonesremote service restart --config /path/to/bones.toml
bonesremote hooks post-deploy --config /path/to/bones.toml
```

---

## Related Commands

- `bonesremote release stage` - Stage a new release
- `bonesremote release wire` - Wire shared paths
- `bonesremote hooks deploy` - Run deployment and activate
- `bonesremote hooks post-deploy` - Post-deployment tasks
- `bonesdeploy deploy` - Manual deployment trigger
