# bonesremote doctor

## Overview

Validates the server environment to ensure all prerequisites and configurations are correct for `bonesremote` to function properly. Performs checks for OS compatibility, binary availability, sudoers configuration, AppArmor enforcement, and Landlock support.

## Command Signature

```bash
bonesremote doctor
```

---

## Detailed Execution Steps

### 1. Print Command Header

**Source:** `doctor.rs:12`

```rust
println!("{}", style(format!("{} doctor", config::Constants::BINARY_NAME)).bold());
```

Displays: `bonesremote doctor`

---

### 2. Initialize Issues Collection

**Source:** `doctor.rs:14`

```rust
let mut issues: Vec<String> = Vec::new();
```

Creates a collection to accumulate all discovered issues.

---

### 3. Check Supported Distribution

**Source:** `doctor.rs:16`, `doctor.rs:37-50`

```rust
check_supported_distribution(&mut issues);
```

Verifies the server is running a supported operating system.

**Implementation:**
```rust
fn check_supported_distribution(issues: &mut Vec<String>) {
    let os_release = fs::read_to_string("/etc/os-release");
    let Ok(os_release) = os_release else {
        issues.push("Failed to read /etc/os-release; expected Debian or Ubuntu host".to_string());
        return;
    };

    let normalized = os_release.to_lowercase();
    if normalized.contains("id=debian") || normalized.contains("id=ubuntu") {
        return;
    }

    issues.push("Unsupported host OS; bonesremote currently supports Debian/Ubuntu only".to_string());
}
```

**Checks:**
1. Reads `/etc/os-release`
2. Looks for `ID=debian` or `ID=ubuntu`
3. Reports issue if not Debian/Ubuntu

**Why Debian/Ubuntu only?**
- Tested and validated on these distributions
- Package names and paths may differ on other distros
- systemd service management assumes Debian-style paths

**Example Issue:**
```
Unsupported host OS; bonesremote currently supports Debian/Ubuntu only
```

---

### 4. Check Global Availability

**Source:** `doctor.rs:17`, `doctor.rs:52-59`

```rust
check_globally_available(&mut issues);
```

Verifies `bonesremote` binary is globally accessible in PATH.

**Implementation:**
```rust
fn check_globally_available(issues: &mut Vec<String>) {
    let result = Command::new(config::Constants::BINARY_NAME).arg("version").output();

    match result {
        Ok(output) if output.status.success() => {}
        _ => issues.push(format!("{} is not globally available (not in PATH)", config::Constants::BINARY_NAME)),
    }
}
```

**Process:**
- Runs `bonesremote version`
- If fails, binary is not properly installed

**Example Issue:**
```
bonesremote is not globally available (not in PATH)
```

**Solution:** Install globally:
```bash
sudo cp bonesremote /usr/local/bin/
sudo chmod +x /usr/local/bin/bonesremote
```

---

### 5. Check Passwordless Sudo

**Source:** `doctor.rs:18`, `doctor.rs:61-81`

```rust
check_passwordless_sudo(&mut issues);
```

Verifies the deploy user can run privileged commands without a password.

**Implementation:**
```rust
fn check_passwordless_sudo(issues: &mut Vec<String>) {
    let privileged_commands = [
        [config::Constants::BINARY_NAME, "release", "stage", "--config", "/nonexistent"],
        [config::Constants::BINARY_NAME, "release", "wire", "--config", "/nonexistent"],
        [config::Constants::BINARY_NAME, "hooks", "post-deploy", "--config", "/nonexistent"],
    ];

    for command in privileged_commands {
        let result = Command::new("sudo").arg("-n").arg("-l").args(command).output();

        match result {
            Ok(output) if output.status.success() => {}
            _ => issues.push(format!(
                "{} is not allowed via passwordless sudo: {} (run 'sudo {} init')",
                config::Constants::BINARY_NAME,
                command.join(" "),
                config::Constants::BINARY_NAME
            )),
        }
    }
}
```

**Checks:**
1. For each privileged command:
   - Runs `sudo -n -l <command>` (no password, list allowed commands)
   - If fails, user can't run that command passwordless

**sudo Flags:**
- `-n`: Non-interactive mode (fail if password needed)
- `-l`: List allowed commands

