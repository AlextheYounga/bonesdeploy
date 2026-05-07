# bonesremote landlock exec

## Overview

Launches the configured runtime command within a Landlock sandbox that restricts filesystem access to only the necessary directories. This provides mandatory access control (MAC) isolation, preventing the application from accessing or modifying files outside its designated workspace. The command is typically invoked by systemd as part of the runtime service.

## Command Signature

```bash
bonesremote landlock exec --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

---

## Detailed Execution Steps

### 1. Verify Not Running as Root

**Source:** `landlock_exec.rs:16`

```rust
privileges::ensure_not_root("bonesremote landlock exec")?;
```

Ensures the command is not being run as root. The runtime should execute as the service user, not root, for security reasons.

**Error if root:**
```
bonesremote landlock exec must not be run as root
```

---

### 2. Load Configuration

**Source:** `landlock_exec.rs:18-21`

```rust
let cfg = config::load(Path::new(config_path))?;
if cfg.runtime.command.is_empty() {
    bail!("runtime.command must be configured before starting runtime with landlock");
}
```

Loads `bones.yaml` and validates that a runtime command is configured.

**Required Configuration:**
```yaml
runtime:
  command:
    - /usr/bin/node
    - server.js
  working_dir: .
  writable_paths: []
```

---

### 3. Resolve Active Runtime Root

**Source:** `landlock_exec.rs:23-24`

```rust
let active_runtime_root = fs::canonicalize(&cfg.data.live_root)
    .with_context(|| format!("Failed to resolve live_root: {}", cfg.data.live_root))?;
```

Resolves the `live_root` symlink to the actual release directory.

**Example:**
- `live_root`: `/var/www/myapp` (symlink)
- `active_runtime_root`: `/srv/deployments/myapp/runtime/20260507_150000` (resolved)

**Why canonicalize?**
- Resolves symlinks to get the actual path
- Ensures absolute path (no relative path ambiguity)
- Validates the directory exists

---

### 4. Resolve Shared Root

**Source:** `landlock_exec.rs:25`

```rust
let shared_root = release_state::shared_dir(&cfg);
```

Gets the path to the shared directory.

**Example:** `/srv/deployments/myapp/shared`

This directory contains files shared across all releases:
- `.env` - Environment configuration
- `storage/` - User uploads, logs, etc.
- Any other paths configured in `releases.shared_paths`

---

### 5. Resolve Command Path

**Source:** `landlock_exec.rs:26`

```rust
let command_path = landlock::resolve_command_path(&cfg.runtime.command[0])?;
```

Resolves the absolute path to the executable.

**Example:**
- Configured: `/usr/bin/node`
- Resolved: `/usr/bin/node` (already absolute)

- Configured: `node`
- Resolved: `/usr/bin/node` (found via PATH)

**Why resolve?**
- Need absolute path for Landlock policy
- Ensures the binary exists
- Prevents PATH manipulation attacks

---

### 6. Resolve Working Directory

**Source:** `landlock_exec.rs:28`, `landlock_exec.rs:43-55`

```rust
let working_dir = resolve_working_dir(&cfg.runtime.working_dir, &active_runtime_root)?;
```

#### 6.1 Resolve Relative vs Absolute

```rust
fn resolve_working_dir(working_dir: &str, runtime_root: &Path) -> Result<PathBuf> {
    let candidate =
        if Path::new(working_dir).is_absolute() { 
            PathBuf::from(working_dir) 
        } else { 
            runtime_root.join(working_dir) 
        };
```

**Examples:**
- `working_dir: "."` → `/srv/deployments/myapp/runtime/20260507_150000`
- `working_dir: "/app"` → `/app` (absolute)
- `working_dir: "subdir"` → `/srv/deployments/myapp/runtime/20260507_150000/subdir`

#### 6.2 Validate Within Runtime Root

```rust
    let resolved = fs::canonicalize(&candidate)
        .with_context(|| format!("Failed to resolve runtime working_dir {}", candidate.display()))?;

    if !resolved.starts_with(runtime_root) {
        bail!("runtime.working_dir resolves outside active runtime root: {}", resolved.display());
    }

    Ok(resolved)
}
```

**Security Check:** Ensures the working directory is within the runtime root, preventing directory traversal attacks.

---

### 7. Build Landlock Policy

**Source:** `landlock_exec.rs:29`, `landlock_exec.rs:57-90`

```rust
let policy = build_policy(&cfg, &active_runtime_root, &shared_root, &command_path)?;
```

Constructs the Landlock access control policy.

#### 7.1 Define Read-Only Paths

**Source:** `landlock_exec.rs:63-72`

```rust
let mut read_only_paths = BTreeSet::new();
read_only_paths.insert(runtime_root.to_path_buf());

