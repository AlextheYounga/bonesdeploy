# bonesremote init

## Overview

Initializes the server-side deployment infrastructure by installing a sudoers drop-in file that allows the deploy user to run privileged `bonesremote` commands without a password. This command must be run as root and is typically executed once during initial server setup.

## Command Signature

```bash
sudo bonesremote init [--deploy-user <user>]
```

**Flags:**
- `--deploy-user <user>`: User allowed to run privileged commands (default: `git`)

---

## Detailed Execution Steps

### 1. Verify Root Privileges

**Source:** `init.rs:10`

```rust
privileges::ensure_root("bonesremote init")?;
```

Ensures the command is being run as root. This is required because:
- Writing to `/etc/sudoers.d/` requires root privileges
- Modifying sudoers configuration is a security-sensitive operation

**Implementation:**
```rust
// privileges.rs
pub fn ensure_root(context: &str) -> Result<()> {
    let uid = unsafe { libc::getuid() };
    if uid != 0 {
        bail!("{} must be run as root", context);
    }
    Ok(())
}
```

**Error if not root:**
```
bonesremote init must be run as root
```

---

### 2. Print Command Header

**Source:** `init.rs:12`

```rust
println!("{}", style(format!("{} init", config::Constants::BINARY_NAME)).bold());
```

Displays: `bonesremote init`

---

### 3. Locate bonesremote Binary

