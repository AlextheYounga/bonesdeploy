# bonesremote hooks post-receive

## Overview

Git hook that runs after a push is received by the bare repository. It checks out the pushed code to the build workspace and wires shared paths. This is the first step in the deployment pipeline, triggered automatically by `git push` or manually via `bonesdeploy deploy`.

## Command Signature

```bash
bonesremote hooks post-receive --config <path> [--revision <ref>]
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)
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
2. Configured branch from `bones.yaml` (default: `master`)

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

### 7. Wire Shared Paths

**Source:** `post_receive.rs:42`

```rust
wire_release::run(config_path)?;
```

Calls `bonesremote release wire` to:
1. Move existing files to shared (first deployment)
2. Create shared files/directories if needed
3. Create symlinks from build workspace to shared paths

**After wiring, the build workspace has:**
```
build/workspace/
├── .env -> ../../shared/.env
├── storage -> ../../shared/storage
├── logs -> ../../shared/logs
├── src/                    # From git checkout
├── public/                 # From git checkout
├── package.json            # From git checkout
└── ...                     # Other project files
```

---

### 8. Print Success (Implicit)

The command succeeds silently unless an error occurs. The `wire_release` command prints progress messages.

---

## Execution Context

### When This Runs

#### Automatic (via Git Hook)

The `post-receive` hook in the bare repository is triggered by `git push`:

```
/home/git/myapp.git/hooks/post-receive
  ↓
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.yaml
```

**Flow:**
1. Developer runs `git push production master`
2. Git receives the push
3. `post-receive` hook executes
4. Code checked out to `build/workspace`
5. Shared paths wired
6. Deployment continues

#### Manual (via bonesdeploy)

The `bonesdeploy deploy` command runs this hook manually:

```bash
bonesdeploy deploy
  ↓ (SSH to server)
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.yaml
```

**Use case:** Re-deploy without pushing new commits.

---

## Environment Variables

When run as a Git hook, the following environment variables are available:

- `GIT_DIR`: Path to the bare repository
- `BONES_FORCE_DEPLOY`: Set to `1` when run manually (bypasses validation)

**From `bonesdeploy deploy`:**
```rust
ssh::stream_cmd(
    &session,
    &format!(
        "BONES_FORCE_DEPLOY=1 GIT_DIR='{repo_path}' '{repo_path}/{}/{}' </dev/null",
        config::Constants::REMOTE_HOOKS_DIR,
        config::Constants::POST_RECEIVE_HOOK
    ),
)
.await?;
```

---

## Checkout Targets

### Branch Checkout

**Default behavior:** Checkout configured branch

```yaml
# bones.yaml
data:
  branch: master
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
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.yaml
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

### Full Pipeline

```
git push production master
  ↓
pre-receive hook (validation)
  ↓
release stage (create directories)
  ↓
post-receive hook ← YOU ARE HERE
  ├─ git checkout
  └─ wire shared paths
  ↓
hooks deploy (run scripts, activate)
  ↓
hooks post-deploy (harden permissions, restart)
```

### Manual Deployment Pipeline

```bash
# 1. Stage
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.yaml

# 2. Post-receive (checkout + wire)
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.yaml

# 3. Deploy
bonesremote hooks deploy --config /home/git/myapp.git/bones/bones.yaml

# 4. Post-deploy
sudo bonesremote hooks post-deploy --config /home/git/myapp.git/bones/bones.yaml
```

---

## Related Commands

- `bonesremote release stage` - Stage a new release
- `bonesremote release wire` - Wire shared paths
- `bonesremote hooks deploy` - Run deployment and activate
- `bonesremote hooks post-deploy` - Post-deployment tasks
- `bonesdeploy deploy` - Manual deployment trigger
