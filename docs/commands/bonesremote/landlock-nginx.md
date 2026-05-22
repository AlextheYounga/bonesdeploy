# bonesremote landlock nginx

## Overview

Launches per-site nginx within a Landlock sandbox that restricts filesystem access to only the site's release directory and socket directory. This provides mandatory access control (MAC) isolation, preventing nginx from accessing files outside its designated workspace. The command is invoked by systemd as part of the per-site nginx service.

## Command Signature

```bash
bonesremote landlock nginx --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

---

## Detailed Execution Steps

### 1. Verify Not Running as Root

**Source:** `landlock_nginx.rs:14`

```rust
privileges::ensure_not_root("bonesremote landlock nginx")?;
```

Ensures the command is not being run as root. Nginx should execute as the service user for security reasons.

**Error if root:**
```
bonesremote landlock nginx must not be run as root
```

---

### 2. Load Configuration

**Source:** `landlock_nginx.rs:16`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads `bones.yaml` to get site configuration.

---

### 3. Resolve Active Runtime Root

**Source:** `landlock_nginx.rs:17-18`

```rust
let active_runtime_root = fs::canonicalize(&cfg.data.live_root)
    .with_context(|| format!("Failed to resolve live_root: {}", cfg.data.live_root))?;
```

Resolves the `live_root` symlink to the actual release directory.

**Example:**
- `live_root`: `/var/www/myapp` (symlink)
- `active_runtime_root`: `/srv/deployments/myapp/runtime/20260507_150000` (resolved)

---

### 4. Build Landlock Policy

**Source:** `landlock_nginx.rs:20-21`, `landlock_nginx.rs:24-41`

```rust
let socket_dir = PathBuf::from("/run").join(&cfg.data.project_name);
let policy = build_policy(&active_runtime_root, &socket_dir);
```

#### 4.1 Define Read-Only Paths

```rust
let mut read_only_paths = BTreeSet::new();
read_only_paths.insert(runtime_root.to_path_buf());

for system_path in landlock::default_system_read_paths() {
    read_only_paths.insert(system_path);
}
```

**Read-only access granted to:**
1. **Runtime root** - Application code and assets (e.g., `/srv/deployments/myapp/runtime/20260507_150000`)
2. **System paths** - Essential system directories:
   - `/usr` - User programs
   - `/lib`, `/lib64` - Shared libraries
   - `/bin`, `/sbin` - System binaries
   - `/etc` - Configuration files
   - `/dev` - Device files
   - `/proc` - Process information

#### 4.2 Define Writable Paths

```rust
let mut writable_paths = BTreeSet::new();
writable_paths.insert(socket_dir.to_path_buf());
```

**Writable access granted to:**
1. **Socket directory** - `/run/{project_name}/` for:
   - Unix socket: `nginx.sock`
   - PID file: `nginx.pid`
   - Temp paths: `client_body/`, `proxy/`, `fastcgi/`, etc.

**Why read-only for code?**
- Nginx serves static files - no need to modify them
- Prevents accidental or malicious modification of application code
- Security best practice

**Why writable for socket directory?**
- Nginx needs to create/listen on unix socket
- Needs to write PID file
- May need temp directories for request bodies

#### 4.3 Construct Policy Object

```rust
landlock::Policy {
    read_only_paths: read_only_paths.into_iter().collect(),
    writable_paths: writable_paths.into_iter().collect(),
}
```

---

### 5. Apply Landlock Restrictions

**Source:** `landlock_nginx.rs:23`

```rust
landlock::restrict_self(&policy)?;
```

Applies the Landlock policy to the current process.

**What this does:**
1. Creates Landlock ruleset with filesystem access rules
2. Enables the ruleset for the current thread
3. All future filesystem operations are restricted

**After this point, nginx can only:**
- Read from: site's release directory, system paths
- Write to: `/run/{project_name}/`
- All other filesystem access is denied

---

### 6. Execute Nginx

**Source:** `landlock_nginx.rs:25-29`

```rust
let nginx_conf = format!("{}/bones/nginx.conf", cfg.data.git_dir);
let mut command = Command::new("nginx");
command.args(["-c", &nginx_conf, "-g", "daemon off;"]);

