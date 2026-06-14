# bonesremote hooks post-deploy

## Overview

Post-deployment hook that runs after a release is activated. It prunes old releases to save disk space, keeping only the configured number of recent releases. This command does not restart services, change ownership, or harden permissions — service restart is handled by the calling hook script (`sudo bonesremote service restart`), and ownership is a provisioning-time contract established once by `bonesdeploy remote setup`.

## Command Signature

```bash
bonesremote hooks post-deploy --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

---

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `post_deploy.rs:14`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration.

---

### 2. Prune Old Releases

---

### 5. Prune Old Releases

**Source:** `post_deploy.rs:19-22`

```rust
let pruned = prune_old_releases(&cfg)?;
if !pruned.is_empty() {
    println!("Pruned releases: {}", pruned.join(", "));
}
```

**Source:** `post_deploy.rs:77-99`

#### 5.1 Get Current Release

```rust
let active_release = release_state::current_release_name(cfg)?;
let mut releases = release_state::list_releases_sorted(cfg)?;
let keep = cfg.releases.keep.max(1);
```

**Example:**
- Active release: `20260507_150000`
- All releases: `["120000", "130000", "140000", "150000"]`
- Keep: `5` (default)

#### 5.2 Remove Oldest Releases

```rust
let mut pruned = Vec::new();
while releases.len() > keep {
    let oldest = releases.remove(0);
    if oldest == active_release {
        releases.push(oldest);
        releases.sort();
        continue;
    }

    let path = release_state::release_dir(cfg, &oldest);
    if path.exists() {
        fs::remove_dir_all(&path).with_context(|| format!("Failed to prune old release {}", path.display()))?;
        pruned.push(oldest);
    }
}
```

**Process:**
1. While more releases than `keep`:
   - Get oldest release
   - Skip if it's the active release (don't delete active!)
   - Remove directory
   - Track deleted releases

**Example:**
- Releases: `["120000", "130000", "140000", "150000"]` (4 releases)
- Keep: `3`
- Pruned: `["120000"]`
- Result: `["130000", "140000", "150000"]` (3 releases)

**Active release protection:**
```rust
if oldest == active_release {
    releases.push(oldest);  // Put it back
    releases.sort();        // Re-sort
    continue;               // Skip to next iteration
}
```

Never deletes the active release, even if it's the oldest.

#### 5.3 Print Pruned Releases

```rust
println!("Pruned releases: {}", pruned.join(", "));
```

**Example Output:**
```
Pruned releases: 20260507_120000, 20260507_130000
```

Or if nothing to prune:
```
(no output)
```

---

### 6. Return Success

**Source:** `post_deploy.rs:24`

```rust
Ok(())
```

---

---

## Release Pruning Strategy

### Default Behavior

**Keep last 5 releases** (configurable):
```yaml
releases:
  keep: 5
```

### Pruning Logic

1. **Oldest first**: Always remove oldest releases
2. **Protect active**: Never delete current release
3. **Minimum 1**: Always keep at least 1 release
4. **Disk space**: Prevents accumulation of old releases

### Example Scenarios

**Scenario 1: Normal Pruning**
```
Releases: [A, B, C, D, E, F]  # 6 releases
Keep: 5
Pruned: [A]
Result: [B, C, D, E, F]
```

**Scenario 2: Active is Oldest**
```
Releases: [A*, B, C, D]  # A is active
Keep: 3
Pruned: [B]  # Skip A, prune B
Result: [A*, C, D]
```

**Scenario 3: Already at Limit**
```
Releases: [A, B, C]
Keep: 3
Pruned: []  # Nothing to prune
Result: [A, B, C]
```

---

## Typical Workflow

```bash
# 1. Stage release
bonesremote release stage --config /home/git/myapp.git/bones/bones.yaml

# 2. Checkout into build workspace (done by post-receive hook)
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.yaml --revision <sha>

# 3. Wire shared paths
bonesremote release wire --config /home/git/myapp.git/bones/bones.yaml

# 4. Deploy scripts + activate
bonesremote hooks deploy --config /home/git/myapp.git/bones/bones.yaml

# 5. Post-deploy (prune old releases)
bonesremote hooks post-deploy --config /home/git/myapp.git/bones/bones.yaml

# 6. Restart nginx (requires elevated privileges)
sudo bonesremote service restart --config /home/git/myapp.git/bones/bones.yaml
```

---

## Error Scenarios

### Systemd Service Failed

```
systemctl command failed with status 1
```

**Possible causes:**
- Service file syntax error
- User doesn't exist
- Binary path incorrect
- Permission denied

### Pruning Failed

```
Failed to prune old release /srv/sites/myapp/releases/20260507_120000
```

**Possible causes:**
- Permission denied
- Directory in use
- Disk errors

---

## Related Commands

- `bonesremote hooks deploy` - Deployment and activation
- `bonesremote release activate` - Activate release
- `bonesremote doctor` - Validate environment