if let Some(parent) = command_path.parent() {
    read_only_paths.insert(parent.to_path_buf());
}

for system_path in landlock::default_system_read_paths() {
    read_only_paths.insert(system_path);
}
```

**Read-only access granted to:**
1. **Runtime root** - Application code and assets
2. **Command parent directory** - Binary directory (e.g., `/usr/bin`)
3. **System paths** - Essential system directories:
   - `/lib`, `/lib64` - Shared libraries
   - `/usr/lib` - More libraries
   - `/etc/ssl`, `/etc/pki` - SSL certificates
   - `/usr/share` - Shared data
   - `/proc`, `/sys` - System information (read-only)

**Why read-only?**
- Application can read code, libraries, and configuration
- Cannot modify runtime files (prevents corruption)
- Cannot tamper with system binaries

#### 7.2 Define Writable Paths

**Source:** `landlock_exec.rs:74-84`

```rust
let mut writable_paths = BTreeSet::new();

if shared_root.exists() {
    let resolved_shared_root = fs::canonicalize(shared_root)
        .with_context(|| format!("Failed to resolve shared root {}", shared_root.display()))?;
    writable_paths.insert(resolved_shared_root);
}

for additional_root in &cfg.runtime.writable_paths {
    writable_paths.insert(resolve_additional_writable_root(additional_root, runtime_root)?);
}
```

**Writable access granted to:**
1. **Shared directory** - User uploads, logs, temp files
   - `.env` can be modified
   - `storage/` can hold user-generated content
2. **Additional writable paths** - Configured in `runtime.writable_paths`

**Example Configuration:**
```yaml
runtime:
  writable_paths:
    - /var/log/myapp
    - /tmp/myapp
```

#### 7.3 Construct Policy Object

```rust
Ok(landlock::Policy {
    read_only_paths: read_only_paths.into_iter().collect(),
    writable_paths: writable_paths.into_iter().collect(),
})
```

---

### 8. Apply Landlock Restrictions

**Source:** `landlock_exec.rs:31`

```rust
landlock::restrict_self(&policy)?;
```

Applies the Landlock policy to the current process.

**What this does:**
1. Creates Landlock ruleset with filesystem access rules
2. Enables the ruleset for the current thread
3. All future filesystem operations are restricted

**After this point, the process can only:**
- Read from: runtime root, system paths, command directory
- Write to: shared directory, configured writable paths
- All other filesystem access is denied

**This restriction is inherited by child processes**, so the runtime command is also sandboxed.

---

### 9. Change Working Directory

**Source:** `landlock_exec.rs:33-34`

```rust
env::set_current_dir(&working_dir)
    .with_context(|| format!("Failed to change working directory to {}", working_dir.display()))?;
```

Changes to the configured working directory before executing the runtime command.

**Example:** Changes to `/srv/deployments/myapp/runtime/20260507_150000`

---

### 10. Execute Runtime Command

**Source:** `landlock_exec.rs:36-40`

```rust
let mut command = Command::new(&cfg.runtime.command[0]);
command.args(cfg.runtime.command.iter().skip(1));

let exec_error = command.exec();
bail!("Failed to exec runtime command {:?}: {exec_error}", cfg.runtime.command)
```

#### 10.1 Build Command

Creates a `Command` with the configured binary and arguments.

**Example:**
```yaml
runtime:
  command:
    - /usr/bin/node
    - server.js
    - --port
    - "3000"
