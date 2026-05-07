# bonesremote hooks post-deploy

## Overview

Post-deployment hook that runs after a release is activated. It ensures the runtime service is configured and running, hardens file permissions to restrict access, and prunes old releases to save disk space. This command must be run as root to manage systemd services and change file ownership.

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

### 3. Ensure Runtime Service

**Source:** `post_deploy.rs:16`

```rust
ensure_runtime_service(&cfg)?;
```

If `runtime.command` is configured, ensures the systemd service is set up and running.

**Source:** `post_deploy.rs:27-42`

#### 3.1 Skip if No Runtime

```rust
if cfg.runtime.command.is_empty() {
    return Ok(());
}
```

If no runtime command is configured, skips service setup. The application is assumed to be static files or managed externally.

#### 3.2 Render Service Unit File

**Source:** `post_deploy.rs:32`, `post_deploy.rs:44-53`

```rust
let service_body = render_runtime_service(cfg);
```

**Implementation:**
```rust
fn render_runtime_service(cfg: &config::BonesConfig) -> String {
    let runtime_config_path = format!("{}/bones/bones.yaml", cfg.data.git_dir);
    format!(
        "[Unit]\n\
         Description=Bones runtime for {service_name}\n\
         After=network.target\n\
         \n\
         [Service]\n\
         Type=simple\n\
         User={service_user}\n\
         WorkingDirectory={working_directory}\n\
         ExecStart=/usr/local/bin/bonesremote landlock exec --config {runtime_config_path}\n\
         Restart=always\n\
         RestartSec=2\n\
         \n\
         [Install]\n\
         WantedBy=multi-user.target\n",
        service_name = cfg.data.project_name,
        service_user = cfg.permissions.defaults.service_user,
        working_directory = cfg.data.live_root,
        runtime_config_path = runtime_config_path,
    )
}
```

**Example Service File:**
```ini
[Unit]
Description=Bones runtime for myapp
After=network.target

[Service]
Type=simple
User=myapp
WorkingDirectory=/var/www/myapp
ExecStart=/usr/local/bin/bonesremote landlock exec --config /home/git/myapp.git/bones/bones.yaml
Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target
```

**Key Configuration:**
- **User**: Service user (e.g., `myapp`)
- **WorkingDirectory**: `live_root` (symlink to current release)
- **ExecStart**: Launches runtime via Landlock sandbox
- **Restart**: Automatically restart on failure
- **RestartSec**: Wait 2 seconds before restart

#### 3.3 Write Service File

**Source:** `post_deploy.rs:33-34`, `post_deploy.rs:55-65`

```rust
let service_path = format!("/etc/systemd/system/{}.service", cfg.data.project_name);
let changed = write_file_if_changed(Path::new(&service_path), &service_body)?;
```

**Implementation:**
```rust
fn write_file_if_changed(path: &Path, contents: &str) -> Result<bool> {
    if path.exists() {
        let existing = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        if existing == contents {
            return Ok(false);  // No changes needed
        }
    }

    fs::write(path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}
```

**Only writes if changed:**
- Compares existing file with new content
- Returns `true` if file was written
- Returns `false` if already up to date

**Why check for changes?**
- Avoid unnecessary daemon-reload
- Preserve file timestamps
- More efficient

#### 3.4 Reload Systemd (if needed)

**Source:** `post_deploy.rs:36-38`

```rust
if changed {
    run_systemctl(["daemon-reload"])?;
}
```

If service file was modified, reloads systemd to pick up changes.

#### 3.5 Enable and Start Service

**Source:** `post_deploy.rs:40`

```rust
run_systemctl(["enable", "--now", &cfg.data.project_name])?;
```

**Command:** `systemctl enable --now myapp`

**Flags:**
- `enable`: Enable service to start on boot
- `--now`: Also start the service immediately

**Result:** Service is running and will auto-start on reboot.

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
- Service user (`myapp`) should own the runtime files
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
  live_root: /var/www/myapp
  git_dir: /home/git/myapp.git
```

**Generated service:**
- User: `myapp`
- WorkingDirectory: `/var/www/myapp`
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
Failed to chown path: /srv/deployments/myapp/runtime/20260507_150000
```

**Possible causes:**
- User or group doesn't exist
- Not running as root
- Path doesn't exist

### Pruning Failed

```
Failed to prune old release /srv/deployments/myapp/runtime/20260507_120000
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
