# bonesremote init

## Overview

Initializes the server-side deployment infrastructure by installing a sudoers drop-in file that allows the deploy user to run the service restart command with root privileges without a password. This command must be run as root and is typically executed once during initial server setup.

## Detailed Execution Steps

### 1. Verify Root Privileges

**Source:** `init.rs:12`

```rust
privileges::ensure_root("bonesremote init")?;
```

Ensures the command is being run as root. This is required because:
- Writing to `/etc/sudoers.d/` requires root privileges
- Modifying sudoers configuration is a security-sensitive operation

---

### 2. Print Command Header

```rust
println!("{}", style(format!("{} init", config::Constants::BINARY_NAME)).bold());
```

---

### 3. Locate bonesremote Binary

**Source:** `init.rs:17`, `init.rs:44-59`

```rust
let bonesdeploy_path = which_bonesdeploy_remote()?;
```

Finds the absolute path to the `bonesremote` binary using `which`.

**Implementation:**
```rust
fn which_bonesdeploy_remote() -> Result<String> {
    let output = Command::new("which")
        .arg(config::Constants::BINARY_NAME)
        .output()
        .context(...)?;

    if !output.status.success() {
        bail!("{} is not in PATH. Install it globally before running init.",
            config::Constants::BINARY_NAME);
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

**Example Output:** `/usr/local/bin/bonesremote`

---

### 4. Generate Sudoers Content

**Source:** `init.rs:19-24`

```rust
let sudoers_content = format!(
    "# Installed by bonesremote init\n\
     {} ALL=(root) NOPASSWD: {bonesdeploy_path} service restart --config *\n",
    paths::DEPLOY_USER
);
```

Creates a sudoers drop-in that allows the deploy user to run only `bonesremote service restart --config *` with root privileges without a password. This is the only `bonesremote` command that needs elevated privileges — it restarts the per-site nginx service.

**Example Generated Content:**
```
# Installed by bonesremote init
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote service restart --config *
```

**Security Notes:**
- Uses `NOPASSWD` for automation (no password prompts during deployment)
- Limited to a single command with a specific subcommand pattern
- Wildcard `*` allows any config path (necessary for different projects)
- The deploy user cannot run arbitrary commands as root

---

### 5. Write Sudoers File

**Source:** `init.rs:26`

```rust
fs::write(sudoers_path, &sudoers_content)?;
```

Writes the sudoers drop-in to `/etc/sudoers.d/bonesdeploy`.

---

### 6. Set Correct Permissions

**Source:** `init.rs:29`

```rust
Command::new("chmod").args(["0440", sudoers_path])...;
```

Sets permissions to `0440` (read-only for owner and group).

**Why 0440?**
- Required by sudo for sudoers files
- Prevents write access (security)
- Sudo rejects sudoers files with incorrect permissions

---

### 7. Validate with visudo

**Source:** `init.rs:32-37`

```rust
let status = Command::new("visudo").args(["-c", "-f", sudoers_path])...;

if !status.success() {
    fs::remove_file(sudoers_path).ok();
    bail!("visudo validation failed — sudoers drop-in removed for safety");
}
```

Validates the sudoers file syntax. If validation fails, the file is immediately removed to ensure sudo remains functional.

---

### 8. Print Success Message

```
Done! Installed sudoers drop-in at /etc/sudoers.d/bonesdeploy
```

---

## Result

After successful execution:

1. **File Created:** `/etc/sudoers.d/bonesdeploy`
   ```
   # Installed by bonesremote init
   git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote service restart --config *
   ```

2. **Permissions:** `0440` (read-only for root)

3. **Validated:** Syntax verified by `visudo`

4. **Deploy user can now run** `sudo bonesremote service restart --config <path>` without entering a password.

---

## Security Implications

### What This Enables

The deploy user can restart services as root via `sudo bonesremote service restart --config <path>`.

### What This Does NOT Allow

The deploy user cannot:
- Run arbitrary commands as root
- Edit arbitrary files
- Install packages
- Modify system configuration (except the specified service restart)

### Why Only `service restart`?

The deployment pipeline (`bonesremote deploy`) runs all non-privileged operations (doctor, stage, checkout, wire, scripts, activate, prune) as the deploy user directly. Only restarting system services requires root — and that's scoped down to the specific `bonesremote service restart` command.

---

## When to Run

1. **Initial server setup**: After installing `bonesremote`
2. **After reinstalling bonesremote**: If binary path has changed
3. **Troubleshooting permissions**: If deploy user can't run `sudo bonesremote service restart`

---

## Typical Setup Workflow

```bash
# 1. Install bonesremote globally
sudo cp bonesremote /usr/local/bin/
sudo chmod +x /usr/local/bin/bonesremote

# 2. Initialize sudoers (run as root)
sudo bonesremote init

# 3. Verify setup
sudo -u git sudo -n bonesremote service restart --config /nonexistent
# (Should succeed without password prompt)
```

---

## Related Commands

- `bonesremote doctor` - Validate server setup (checks sudoers config)
- `bonesdeploy remote setup` - Client-side server provisioning
- `bonesdeploy push` - Push configuration to server
