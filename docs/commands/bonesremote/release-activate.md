# bonesremote release activate

## Overview

Atomically switches the `current` symlink to point to the staged release, making it the active deployment. Called internally by `bonesremote deploy --config <path>` (the recommended unified command) and by `bonesremote hooks deploy`. Use directly when composing a custom pipeline from individual building blocks.

## Command Signature

```bash
bonesremote release activate --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.toml` configuration file (required)

**Note:** Must NOT be run as root (runs as deploy user).

---

## Detailed Execution Steps

### 1. Verify Not Running as Root

**Source:** `activate_release.rs:10`

```rust
privileges::ensure_not_root("bonesremote release activate")?;
```

Ensures the command is not running as root. Activation should be done as the deploy user for security and proper permissions.

---

### 2. Load Configuration

**Source:** `activate_release.rs:12`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration to determine paths.

---

### 3. Read Staged Release Name

**Source:** `activate_release.rs:13`

```rust
let release_name = release_state::read_staged_release(&cfg)?;
```

Reads the staged release name from `<repo_path>/bones/.staged_release`.

**Example:** `20260507_150432`

---

### 4. Define Paths

**Source:** `activate_release.rs:14-15`

```rust
let release_dir = release_state::release_dir(&cfg, &release_name);
let current_link = release_state::current_link(&cfg);
```

**Paths:**
- `release_dir`: `/srv/deployments/myapp/releases/20260507_150432`
- `current_link`: `/srv/deployments/myapp/current`

---

### 5. Verify Staged Release Exists

**Source:** `activate_release.rs:17-19`

```rust
if !release_dir.exists() {
    anyhow::bail!("Staged release directory does not exist: {}", release_dir.display());
}
```

Ensures the release directory was actually created and populated.

**Fails if:** Release directory doesn't exist (deployment never completed or failed).

---

### 6. Validate Current Link State

**Source:** `activate_release.rs:21-23`

```rust
if current_link.exists() && !current_link.is_symlink() {
    bail!("current exists and is not a symlink: {}", current_link.display());
}
```

**Validates:** If `current` exists, it must be a symlink.

**Why?**
- `current` should always be a symlink to the active release
- If it's a real directory, activation would fail or have unexpected behavior
- Indicates a configuration error or manual intervention

**Example Error:**
```
current exists and is not a symlink: /srv/deployments/myapp/current
```

**Solution:** Remove or rename the directory, then re-run.

---

### 7. Define Current Link

**Source:** `activate_release.rs:25`

```rust
let current_link = release_state::current_link(&cfg);
```

**Path:** `/srv/deployments/myapp/current`

---

### 8. Atomically Switch Current

**Source:** `activate_release.rs:27-29`

```rust
release_state::point_symlink_atomically(&current_link, &release_dir)?;
```

**Creates:** `/srv/deployments/myapp/current` → `/srv/deployments/myapp/releases/20260507_150432`

**Purpose:** Points `current` to the new release.

#### 8.3 Atomic Symlink Switch

**Implementation:** `release_state.rs:107-130`

```rust
pub fn point_symlink_atomically(link_path: &Path, target_path: &Path) -> Result<()> {
    let Some(parent) = link_path.parent() else {
        bail!("Invalid symlink path: {}", link_path.display());
    };

    fs::create_dir_all(parent).with_context(|| format!("Failed to create symlink parent: {}", parent.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    let temp_name = format!(".tmp_current_{}_{}", process::id(), nanos);
    let temp_link = parent.join(temp_name);

    if fs::symlink_metadata(&temp_link).is_ok() {
        fs::remove_file(&temp_link)
            .with_context(|| format!("Failed to cleanup stale temp link: {}", temp_link.display()))?;
    }

    symlink(target_path, &temp_link).with_context(|| {
        format!("Failed to create temporary symlink {} -> {}", temp_link.display(), target_path.display())
    })?;

    fs::rename(&temp_link, link_path).with_context(|| {
        format!("Failed to atomically switch symlink {} -> {}", link_path.display(), target_path.display())
    })
}
```

**Atomic Operation:**
1. Create temporary symlink: `.tmp_current_<pid>_<nanos>` → `target_path`
2. Atomically rename temp symlink to final location: `.tmp_current_*` → `current`
3. Rename is atomic on POSIX systems (single operation)

**Why atomic?**
- No window where `current` doesn't exist
- No window where `current` points to wrong location
- Instantaneous switch (filesystem operation)
- No race conditions

