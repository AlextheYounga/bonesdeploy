# bonesremote release rollback

## Overview

Reverts to the previous release by repointing the `current` symlink to the chronologically older release. This provides an instant, zero-downtime rollback mechanism to recover from problematic deployments without requiring a new deployment.

## Command Signature

```bash
bonesremote release rollback --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.toml` configuration file (required)

**Note:** Must NOT be run as root (runs as deploy user).

---

## Detailed Execution Steps

### 1. Verify Not Running as Root

**Source:** `rollback.rs:10`

```rust
privileges::ensure_not_root("bonesremote release rollback")?;
```

Ensures the command is not running as root. Rollback should be done as the deploy user.

---

### 2. Load Configuration

**Source:** `rollback.rs:12`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration to determine paths.

---

### 3. List All Releases

**Source:** `rollback.rs:13`

```rust
let releases = release_state::list_releases_sorted(&cfg)?;
```

**Implementation:** `release_state.rs:87-105`

```rust
pub fn list_releases_sorted(cfg: &BonesConfig) -> Result<Vec<String>> {
    let releases_dir = releases_dir(cfg);
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&releases_dir)
        .with_context(|| format!("Failed to read releases dir: {}", releases_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    names.sort();
    Ok(names)
}
```

**Process:**
1. List all directories in `/srv/deployments/myapp/releases/`
2. Filter to only directories
3. Sort chronologically (oldest first)

**Example Result:**
```
["20260507_120000", "20260507_130000", "20260507_140000", "20260507_150000"]
```

---

### 4. Validate Minimum Releases

**Source:** `rollback.rs:14-16`

```rust
if releases.len() < 2 {
    bail!("Need at least two releases to perform rollback");
}
```

**Why 2?** Rollback requires:
1. Current release (to roll back from)
2. Previous release (to roll back to)

**Example Error:**
```
Need at least two releases to perform rollback
```

---

### 5. Identify Current Release

**Source:** `rollback.rs:18-22`

```rust
let current_name = release_state::current_release_name(&cfg)?;
let current_idx = releases
    .iter()
    .position(|name| name == &current_name)
    .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;
```

#### 5.1 Read Current Release Name

**Implementation:** `release_state.rs:79-85`

```rust
pub fn current_release_name(cfg: &BonesConfig) -> Result<String> {
    let current_release = current_release_dir(cfg)?;
    current_release
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve current release name from {}", current_release.display()))
}
```

**Process:**
1. Read `current` symlink: `/srv/deployments/myapp/current`
2. Resolve to actual directory: `/srv/deployments/myapp/releases/20260507_150000`
3. Extract directory name: `20260507_150000`

#### 5.2 Find Current Index

**Example:**
- Current release: `20260507_150000`
- Sorted releases: `["120000", "130000", "140000", "150000"]`
- Current index: `3` (0-indexed)

---

### 6. Validate Can Rollback

**Source:** `rollback.rs:24-26`

```rust
if current_idx == 0 {
    bail!("Current release is already the oldest release; cannot roll back");
}
```

**If current release is the oldest (index 0), there's nothing to roll back to.**

**Example Error:**
```
Current release is already the oldest release; cannot roll back
```

**Note:** This means you can't roll back to a release that was already pruned.

---

### 7. Identify Previous Release

**Source:** `rollback.rs:28`

```rust
let previous_name = releases[current_idx - 1].clone();
```

Gets the previous release by subtracting 1 from current index.

**Example:**
- Current index: `3`
- Previous index: `2`
- Previous release: `20260507_140000`

---

### 8. Switch Current Symlink

**Source:** `rollback.rs:29-31`

```rust
let previous_dir = release_state::release_dir(&cfg, &previous_name);
let current_link = release_state::current_link(&cfg);
release_state::point_symlink_atomically(&current_link, &previous_dir)?;
```

**Process:**
1. Define previous release directory: `/srv/deployments/myapp/releases/20260507_140000`
2. Define current link path: `/srv/deployments/myapp/current`
3. Atomically switch symlink: `current` → `20260507_140000`

**Atomic Operation:** (same as activation)
1. Create temp symlink
2. Rename to `current` (atomic)