let exec_error = command.exec();
bail!("Failed to exec nginx: {exec_error}")
```

#### 6.1 Build Command

Creates a `Command` to run nginx with:
- `-c {path}`: Path to per-site nginx config
- `-g "daemon off;"`: Run in foreground (required for systemd)

**Config location:** `{git_dir}/bones/nginx.conf` (e.g., `/home/git/myapp.git/bones/nginx.conf`)

#### 6.2 Execute (exec)

Uses `.exec()` instead of `.status()`:
- Replaces current process with nginx
- No fork, just exec
- Landlock restrictions persist
- Process becomes nginx (proper PID for systemd)

**Why exec?**
- More efficient (no fork overhead)
- Process becomes nginx (proper PID for systemd)
- Simpler process tree

---

## Security Model

### What Landlock Prevents

1. **Unauthorized file access**
   - Cannot read `/etc/shadow`
   - Cannot read other sites' directories
   - Cannot read other users' home directories

2. **Unauthorized file modification**
   - Cannot modify system configuration
   - Cannot modify application code
   - Cannot modify other sites' files

3. **Cross-site contamination**
   - Site A's nginx cannot access Site B's files
   - Isolated socket directories per site
   - No shared temp directories

### What Landlock Allows

1. **Reading site files** - Site's release directory (read-only)
2. **Reading system files** - `/lib`, `/usr`, `/etc`, etc. (read-only)
3. **Writing socket/pid** - `/run/{project}/` (read-write)

### Defense in Depth

Landlock provides an additional security layer:

1. **User isolation** - Service user has limited system permissions
2. **Landlock** - Kernel-enforced filesystem restrictions
3. **Nginx security** - Standard nginx security practices

Even if nginx is compromised:
- Cannot read sensitive system files
- Cannot modify system configuration
- Cannot access other sites' files

---

## Process Isolation Example

**Before Landlock:**
```
nginx can access:
  ✅ /srv/deployments/site-a/runtime/20260507_150000
  ✅ /srv/deployments/site-b/runtime/20260507_140000 ❌ (should not be accessible)
  ✅ /etc/shadow ❌ (should not be accessible)
  ✅ /home/user ❌ (should not be accessible)
```

**After Landlock:**
```
nginx can access:
  ✅ /srv/deployments/site-a/runtime/20260507_150000 (read-only)
  ✅ /run/site-a/ (read-write)
  ✅ /usr, /lib, /etc (read-only)
  ❌ /srv/deployments/site-b (denied)
  ❌ /etc/shadow (denied)
  ❌ /home/user (denied)
```

---

## Systemd Integration

Invoked via systemd service:

**Service Unit:** `/etc/systemd/system/{project}-nginx.service`
```ini
[Unit]
Description=Per-site nginx for myapp
After=network.target

[Service]
Type=simple
User=myapp
WorkingDirectory=/var/www/myapp
ExecStart=/usr/local/bin/bonesremote landlock nginx --config /home/git/myapp.git/bones/bones.yaml
Restart=always
RestartSec=2

[Install]
WantedBy=multi-user.target
```

**Execution Flow:**
1. systemd starts service as `myapp` user
2. Runs `bonesremote landlock nginx`
3. Landlock restrictions applied
4. Nginx executed in sandbox
5. Service is now isolated

---

## Nginx Configuration

Per-site nginx config is stored at `{git_dir}/bones/nginx.conf`:

```nginx
daemon off;
worker_processes 1;
pid /run/myapp/nginx.pid;
error_log stderr notice;

events {
    worker_connections 1024;
}

http {
    access_log stderr;
    client_body_temp_path /run/myapp/client_body;
    proxy_temp_path /run/myapp/proxy;
    fastcgi_temp_path /run/myapp/fastcgi;
    uwsgi_temp_path /run/myapp/uwsgi;
    scgi_temp_path /run/myapp/scgi;

    server {
        listen unix:/run/myapp/nginx.sock;
        root /var/www/myapp;
        index index.html;

        location / {
            try_files $uri $uri/ /index.html;
        }
    }
}
```

**Key configuration:**
- Listens on unix socket (not port)
- All temp paths under `/run/{project}/`
- Logs to stderr (for systemd/journald)
- Serves from `live_root`

---

## Architecture

```
                    ┌─────────────────────────┐
                    │   Main nginx (router)   │
                    │   :80 / :443            │
                    └───────────┬─────────────┘
                                │ proxy_pass
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
        ▼                       ▼                       ▼
┌───────────────┐       ┌───────────────┐       ┌───────────────┐
│ Site A nginx  │       │ Site B nginx  │       │ Site C nginx  │
│ (landlocked)  │       │ (landlocked)  │       │ (landlocked)  │
│ unix socket   │       │ unix socket   │       │ unix socket   │
└───────────────┘       └───────────────┘       └───────────────┘
        │                       │                       │
        ▼                       ▼                       ▼
   /var/www/a/            /var/www/b/            /var/www/c/
```

Each per-site nginx is isolated from others via Landlock.

---

## Troubleshooting

### Landlock Not Supported

```
Landlock support check failed: Kernel does not support Landlock
```

**Solution:** Upgrade to Linux kernel 5.13+ or enable Landlock in kernel config.

### Permission Denied

```
Failed to exec nginx: Permission denied
```

**Possible causes:**
- Nginx binary not executable
- Wrong user permissions
- Landlock policy too restrictive

### Socket Bind Failed

```
nginx: [emerg] bind() to unix:/run/myapp/nginx.sock failed
```

**Possible causes:**
- Socket directory doesn't exist
- Wrong permissions on `/run/myapp/`
- Socket already exists

---

## Related Commands

- `bonesremote doctor` - Check Landlock support
- `bonesremote hooks post-deploy` - Restarts per-site nginx
- `bonesdeploy remote setup` - Provisions server with per-site nginx