---

### 9. Clear Staged Release State

**Source:** `activate_release.rs:31`

```rust
release_state::clear_staged_release(&cfg)?;
```

**Implementation:** `release_state.rs:39-45`

```rust
pub fn clear_staged_release(cfg: &BonesConfig) -> Result<()> {
    let path = staged_release_path(cfg);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Failed to remove staged release state: {}", path.display()))?;
    }
    Ok(())
}
```

**Deletes:** `<repo_path>/bones/.staged_release`

**Purpose:** Marks the deployment as complete. No staged release exists anymore.

---

### 10. Print Success Message

**Source:** `activate_release.rs:33`

```rust
println!("Activated release: {release_name}");
```

**Example Output:**
```
Activated release: 20260507_150432
```

---

## Directory Structure After Activation

```
/srv/deployments/myapp/
├── releases/
│   ├── 20260507_130000/
│   ├── 20260507_140000/
│   └── 20260507_150432/    # New active release
├── shared/
│   ├── .env
│   └── storage/
└── current -> releases/20260507_150432/  # Switched!

/var/www/myapp -> /srv/deployments/myapp/current  # Updated!

/home/git/myapp.git/bones/
├── bones.toml
└── (no .staged_release file)  # Cleared
```

---

## Symlink Chain

After activation, the symlink chain is:

```
/var/www/myapp (web_root)
    ↓
/srv/deployments/myapp/current (current link)
    ↓
/srv/deployments/myapp/releases/20260507_150432 (actual release)
```

**Why two levels?**
1. **web_root**: User-facing path, typically in `/var/www/`
2. **current**: Deployment path, in `/srv/deployments/`
3. **Separation of concerns**: Web server uses `web_root`, deployment manages `current`

---

## Atomic Switch Benefits

### Zero Downtime

- Symlink switch is instantaneous
- No period where application is unavailable
- Requests continue to flow during activation

### No Inconsistent State

- Old release remains active until exact moment of switch
- No partial state where some processes see old release, others see new
- Filesystem guarantees atomicity

### Instant Rollback

- To rollback: switch symlink back to previous release
- Same atomic operation
- Immediate effect

---

## Validation Checks

### Pre-activation Validation

1. **Release directory exists**: Confirms deployment completed
2. **current is symlink**: Prevents overwriting a real directory

### Why These Matter

**Missing release directory:**
```
Staged release directory does not exist: /srv/deployments/myapp/releases/20260507_150432
```
- Deployment failed or didn't run
- No release to activate

**current is not symlink:**
```
current exists and is not a symlink: /srv/deployments/myapp/current
```
- Someone manually created a directory
- Would conflict with symlink creation
- Indicates configuration issue

---

## Typical Workflow

```bash
# 1. Stage release
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.toml

# 2. Check out code
git --work-tree=/srv/deployments/myapp/build/workspace \
    --git-dir=/home/git/myapp.git \
    checkout -f master

# 3. Wire shared paths
sudo bonesremote release wire --config /home/git/myapp.git/bones/bones.toml

# 4. Run deployment scripts
# (scripts populate build/workspace and releases/<release-id>/)

# 5. Activate release
bonesremote release activate --config /home/git/myapp.git/bones/bones.toml

# 6. Post-deploy (restart service, harden permissions)
sudo bonesremote hooks post-deploy --config /home/git/myapp.git/bones/bones.toml
```

---

## Error Scenarios

### No Staged Release

```
Failed to read staged release state at /home/git/myapp.git/bones/.staged_release
```

**Solution:** Run `release stage` first.

### Release Directory Missing

```
Staged release directory does not exist: /srv/deployments/myapp/releases/20260507_150432
```

**Solution:** Deployment scripts didn't complete. Check deployment logs.

### Symlink Creation Failed

```
Failed to atomically switch symlink /srv/deployments/myapp/current -> /srv/deployments/myapp/releases/20260507_150432
```

**Possible causes:**
- Permission denied
- Disk space issues
- Parent directory doesn't exist

---

## Related Commands

- `bonesremote release stage` - Stage a new release
- `bonesremote release wire` - Wire shared paths
- `bonesremote release rollback` - Rollback to previous release
- `bonesremote release drop-failed` - Clean up failed release
- `bonesremote hooks deploy` - Runs deployment and activation
- `bonesremote hooks post-deploy` - Post-activation tasks