**Example Issue:**
```
bonesremote is not allowed via passwordless sudo: bonesremote release stage --config /nonexistent (run 'sudo bonesremote init')
```

**Solution:** Run initialization:
```bash
sudo bonesremote init --deploy-user git
```

---

### 6. Check AppArmor Support

`bonesremote doctor` validates AppArmor with read-only checks by checking:

1. `/sys/module/apparmor/parameters/enabled` reports enabled (`Y/yes/1`)
2. `systemctl is-active apparmor` is `active`
3. `/etc/apparmor.d/` contains one or more `bonesdeploy-*-nginx` profiles
4. For each installed `bonesdeploy-<project>-nginx` profile, `/etc/systemd/system/<project>-nginx.service` exists, includes `AppArmorProfile=bonesdeploy-<project>-nginx`, and declares `apparmor.service` in both `After=` and `Requires=`

Example failures:

```text
AppArmor check failed: kernel module is not enabled
AppArmor check failed: apparmor service is not active (status: inactive)
AppArmor check failed: no bonesdeploy AppArmor profile found under /etc/apparmor.d
AppArmor check failed: expected /etc/systemd/system/demo-nginx.service for installed bonesdeploy profile
AppArmor check failed: /etc/systemd/system/demo-nginx.service references unknown AppArmor profile bonesdeploy-demo-ngnix
```

---

### 7. Check Landlock Support

**Source:** `doctor.rs:19`, `doctor.rs:83-88`

```rust
check_landlock_support(&mut issues);
```

Verifies the kernel supports Landlock LSM for runtime isolation.

**Implementation:**
```rust
fn check_landlock_support(issues: &mut Vec<String>) {
    match landlock::verify_support() {
        Ok(()) => {}
        Err(error) => issues.push(format!("Landlock support check failed: {error}")),
    }
}
```

**Landlock Requirements:**
- Linux kernel 5.13+ (for full feature support)
- Landlock LSM enabled in kernel config
- May require kernel boot parameters

**Why Landlock?**
- Provides mandatory access control (MAC)
- Sandboxes runtime processes
- Limits filesystem access to specific directories
- Enhances security without requiring root

**Example Issue:**
```
Landlock support check failed: Kernel does not support Landlock
```

**Solution:** Upgrade kernel or enable Landlock in kernel configuration.

---

### 8. Report Results

**Source:** `doctor.rs:25-34`

```rust
if issues.is_empty() {
    println!("\n{} All checks passed.", style("OK").green().bold());
    Ok(())
} else {
    println!();
    for issue in &issues {
        println!("  {} {issue}", style("!").red().bold());
    }
    anyhow::bail!("Doctor found {} issue{}", issues.len(), if issues.len() == 1 { "" } else { "s" });
}
```

**Success Output:**
```
bonesremote doctor

OK All checks passed.
```

**Failure Output:**
```
bonesremote doctor

  ! bonesremote is not globally available (not in PATH)
  ! bonesremote is not allowed via passwordless sudo: bonesremote release stage --config /nonexistent (run 'sudo bonesremote init')
Doctor found 2 issues
```

---

## Checks Summary

| Check | What it validates | Fix |
|-------|-------------------|-----|
| **Supported distribution** | Debian/Ubuntu OS | Use supported OS |
| **Global availability** | `bonesremote` in PATH | Install globally |
| **Passwordless sudo** | Sudoers configured | Run `sudo bonesremote init` |
| **AppArmor support** | Kernel + service + enforcing profiles | Enable and enforce AppArmor |
| **Landlock support** | Kernel supports Landlock | Upgrade kernel |

---

## When to Run

1. **After installing bonesremote**: Verify setup
2. **Before first deployment**: Ensure environment is ready
3. **Troubleshooting**: Diagnose permission or configuration issues
4. **After server updates**: Verify nothing broke
5. **CI/CD validation**: Automated environment checks

---

## Typical Workflow

```bash
# 1. Install bonesremote
sudo cp bonesremote /usr/local/bin/
sudo chmod +x /usr/local/bin/bonesremote

# 2. Initialize sudoers
sudo bonesremote init

# 3. Verify setup
bonesremote doctor
```

---

## Related Commands

- `bonesremote init` - Initialize sudoers
- `bonesdeploy doctor` - Client-side validation
- `bonesdeploy remote setup` - Provision server
