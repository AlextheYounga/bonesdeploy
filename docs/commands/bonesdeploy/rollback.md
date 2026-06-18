# bonesdeploy rollback

## Overview

Reverts the currently active release to the previous release by repointing the `current` symlink on the remote server. This is a fast, atomic operation that immediately switches the live application back to the previous version without requiring a new deployment. Useful for quickly recovering from a failed deployment or problematic release.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `rollback.rs:10-11`

```rust
let bones_toml = Path::new(config::Constants::BONES_TOML);
let cfg = config::load(bones_toml)?;
```

Loads deployment configuration to determine:
- Remote server connection details
- Git directory path
- Project name

---

### 2. Construct Remote Config Path

**Source:** `rollback.rs:13`

```rust
let remote_bones_toml = format!("{}/{}/bones.toml", cfg.data.repo_path, config::Constants::REMOTE_BONES_DIR);
```

Builds the path to `bones.toml` on the remote server:
```
<repo_path>/bones/bones.toml
```

**Example:** `/home/git/myapp.git/bones/bones.toml`

---

### 3. Print Rollback Header

**Source:** `rollback.rs:15`

```rust
println!("Rolling back {} on {}...", style(&cfg.data.project_name).cyan().bold(), style(&cfg.data.host).cyan());
```

Displays the project name and target host.

---

### 4. Establish SSH Connection

**Source:** `rollback.rs:17`

```rust
let session = ssh::connect(&cfg).await?;
```

Opens an SSH session to the remote server.

---

### 5. Execute Remote Rollback Command

**Source:** `rollback.rs:18-19`

```rust
let command = format!("bonesremote release rollback --config '{remote_bones_toml}'");
ssh::stream_cmd(&session, &command).await?;
```

Executes `bonesremote release rollback` on the remote server and streams the output back to the local terminal.

**Command Executed:**
```bash
bonesremote release rollback --config '/home/git/myapp.git/bones/bones.toml'
```

---

### 6. Close SSH Session

**Source:** `rollback.rs:20`

```rust
session.close().await?;
```

Cleanly closes the SSH connection.

---

### 7. Print Success Message

**Source:** `rollback.rs:22`

```rust
println!("\n{} Rollback complete.", style("Done!").green().bold());
```

---

## What `bonesremote release rollback` Does

**Source:** `bonesremote/src/commands/rollback.rs`

### 1. Verify Not Running as Root

**Source:** `rollback.rs:10`

```rust
privileges::ensure_not_root("bonesremote release rollback")?;
```

Ensures the command is not being run as root (security best practice).

---

### 2. Load Remote Configuration

**Source:** `rollback.rs:12`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads `bones.toml` from the bare repository.

---

### 3. List All Releases

**Source:** `rollback.rs:13`

```rust
let releases = release_state::list_releases_sorted(&cfg)?;
```

Lists all release directories in `<project_root>/releases/`, sorted chronologically (oldest first).

**Example:**
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

Rollback requires at least 2 releases: the current one and the previous one.

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

Finds the current release by:
1. Reading the `current` symlink
2. Extracting the release name
3. Finding its position in the sorted releases list

---

### 6. Validate Can Rollback

**Source:** `rollback.rs:24-26`

```rust
if current_idx == 0 {
    bail!("Current release is already the oldest release; cannot roll back");
}
```

If the current release is the oldest (index 0), there's nothing to roll back to.

---

### 7. Switch to Previous Release

**Source:** `rollback.rs:28-31`

```rust
let previous_name = releases[current_idx - 1].clone();
let previous_dir = release_state::release_dir(&cfg, &previous_name);
let current_link = release_state::current_link(&cfg);
release_state::point_symlink_atomically(&current_link, &previous_dir)?;
```

**Process:**
1. Gets the previous release name (current index - 1)
2. Constructs path to previous release directory
3. Atomically switches the `current` symlink to point to previous release

