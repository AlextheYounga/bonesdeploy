# bonesremote release drop-failed

## Overview

Cleans up a failed deployment by removing the staged release directory and clearing the staged release state. This command is called automatically when deployment scripts fail, or can be run manually to clean up after a failed deployment attempt.

## Command Signature

```bash
bonesremote release drop-failed --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

---

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `drop_failed_release.rs:10`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration to determine paths.

---

### 2. Define Staged Release Path

**Source:** `drop_failed_release.rs:11`

```rust
let staged_path = release_state::staged_release_path(&cfg);
```

**Path:** `<git_dir>/bones/.staged_release`

**Example:** `/home/git/myapp.git/bones/.staged_release`

---

### 3. Check for Staged Release State

**Source:** `drop_failed_release.rs:13-16`

```rust
if !staged_path.exists() {
    println!("No staged release state found. Nothing to clean.");
    return Ok(());
}
```

If no staged release state file exists, there's nothing to clean up. Prints message and exits successfully.

**This is not an error** - allows idempotent cleanup.

---

### 4. Read Staged Release Name

**Source:** `drop_failed_release.rs:18`

```rust
let release_name = release_state::read_staged_release(&cfg)?;
```

Reads the failed release name from the state file.

**Example:** `20260507_150432`

---

### 5. Define Release Directory Path

**Source:** `drop_failed_release.rs:19`

```rust
let release_dir = release_state::release_dir(&cfg, &release_name);
```

**Path:** `/srv/deployments/myapp/runtime/20260507_150432`

---

### 6. Remove Release Directory

**Source:** `drop_failed_release.rs:21-25`

```rust
if release_dir.exists() {
    fs::remove_dir_all(&release_dir)
        .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
    println!("Removed failed release: {release_name}");
}
```

**Actions:**
1. Check if release directory exists
2. Remove entire directory tree (`remove_dir_all`)
3. Print confirmation

**Why check exists?**
- Deployment may have failed before release directory was created
- Release directory may have been manually removed
- Idempotent operation

**remove_dir_all:** Recursively removes directory and all contents.

---

### 7. Clear Staged Release State

**Source:** `drop_failed_release.rs:27`

```rust
release_state::clear_staged_release(&cfg)?;
```

Deletes the `<git_dir>/bones/.staged_release` file.

---

### 8. Print Success Message

**Source:** `drop_failed_release.rs:28`

```rust
println!("Cleared staged release state.");
```

**Example Output:**
```
Removed failed release: 20260507_150432
Cleared staged release state.
```

Or if nothing to clean:
```
No staged release state found. Nothing to clean.
```

---

## When This Runs

### Automatic Invocation

**Source:** `deploy.rs:46-49`

Called automatically by `bonesremote hooks deploy` when a deployment script fails:

```rust
if !status.success() {
    println!("Deployment script {script_name} failed.");
    drop_failed_release::run(config_path)
        .with_context(|| "Failed to drop staged release after deployment script failure")?;
    bail!("Deployment script {script_name} failed with status {status}");
}
```

**Flow:**
1. Deployment script fails (non-zero exit)
2. `drop_failed_release` called automatically
3. Failed release removed
4. Deployment aborts
5. Current symlink remains pointing to previous release

### Manual Invocation

Can be run manually to clean up after:
- Aborted deployments
- Failed post-receive hooks
- Manual intervention scenarios

---

## Directory State After Cleanup

### Before (Failed Deployment)

```
/srv/deployments/myapp/
├── runtime/
│   ├── 20260507_140000/    # Previous release (still current)
│   └── 20260507_150432/    # Failed release (partial/incomplete)
└── current -> runtime/20260507_140000/

/home/git/myapp.git/bones/
├── bones.yaml
└── .staged_release         # Contains: 20260507_150432
```

### After (drop-failed)

```
/srv/deployments/myapp/
├── runtime/
│   └── 20260507_140000/    # Still current
└── current -> runtime/20260507_140000/

/home/git/myapp.git/bones/
├── bones.yaml
└── (no .staged_release)    # Cleared
```

---

## Why Cleanup Failed Releases?

### Disk Space

- Failed releases consume disk space
- Build artifacts, dependencies, etc.
- Prevents accumulation of garbage

### Avoid Confusion

- Failed releases shouldn't appear in release list
- Prevents accidentally activating broken release
- Clear state for next deployment attempt

### State Consistency

- Staged release state indicates deployment in progress
- Clearing it allows new deployment to start
- Prevents conflicts between deployment attempts

---

## Typical Workflow

### Automatic Cleanup (Failed Deployment)

```bash
# 1. Deployment starts
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.yaml
# Created: /srv/deployments/myapp/runtime/20260507_150432/

# 2. Checkout and wire
# ...

# 3. Deployment script fails
bonesremote hooks deploy --config /home/git/myapp.git/bones/bones.yaml
# Script: 03_migrate.sh fails

# 4. Automatic cleanup
# - Removes /srv/deployments/myapp/runtime/20260507_150432/
# - Removes .staged_release

# 5. Deployment aborts
# Current release remains active: 20260507_140000
```

### Manual Cleanup

```bash
# Deployment failed or was aborted
# Manual cleanup needed

bonesremote release drop-failed --config /home/git/myapp.git/bones/bones.yaml
# Output:
# Removed failed release: 20260507_150432
# Cleared staged release state.

# Ready for next deployment attempt
```

---

## Idempotent Behavior

### Multiple Calls Safe

```bash
# First call
bonesremote release drop-failed --config /home/git/myapp.git/bones/bones.yaml
# Output: Removed failed release: 20260507_150432
#         Cleared staged release state.

# Second call (nothing to clean)
bonesremote release drop-failed --config /home/git/myapp.git/bones/bones.yaml
# Output: No staged release state found. Nothing to clean.
```

**Always succeeds** - safe to call multiple times.

---

## What's NOT Cleaned

### Shared Files

- Files in `/srv/deployments/myapp/shared/` are NOT touched
- These persist across deployments
- Even if deployment failed

### Build Workspace

- `/srv/deployments/myapp/build/workspace/` is NOT cleaned
- May contain useful debugging information
- Will be overwritten by next deployment

### Current Release

- The active release is never affected
- Application continues running on previous release
- Zero downtime

---

## Error Scenarios

### Permission Denied

```
Failed to remove failed release /srv/deployments/myapp/runtime/20260507_150432
```

**Possible causes:**
- Running as wrong user
- Files owned by root
- File permissions issue

**Solution:** Run with appropriate permissions or as root.

### State File Read Error

```
Failed to read staged release state at /home/git/myapp.git/bones/.staged_release
```

**Possible causes:**
- File corrupted
- Permission denied
- File was deleted during operation

---

## Related Commands

- `bonesremote release stage` - Stage a new release
- `bonesremote release activate` - Activate a release
- `bonesremote hooks deploy` - Deployment command (calls drop-failed on failure)
- `bonesremote doctor` - Check for issues
