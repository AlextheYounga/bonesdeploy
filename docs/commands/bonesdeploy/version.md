# bonesdeploy version

## Overview

Displays the version number of the `bonesdeploy` binary. This is a simple informational command used to verify which version of the tool is installed.

## Command Signature

```bash
bonesdeploy version
```

## Execution

**Source:** `version.rs:1-3`

```rust
pub fn run() {
    println!("bonesdeploy {}", env!("CARGO_PKG_VERSION"));
}
```

**Output:**
```
bonesdeploy 1.0.0
```

---

## How Version is Determined

The version is embedded at compile time using Rust's `env!` macro:

- `CARGO_PKG_VERSION`: Automatically set by Cargo from `Cargo.toml`
- Extracted from the `version` field in `Cargo.toml`

**Example `Cargo.toml`:**
```toml
[package]
name = "bonesdeploy"
version = "1.2.3"
```

**Output:** `bonesdeploy 1.2.3`

---

## Use Cases

1. **Verify installation**: Check which version is installed
   ```bash
   bonesdeploy version
   ```

2. **Debug issues**: Report version when filing bug reports
   ```bash
   bonesdeploy version
   # bonesdeploy 1.0.0
   ```

3. **Check for updates**: Compare installed version with latest release
   ```bash
   bonesdeploy version
   # Check GitHub releases or package manager
   ```

4. **Script compatibility**: Ensure required version is installed
   ```bash
   #!/bin/bash
   VERSION=$(bonesdeploy version | awk '{print $2}')
   if [ "$VERSION" \< "1.0.0" ]; then
       echo "bonesdeploy 1.0.0 or later required"
       exit 1
   fi
   ```

---

## Integration with Other Commands

The version command is also accessible via:
- `bonesdeploy --version` (via clap's automatic handling)
- `bonesdeploy -V` (short flag)

---

## Comparison with bonesremote version

| Command | Output |
|---------|--------|
| `bonesdeploy version` | `bonesdeploy 1.0.0` |
| `bonesremote version` | `bonesremote 1.0.0` |

Both tools use independent versioning and should typically be kept in sync.

---

## Exit Code

Always exits with code `0` (success).

---

## Related Commands

- `bonesremote version` - Server-side tool version
