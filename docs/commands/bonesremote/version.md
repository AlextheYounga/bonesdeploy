# bonesremote version

## Overview

Displays the version number of the `bonesremote` binary. This is a simple informational command used to verify which version of the server-side tool is installed.

## Command Signature

```bash
bonesremote version
```

---

## Execution

**Source:** `version.rs:1-5`

```rust
use crate::config;

pub fn run() {
    println!("{} {}", config::Constants::BINARY_NAME, env!("CARGO_PKG_VERSION"));
}
```

**Output:**
```
bonesremote 1.0.0
```

---

## How Version is Determined

The version is embedded at compile time using Rust's `env!` macro:

- `CARGO_PKG_VERSION`: Automatically set by Cargo from `Cargo.toml`
- Extracted from the `version` field in `Cargo.toml`

**Example `Cargo.toml`:**
```toml
[package]
name = "bonesremote"
version = "1.2.3"
```

**Output:** `bonesremote 1.2.3`

---

## Use Cases

1. **Verify installation**: Check which version is installed
   ```bash
   bonesremote version
   ```

2. **Debug issues**: Report version when filing bug reports
   ```bash
   bonesremote version
   # bonesremote 1.0.0
   ```

3. **Check for updates**: Compare installed version with latest release
   ```bash
   bonesremote version
   # Check GitHub releases or package manager
   ```

4. **Script compatibility**: Ensure required version is installed
   ```bash
   #!/bin/bash
   VERSION=$(bonesremote version | awk '{print $2}')
   if [ "$VERSION" \< "1.0.0" ]; then
       echo "bonesremote 1.0.0 or later required"
       exit 1
   fi
   ```

5. **Server setup validation**: Confirm bonesremote is accessible
   ```bash
   # During site setup
   ssh git@server "bonesremote version"
   # bonesremote 1.0.0
   ```

---

## Integration with Other Commands

The version command is also accessible via:
- `bonesremote --version` (via clap's automatic handling)
- `bonesremote -V` (short flag)

---

## Comparison with bonesdeploy version

| Command | Output | Location |
|---------|--------|----------|
| `bonesdeploy version` | `bonesdeploy 1.0.0` | Client machine |
| `bonesremote version` | `bonesremote 1.0.0` | Server |

**Both tools use independent versioning** and should typically be kept in sync for compatibility.

---

## Exit Code

Always exits with code `0` (success).

---

## When to Run

1. **After installation**: Verify bonesremote is properly installed
   ```bash
   which bonesremote
   bonesremote version
   ```

2. **Before initialization**: Confirm binary is in PATH
   ```bash
   bonesremote version
   sudo bonesremote init
   ```

3. **Troubleshooting**: Check version when diagnosing issues
   ```bash
   bonesremote version
   # Check if version matches expected
   ```

4. **Upgrade validation**: Confirm upgrade succeeded
   ```bash
   # After upgrade
   bonesremote version
   # Should show new version
   ```

---

## Example Output

```
$ bonesremote version
bonesremote 1.0.0
```

Or with full path:

```
$ /usr/local/bin/bonesremote version
bonesremote 1.0.0
```

---

## Related Commands

- `bonesdeploy version` - Client-side tool version
- `bonesremote init` - Initialize server setup
- `bonesremote doctor` - Validate server environment