**Result:** Instant switch to previous release.

---

### 9. Print Rollback Result

**Source:** `rollback.rs:33`

```rust
println!("Rollback complete: {current_name} -> {previous_name}");
```

**Example Output:**
```
Rollback complete: 20260507_150000 -> 20260507_140000
```

---

## Directory State Before Rollback

```
/srv/deployments/myapp/
├── releases/
│   ├── 20260507_120000/
│   ├── 20260507_130000/
│   ├── 20260507_140000/    # Previous (to rollback to)
│   └── 20260507_150000/    # Current (has bug)
└── current -> releases/20260507_150000/

/var/www/myapp -> /srv/deployments/myapp/current
```

## Directory State After Rollback

```
/srv/deployments/myapp/
├── releases/
│   ├── 20260507_120000/
│   ├── 20260507_130000/
│   ├── 20260507_140000/    # Now active
│   └── 20260507_150000/    # Still exists, not active
└── current -> releases/20260507_140000/  # Switched!

/var/www/myapp -> /srv/deployments/myapp/current
```

**Note:** The problematic release (`150000`) is NOT deleted. It remains available for:
- Debugging
- Future rollback forward
- Manual inspection

---

## Rollback Scenarios

### Scenario 1: Bug in Latest Release

```
Releases: [A, B, C, D]  # D is current
Current: D (has critical bug)
Rollback: D → C
Result: [A, B, C, D]  # C is now current
```

### Scenario 2: Multiple Rollbacks

```
Releases: [A, B, C, D]
Current: D
Rollback: D → C
Current: C
Rollback: C → B
Current: B
```

### Scenario 3: Cannot Rollback (Oldest)

```
Releases: [A, B]
Current: A (oldest)
Error: Current release is already the oldest release; cannot roll back
```

### Scenario 4: Cannot Rollback (Only One)

```
Releases: [A]
Current: A
Error: Need at least two releases to perform rollback
```

---

## Important Considerations

### Database Migrations

**Rollback does NOT reverse database migrations.**

If the latest deployment included a database migration:
- Code rolls back to previous version
- Database schema remains migrated
- Previous code may be incompatible with new schema

**Solutions:**
1. Design backward-compatible migrations
2. Write rollback migration scripts
3. Test migrations thoroughly before deployment

### Shared Files

Shared files (`.env`, `storage/`) are unaffected by rollback:
- Same files accessible in both releases
- No data loss
- Configuration persists

### Service Restart

The application service may need a restart after rollback:
- If application caches the release path
- If running processes need to reload code

**Check service:**
```bash
sudo systemctl status myapp
# May need:
sudo systemctl restart myapp
```

---

## When to Use

1. **Critical bug discovered**: Immediate mitigation
2. **Performance regression**: Restore previous performance
3. **Failed feature**: Disable problematic feature quickly
4. **Configuration error**: Revert to working configuration
5. **Security vulnerability**: Switch to known-safe version

---

## Rollback vs. Redeploy

| Aspect | Rollback | Redeploy |
|--------|----------|----------|
| **Speed** | Instant | Minutes |
| **Downtime** | None | Minimal |
| **Database** | Not changed | Depends on migrations |
| **Risk** | Low (known state) | Medium (new deployment) |
| **Use case** | Emergency | Planned fix |

---

## Typical Workflow

### Emergency Rollback

```bash
# 1. Issue discovered in production
# 2. Immediate rollback
bonesremote release rollback --config /home/git/myapp.git/bones/bones.toml

# 3. Verify service restored
curl https://app.example.com/health

# 4. Investigate issue
# 5. Fix in development
# 6. Test fix
# 7. Deploy new version when ready
```

### Planned Rollback

```bash
# 1. Monitor release performance
# 2. Decide to rollback
# 3. Announce to team
# 4. Execute rollback
bonesremote release rollback --config /home/git/myapp.git/bones/bones.toml

# 5. Verify application
# 6. Monitor for issues
```

---

## Related Commands

- `bonesremote release activate` - Activate a specific release
- `bonesremote release stage` - Stage a new release
- `bonesremote hooks post-deploy` - Post-deployment tasks
- `bonesdeploy rollback` - Client-side rollback command
