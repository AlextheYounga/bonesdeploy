# bonesremote hooks post-deploy

## Overview

Post-deployment hook that runs after a release is activated. It restarts the per-site nginx service to pick up the new release, hardens file permissions to restrict access, and prunes old releases to save disk space. This command must be run as root to manage systemd services and change file ownership.

## Command Signature

```bash
sudo bonesremote hooks post-deploy --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

**Note:** Must be run as root (via sudo).

---

## Detailed Execution Steps

### 1. Verify Root Privileges

**Source:** `post_deploy.rs:13`

```rust
privileges::ensure_root("bonesremote hooks post-deploy")?;
```

Ensures the command is running as root. Required for:
- Managing systemd services
- Changing file ownership
- Hardening permissions

---

### 2. Load Configuration

**Source:** `post_deploy.rs:15`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration.

---

### 3. Restart Per-Site Nginx

**Source:** `post_deploy.rs:16`

```rust
restart_site_nginx(&cfg)?;
```

Restarts the per-site nginx service if it's running, so it picks up the new release.

**Source:** `post_deploy.rs:27-47`

#### 3.1 Check Service Status

```rust
let service_name = format!("{}-nginx", cfg.data.project_name);
let status = Command::new("systemctl")
    .args(["is-active", "--quiet", &service_name])
    .status()
    .context("Failed to check nginx service status")?;
```

Checks if the per-site nginx service is currently active.

**Service name:** `{project_name}-nginx` (e.g., `myapp-nginx`)

#### 3.2 Restart if Active

```rust
if status.success() {
    let restart_status = Command::new("systemctl")
        .args(["restart", &service_name])
        .status()
        .context("Failed to restart nginx service")?;

    if !restart_status.success() {
        bail!("Failed to restart {service_name} service");
    }
    println!("Restarted {service_name} service");
}
```

If the service is active, restarts it. This causes nginx to:
1. Stop serving the old release
2. Re-read the `current` symlink (now pointing to the new release)
3. Start serving the new release

**Why restart instead of reload?**
- Ensures nginx picks up the new release via `current`
- Cleaner than trying to reload with changed paths
- Minimal downtime (typically < 1 second)

#### 3.3 Skip if Not Active

If the service isn't running yet (e.g., first deployment), the restart is skipped. The service will be started during the initial `remote setup`.

---

### 4. Harden Active Release Permissions

**Source:** `post_deploy.rs:17`

```rust
permissions::harden_active_release(&cfg)?;
```

Restricts permissions on the active release to the service user.

**Typical actions:**
1. Change ownership to `service_user:group`
2. Set directory permissions to `dir_mode` (default: `750`)
3. Set file permissions to `file_mode` (default: `640`)
4. Apply any path-specific overrides from `permissions.paths`

**Why harden?**
- Deploy user (`git`) has write access during deployment
- Service user (`myapp`) should own the release files
- Restricts access to application code and data
- Security best practice

**After hardening:**
- Service user can read/write files
- Group can read files (e.g., web server for static files)
- Others have no access

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

## systemd Service Management

### Service Lifecycle

1. **First deployment**
   - Service file created
   - Service enabled (auto-start on boot)
   - Service started immediately

2. **Subsequent deployments**
   - Service file updated (if needed)
   - Service restarted (via `Restart=always` on crash)
   - Service continues running

3. **Service monitoring**
   ```bash
   sudo systemctl status myapp
   sudo journalctl -u myapp -f
   ```

### Service Configuration

**From `bones.yaml`:**
```yaml
runtime:
  command:
    - /usr/bin/node
    - dist/server.js
  working_dir: .
  writable_paths: []

permissions:
  defaults:
    service_user: myapp
    group: www-data

data:
  project_name: myapp
  web_root: public
  repo_path: /home/git/myapp.git
```

**Generated service:**
- User: `myapp`
- WorkingDirectory: `/srv/deployments/myapp/current/public`
- ExecStart: `bonesremote landlock exec --config /home/git/myapp.git/bones/bones.yaml`

---

## Permission Hardening

### Ownership Changes

**Before hardening** (post-deploy):
- Owner: `git:git` (deploy user created files)
- Permissions: Default umask

**After hardening:**
- Owner: `myapp:www-data` (service user owns files)
- Directories: `750` (rwxr-x---)
- Files: `640` (rw-r-----)

### Path-Specific Overrides

**Configuration:**
```yaml
permissions:
  paths:
    - path: storage/logs
      mode: "770"
      recursive: true
    - path: bootstrap/cache
      mode: "775"
      recursive: true
```

Allows custom permissions for specific paths.

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
# 1. Deployment triggered
git push production master

# 2. Staging
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.yaml

# 3. Checkout
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.yaml

# 4. Deploy
bonesremote hooks deploy --config /home/git/myapp.git/bones/bones.yaml

# 5. Post-deploy ← YOU ARE HERE
sudo bonesremote hooks post-deploy --config /home/git/myapp.git/bones/bones.yaml
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

### Permission Hardening Failed

```
Failed to chown path: /srv/deployments/myapp/releases/20260507_150000
```

**Possible causes:**
- User or group doesn't exist
- Not running as root
- Path doesn't exist

### Pruning Failed

```
Failed to prune old release /srv/deployments/myapp/releases/20260507_120000
```

**Possible causes:**
- Permission denied
- Directory in use
- Disk errors

---

## Related Commands

- `bonesremote hooks deploy` - Deployment and activation
- `bonesremote landlock exec` - Runtime sandbox
- `bonesremote release activate` - Activate release
- `bonesremote doctor` - Validate environment
