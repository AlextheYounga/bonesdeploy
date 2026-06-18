# bonesdeploy manage

## Overview

Opens an interactive remote management terminal (TUI) on the server by establishing an SSH connection and launching `bonesremote manage`. This provides a user-friendly interface for managing releases, viewing logs, and performing server-side operations without manually SSH-ing into the server.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `manage.rs:9-10`

```rust
let bones_toml = Path::new(config::Constants::BONES_TOML);
let cfg = config::load(bones_toml)?;
```

Loads deployment configuration to determine:
- Remote server hostname (`host`)
- SSH port (`port`)
- Deploy user (`deploy_user`)
- Git directory path (`repo_path`)

---

### 2. Construct Remote Command

**Source:** `manage.rs:12-13`

```rust
let remote_bones_toml = format!("{}/{}/bones.toml", cfg.data.repo_path, config::Constants::REMOTE_BONES_DIR);
let remote_command = format!("bonesremote manage --config {}", shell_quote_single(&remote_bones_toml));
```

#### 2.1 Remote Configuration Path

The `bones.toml` on the server is located at:
```
<repo_path>/bones/bones.toml
```

**Example:** `/home/git/myapp.git/bones/bones.toml`

#### 2.2 Command Construction

**Example Remote Command:**
```bash
bonesremote manage --config '/home/git/myapp.git/bones/bones.toml'
```

**Why quote the path?** The `shell_quote_single` function ensures the path is properly escaped if it contains special characters or spaces.

**Source:** `manage.rs:33-35`
```rust
fn shell_quote_single(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}
```

This handles paths containing single quotes by escaping them properly:
- Input: `/path/with'quote/bones.toml`
- Output: `'/path/with'"'"'quote/bones.toml'`

---

### 3. Construct SSH Target

**Source:** `manage.rs:15`

```rust
let target = format!("{}@{}", cfg.permissions.defaults.deploy_user, cfg.data.host);
```

**Example:** `git@deploy.example.com`

---

### 4. Launch SSH Session

**Source:** `manage.rs:17-24`

```rust
let status = Command::new("ssh")
    .arg("-t")
    .arg("-p")
    .arg(&cfg.data.port)
    .arg(&target)
    .arg(&remote_command)
    .status()
    .context("Failed to launch ssh for remote manage session")?;
```

#### 4.1 SSH Flags

**`-t` flag:** Force pseudo-terminal allocation
- Required for interactive TUI applications
- Ensures the remote command can display colors, accept input, and handle screen rendering
- Without this, the TUI would not function properly

**`-p {port}` flag:** Specify SSH port
- Uses the port from `bones.toml` (default: 22)

#### 4.2 Full SSH Command

**Example:**
```bash
ssh -t -p 22 git@deploy.example.com "bonesremote manage --config '/home/git/myapp.git/bones/bones.toml'"
```

#### 4.3 Execution Mode

Uses `.status()` instead of `.output()`:
- Runs SSH interactively
- Passes local stdin/stdout/stderr directly to remote command
- Allows real-time interaction with the TUI

---

### 5. Handle Exit Status

**Source:** `manage.rs:26-28`

```rust
if !status.success() {
    bail!("Remote manage session failed with status {status}");
}
```

If the SSH session or remote command exits with non-zero status, the command fails.

---

## What `bonesremote manage` Does

While the implementation of `bonesremote manage` is not shown in the provided code, it typically provides:

### Common Features

1. **Release Management**
   - View all releases
   - Activate a specific release
   - Rollback to previous release
   - Delete old releases

2. **Log Viewing**
   - View application logs
   - View deployment logs
   - View service logs (systemd)

3. **Service Control**
   - Start/stop/restart service
   - View service status
   - Check service health

4. **Configuration**
   - View current configuration
   - Edit configuration
   - Validate configuration

5. **Environment Management**
   - View environment variables
   - Edit `.env` file
   - Manage shared paths

---

## Interactive TUI Example

```
┌─────────────────────────────────────────────────────┐
│  bonesremote manage - myapp                         │
├─────────────────────────────────────────────────────┤
│  Releases                                           │
│  ┌───────────────────────────────────────────────┐ │
│  │ ● 20260507_150000 (current)                   │ │
│  │   20260507_140000                             │ │
│  │   20260507_130000                             │ │
│  └───────────────────────────────────────────────┘ │
│                                                     │
│  [A]ctivate  [R]ollback  [D]elete  [L]ogs  [Q]uit │
└─────────────────────────────────────────────────────┘
```

---

## When to Use

1. **Release Management**: View and switch between releases
2. **Troubleshooting**: Check logs without SSH-ing manually
3. **Rollback**: Quickly revert to previous release
4. **Monitoring**: Check deployment and application status
5. **Maintenance**: Clean up old releases

---

## Advantages Over Manual SSH

| Manual SSH | `bonesdeploy manage` |
|-----------|---------------------|
| `ssh git@host` | `bonesdeploy manage` |
| Remember paths | Configuration loaded automatically |
| Run multiple commands | Single TUI interface |
| No visual feedback | Interactive visual interface |
| Error-prone | Guided operations |
| Requires server knowledge | User-friendly |

---

## Security Considerations

1. **SSH Access Required**: User must have SSH access to the server as the deploy user
2. **No Password Prompt**: Assumes SSH key authentication is configured
3. **Deploy User Permissions**: Commands run as deploy user, not root
4. **Configuration Access**: Can view and modify `bones.toml` on server

---

## Error Scenarios

1. **SSH Connection Failed**
   ```
   Failed to launch ssh for remote manage session
   ```
   - Check SSH key authentication
   - Verify host and port are correct
   - Ensure deploy user exists on server

2. **Remote Command Not Found**
   ```
   Remote manage session failed with status 127
   ```
   - `bonesremote` not installed on server
   - Run `bonesremote init` on server first

3. **Permission Denied**
   ```
   Remote manage session failed with status 1
   ```
   - Deploy user lacks permissions to read config
   - Configuration file doesn't exist

---

## Related Commands

- `bonesdeploy deploy` - Trigger deployment
- `bonesdeploy rollback` - Rollback to previous release
- `bonesdeploy doctor` - Validate environment
- `bonesremote manage` - Server-side management TUI