**Source:** `init.rs:15`, `init.rs:42-57`

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
        .context(format!("Failed to run 'which {}'", config::Constants::BINARY_NAME))?;

    if !output.status.success() {
        bail!(
            "{} is not in PATH. \
             Install it globally before running init.",
            config::Constants::BINARY_NAME
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
```

**Example Output:** `/usr/local/bin/bonesremote`

**Why needed?** The sudoers file must specify the absolute path to the binary for security reasons.

---

### 4. Generate Sudoers Content

**Source:** `init.rs:18-22`

```rust
let sudoers_content = format!(
    "# Installed by bonesremote init\n\
     {deploy_user} ALL=(root) NOPASSWD: {bonesdeploy_path} release stage --config *, {bonesdeploy_path} release wire --config *, {bonesdeploy_path} hooks post-deploy --config *\n"
);
```

Creates a sudoers drop-in that allows the deploy user to run specific `bonesremote` commands with root privileges without a password.

**Allowed Commands:**
1. `bonesremote release stage --config *`
   - Creates release directories
   - Requires root to create directories and set ownership

2. `bonesremote release wire --config *`
   - Wires shared paths
   - Requires root to manage symlinks and ownership

3. `bonesremote hooks post-deploy --config *`
   - Runs post-deployment tasks
   - Requires root to harden permissions and manage systemd services

**Example Generated Content:**
```
# Installed by bonesremote init
git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote release stage --config *, /usr/local/bin/bonesremote release wire --config *, /usr/local/bin/bonesremote hooks post-deploy --config *
```

**Security Notes:**
- Uses `NOPASSWD` for automation (no password prompts during deployment)
- Limited to specific commands (not blanket sudo access)
- Can only run as root (not arbitrary users)
- Wildcard `*` allows any config path (necessary for different projects)

---

### 5. Write Sudoers File

**Source:** `init.rs:24`

```rust
fs::write(sudoers_path, &sudoers_content).with_context(|| format!("Failed to write {sudoers_path}"))?;
```

Writes the sudoers drop-in to `/etc/sudoers.d/bonesdeploy`.

**Why `/etc/sudoers.d/`?**
- Drop-in directory for additional sudoers rules
- Safer than editing `/etc/sudoers` directly
- Easier to manage and remove
- Automatically included by modern sudo installations

---

### 6. Set Correct Permissions

**Source:** `init.rs:26-27`

```rust
Command::new("chmod").args(["0440", sudoers_path]).status().context("Failed to chmod sudoers drop-in")?;
```

Sets permissions to `0440` (read-only for owner and group).

**Why 0440?**
- Required by sudo for sudoers files
- Prevents write access (security)
- Only root (owner) and root group can read
- Sudo will reject sudoers files with incorrect permissions

---

### 7. Validate with visudo

**Source:** `init.rs:29-35`

```rust
let status = Command::new("visudo").args(["-c", "-f", sudoers_path]).status().context("Failed to run visudo")?;

if !status.success() {
    fs::remove_file(sudoers_path).ok();
    bail!("visudo validation failed — sudoers drop-in removed for safety");
}
```

Validates the sudoers file syntax using `visudo -c -f`.

**Why validate?**
- Syntax errors in sudoers can break sudo entirely
- `visudo -c` checks syntax without editing
- If validation fails, removes the file to prevent system issues

**Safety First:** If validation fails, the file is immediately removed to ensure sudo remains functional.

---

### 8. Print Success Message

**Source:** `init.rs:37`

```rust
println!("{} Installed sudoers drop-in at {}", style("Done!").green().bold(), sudoers_path);
```

**Example Output:**
```
Done! Installed sudoers drop-in at /etc/sudoers.d/bonesdeploy
```

---

## Result

After successful execution:

1. **File Created:** `/etc/sudoers.d/bonesdeploy`
   ```
   # Installed by bonesremote init
   git ALL=(root) NOPASSWD: /usr/local/bin/bonesremote release stage --config *, /usr/local/bin/bonesremote release wire --config *, /usr/local/bin/bonesremote hooks post-deploy --config *
   ```

2. **Permissions:** `0440` (read-only for root)

3. **Validated:** Syntax verified by `visudo`

4. **Deploy user can now run:**
   ```bash
   sudo bonesremote release stage --config /path/to/bones.yaml
   sudo bonesremote release wire --config /path/to/bones.yaml
   sudo bonesremote hooks post-deploy --config /path/to/bones.yaml
   ```
   ...without entering a password.

---

## Security Implications

### What This Enables

The deploy user (e.g., `git`) can run specific commands as root:
- Stage release: Create directories, set ownership
- Wire release: Manage symlinks, set ownership
- Post-deploy: Harden permissions, restart services

### What This Does NOT Allow

The deploy user cannot:
- Run arbitrary commands as root
- Edit arbitrary files
- Install packages
- Modify system configuration (except deployment-related)
- Run commands as other users

### Mitigations

1. **Limited command scope**: Only specific commands allowed
2. **No password prompt**: Reduces attack surface (no password to intercept)
3. **Audit trail**: All sudo commands are logged
4. **Config path restricted**: Commands require `--config` flag
5. **Easy to revoke**: Remove `/etc/sudoers.d/bonesdeploy` to disable

---

## When to Run

1. **Initial server setup**: After installing `bonesremote`
2. **After reinstalling bonesremote**: If binary path changes
3. **When changing deploy user**: Re-run with `--deploy-user`
4. **Troubleshooting permissions**: If deploy user can't run commands

---

## Typical Setup Workflow

```bash
# 1. Install bonesremote globally
sudo cp bonesremote /usr/local/bin/
sudo chmod +x /usr/local/bin/bonesremote

# 2. Initialize sudoers (run as root)
sudo bonesremote init --deploy-user git

# 3. Verify setup
sudo -u git sudo -n -l bonesremote release stage --config /nonexistent
# (Should succeed without password prompt)
```

---

## Troubleshooting

### Command Not Found

```
bonesremote is not in PATH. Install it globally before running init.
```

**Solution:** Install `bonesremote` in a system-wide location:
```bash
sudo cp bonesremote /usr/local/bin/
sudo chmod +x /usr/local/bin/bonesremote
```

### visudo Validation Failed

```
visudo validation failed — sudoers drop-in removed for safety
```

**Possible causes:**
- Syntax error in sudoers file
- Duplicate entry
- Invalid path to binary

**Solution:** Check for existing sudoers entries and ensure `bonesremote` path is correct:
```bash
which bonesremote
```

### Permissions Error

```
Failed to write /etc/sudoers.d/bonesdeploy
```

**Solution:** Run as root:
```bash
sudo bonesremote init
```

---

## Related Commands

- `bonesremote doctor` - Validate server setup
- `bonesdeploy remote setup` - Client-side server provisioning
- `bonesdeploy push` - Push configuration to server