```

**Command:** `/usr/bin/node server.js --port 3000`

#### 10.2 Execute (exec)

Uses `.exec()` instead of `.status()`:
- Replaces current process with the new command
- No fork, just exec
- Landlock restrictions persist
- Command runs as the same user (service user)

**Why exec?**
- More efficient (no fork overhead)
- Process becomes the application (proper PID for systemd)
- Simpler process tree

#### 10.3 Error Handling

If `exec` fails, it returns an error (exec only returns on failure).

**Common failures:**
- Binary not found
- Permission denied
- Missing libraries

---

## Security Model

### What Landlock Prevents

1. **Unauthorized file access**
   - Cannot read `/etc/shadow`
   - Cannot read other users' home directories
   - Cannot read other applications' data

2. **Unauthorized file modification**
   - Cannot modify system configuration
   - Cannot modify runtime code
   - Cannot modify other applications' files

3. **Privilege escalation**
   - Cannot modify sudoers
   - Cannot modify systemd services
   - Cannot install system packages

### What Landlock Allows

1. **Reading application code** - Runtime root (read-only)
2. **Reading system libraries** - `/lib`, `/usr/lib`, etc. (read-only)
3. **Reading SSL certificates** - `/etc/ssl` (read-only)
4. **Writing application data** - Shared directory (read-write)
5. **Writing logs** - Configured writable paths (read-write)

### Defense in Depth

Landlock provides an additional security layer:

1. **User isolation** - Service user has limited system permissions
2. **Landlock** - Kernel-enforced filesystem restrictions
3. **Application-level security** - Application's own auth/authorization

Even if the application is compromised:
- Cannot read sensitive system files
- Cannot modify system configuration
- Cannot access other applications' data

---

## Process Isolation Example

**Before Landlock:**
```
Process can access:
  ✅ /srv/deployments/myapp/runtime/20260507_150000
  ✅ /var/www/myapp
  ✅ /etc/ssl
  ✅ /etc/shadow ❌ (should not be accessible)
  ✅ /home/other-user ❌ (should not be accessible)
  ✅ /srv/other-app ❌ (should not be accessible)
```

**After Landlock:**
```
Process can access:
  ✅ /srv/deployments/myapp/runtime/20260507_150000 (read-only)
  ✅ /srv/deployments/myapp/shared (read-write)
  ✅ /etc/ssl (read-only)
  ✅ /lib, /usr/lib (read-only)
  ❌ /etc/shadow (denied)
  ❌ /home/other-user (denied)
  ❌ /srv/other-app (denied)
```

---

## Systemd Integration

Typically invoked via systemd service:

**Service Unit:** `/etc/systemd/system/myapp.service`
```ini
[Service]
Type=simple
User=myapp
WorkingDirectory=/var/www/myapp
ExecStart=/usr/local/bin/bonesremote landlock exec --config /home/git/myapp.git/bones/bones.yaml
Restart=always
RestartSec=2
```

**Execution Flow:**
1. systemd starts service as `myapp` user
2. Runs `bonesremote landlock exec`
3. Landlock restrictions applied
4. Runtime command executed in sandbox
5. Service is now isolated

---

## Configuration Example

**bones.yaml:**
```yaml
runtime:
  command:
    - /usr/bin/node
    - dist/server.js
  working_dir: .
  writable_paths:
    - /var/log/myapp
    - /tmp/myapp-cache

releases:
  shared_paths:
    - .env
    - storage
    - logs

data:
  live_root: /var/www/myapp
  deploy_root: /srv/deployments/myapp
```

**Resulting Policy:**
- **Read-only:** `/srv/deployments/myapp/runtime/20260507_150000`
- **Writable:** `/srv/deployments/myapp/shared`, `/var/log/myapp`, `/tmp/myapp-cache`

---

## Troubleshooting

### Landlock Not Supported

```
Landlock support check failed: Kernel does not support Landlock
```

**Solution:** Upgrade to Linux kernel 5.13+ or enable Landlock in kernel config.

### Permission Denied

```
Failed to exec runtime command: Permission denied
```

**Possible causes:**
- Binary not executable
- Wrong user permissions
- Landlock policy too restrictive

### File Access Denied in Application

Application logs show "Permission denied" errors.

**Solution:** Add needed paths to `runtime.writable_paths` or check `shared_paths` configuration.

---

## Related Commands

- `bonesremote doctor` - Check Landlock support
- `bonesremote hooks post-deploy` - Creates systemd service
- `bonesdeploy site setup` - Provisions server with Landlock support