**Atomic Symlink Switch** (`release_state.rs:107-130`):
```rust
pub fn point_symlink_atomically(link_path: &Path, target_path: &Path) -> Result<()> {
    // Create temp symlink
    let temp_link = parent.join(format!(".tmp_current_{}_{}", process::id(), nanos));
    symlink(target_path, &temp_link)?;
    
    // Atomically rename (replaces old symlink)
    fs::rename(&temp_link, link_path)?;
    
    Ok(())
}
```

**Why atomic?** The symlink switch is atomic, ensuring:
- No downtime during the switch
- No race conditions where `current` points to nothing
- Instant rollback

---

### 8. Print Rollback Result

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
│   ├── 20260507_130000/
│   ├── 20260507_140000/
│   └── 20260507_150000/  # Current (has bug)
└── current -> releases/20260507_150000/

/var/www/myapp -> /srv/deployments/myapp/current
```

## Directory State After Rollback

```
/srv/deployments/myapp/
├── releases/
│   ├── 20260507_130000/
│   ├── 20260507_140000/  # Now active
│   └── 20260507_150000/  # Still exists, just not active
└── current -> releases/20260507_140000/  # Switched!

/var/www/myapp -> /srv/deployments/myapp/current
```

**Note:** The failed/problematic release is not deleted. It remains available for:
- Inspection and debugging
- Future rollback forward (if needed)
- Manual cleanup

---

## When to Use

1. **Failed deployment**: Quickly revert to previous working version
2. **Critical bug discovered**: Immediate mitigation while fixing the issue
3. **Performance regression**: Switch back to previous version
4. **Configuration error**: Revert to last known good configuration
5. **Testing validation**: Rollback after testing a new release

---

## Rollback vs. Redeploy

| Aspect | Rollback | Redeploy |
|--------|----------|----------|
| **Speed** | Instant (symlink switch) | Slow (full deployment) |
| **Data preserved** | Yes (shared paths) | Yes (shared paths) |
| **Database** | No changes | Depends on migrations |
| **Reversible** | Yes (can roll forward) | Yes (deploy new version) |
| **Use case** | Emergency reversion | Fix and redeploy |

---

## Rollback Workflow

### Scenario: Deployment Failed

```bash
# Deploy new version
git push production master

# Deployment fails (e.g., migration error)
# Application is down

# Quick rollback to restore service
bonesdeploy rollback

# Fix the issue locally
# ...

# Test the fix
# ...

# Redeploy when ready
git push production master
```

### Scenario: Bug in Production

```bash
# Bug discovered in production

# Immediate rollback to restore service
bonesdeploy rollback

# Verify service is restored
curl https://app.example.com/health

# Debug the issue
ssh git@app.example.com
cd /srv/deployments/myapp/releases/20260507_150000
# Investigate...

# Fix and redeploy
git push production master
```

---

## Limitations

1. **Cannot roll back if only one release exists**
   ```
   Need at least two releases to perform rollback
   ```

2. **Cannot roll back from oldest release**
   ```
   Current release is already the oldest release; cannot roll back
   ```

3. **No automatic service restart**: If your application caches the release path, you may need to restart the service manually
   ```bash
   ssh git@app.example.com
   sudo systemctl restart myapp
   ```

4. **Database migrations not rolled back**: Rollback only switches code, not database schema
   - Consider migration rollback scripts if needed
   - Or design migrations to be backward-compatible

---

## Post-Rollback Actions

After rolling back, you may need to:

1. **Verify service is running**:
   ```bash
   ssh git@app.example.com
   sudo systemctl status myapp
   ```

2. **Check application logs**:
   ```bash
   ssh git@app.example.com
   journalctl -u myapp -f
   ```

3. **Monitor for issues**:
   - Check error rates
   - Monitor performance
   - Verify user functionality

4. **Investigate the failed release**:
   ```bash
   ssh git@app.example.com
   cd /srv/deployments/myapp/releases/20260507_150000
   # Check deployment logs, app logs, etc.
   ```

---

## Related Commands

- `bonesdeploy deploy` - Deploy current version
- `bonesdeploy manage` - Open management TUI
- `bonesdeploy push` - Sync configuration
- `bonesremote release rollback` - Server-side rollback command
- `bonesremote release activate` - Activate a specific release
