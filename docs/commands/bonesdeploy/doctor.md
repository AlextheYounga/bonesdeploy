# bonesdeploy doctor

## Overview

Validates the local and remote deployment environment to ensure everything is properly configured. This command performs a series of health checks on the local `.bones/` structure, Git configuration, and optionally the remote server setup. It helps identify configuration issues before deployment.

## Command Signature

```bash
bonesdeploy doctor [--local]
```

**Flags:**
- `--local`: Skip remote checks and only validate local configuration

## Detailed Execution Steps

### 1. Print Header

**Source:** `doctor.rs:12`

```rust
println!("{}", style("bonesdeploy doctor").bold());
```

Displays the command header.

---

### 2. Initialize Issues Collection

**Source:** `doctor.rs:14`

```rust
let mut issues: Vec<String> = Vec::new();
```

Creates a collection to store any problems discovered during checks. All checks append to this list rather than failing immediately, allowing the command to report all issues at once.

---

### 3. Local Checks

#### 3.1 Check `.bones/` Structure

**Source:** `doctor.rs:16`, `doctor.rs:40-60`

```rust
check_bones_structure(&mut issues);
```

**Validation Steps:**

1. **Verify `.bones/` directory exists**
   ```rust
   if !bones_dir.exists() {
       issues.push(format!("{}/ directory does not exist", config::Constants::BONES_DIR));
       return;
   }
   ```
   If missing, adds issue and skips remaining structure checks.

2. **Verify required files and directories exist**
   ```rust
   let expected = [
       config::Constants::BONES_TOML,           // .bones/bones.toml
       config::Constants::BONES_HOOKS_SCRIPT,   // .bones/hooks/hooks.sh
       config::Constants::BONES_HOOKS_DIR,      // .bones/hooks/
       config::Constants::BONES_DEPLOYMENT_DIR, // .bones/deployment/
   ];
   ```
   
   For each expected path:
   - Checks if the path exists
   - If missing, adds an issue to the list

**Expected Structure:**
```
.bones/
├── bones.toml
├── hooks.sh
├── hooks/
│   ├── pre-push
│   ├── pre-receive
│   └── post-receive
└── deployment/
    └── (deployment scripts)
```

---

#### 3.2 Check Deployment Script Naming

**Source:** `doctor.rs:17`, `doctor.rs:62-84`

```rust
check_deployment_naming(&mut issues);
```

Validates that all deployment scripts follow the numeric prefix convention.

**Implementation:**
1. Checks if `.bones/deployment/` exists
2. Iterates through all files in the directory
3. For each file, validates it has a numeric prefix (e.g., `01_`, `02_`)

**Why numeric prefix?** Deployment scripts are executed in sorted order. Numeric prefixes ensure deterministic execution order:
- `01_install_deps.sh`
- `02_build_assets.sh`
- `03_migrate_database.sh`

**Example Issue:**
```
Deployment script 'migrate.sh' does not start with a numeric prefix (e.g. 01_)
```

---

#### 3.3 Check Pre-Push Hook Symlink

**Source:** `doctor.rs:18`, `doctor.rs:86-108`

```rust
check_pre_push_symlink(&mut issues);
```

Verifies that the pre-push hook is properly symlinked from `.git/hooks/pre-push` to `../../.bones/hooks/pre-push`.

**Implementation:**
1. Check if `.git/hooks/pre-push` is a symlink
   ```rust
   if !link.symlink_metadata().is_ok_and(|m| m.is_symlink()) {
       issues.push(format!("{} is not symlinked", config::Constants::GIT_PRE_PUSH_HOOK_PATH));
       return;
   }
   ```

2. Read the symlink target
   ```rust
   let Ok(target) = fs::read_link(link) else {
       issues.push(format!("{}: cannot read symlink target", ...));
       return;
   };
   ```

3. Verify the target matches expected path
   ```rust
   let expected = Path::new(config::Constants::PRE_PUSH_HOOK_TARGET);
   if target != expected {
       issues.push(format!("{} points to '{}', expected '{}'", ...));
   }
   ```

**Valid State:**
- `.git/hooks/pre-push` is a symlink
- Target is `../../.bones/hooks/pre-push`

**Invalid States:**
- Not a symlink (e.g., regular file)
- Symlink points to wrong location
- Symlink doesn't exist

---

### 4. Remote Checks (Optional)

**Source:** `doctor.rs:20-26`

```rust
if !local_only {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    match config::load(bones_toml) {
        Ok(cfg) => check_remote(&cfg, &mut issues).await,
        Err(e) => issues.push(format!("Cannot load config: {e}")),
    }
}
```

If `--local` flag is not set, performs remote validation. Requires `bones.toml` to exist and be loadable.

#### 4.1 Establish SSH Connection

**Source:** `doctor.rs:110-117`

```rust
let session = match ssh::connect(cfg).await {
    Ok(s) => s,
    Err(e) => {
        issues.push(format!("Cannot connect to remote: {e}"));
        return;
    }
};
```

Connects to the remote server using configuration from `bones.toml`:
- Host: `cfg.data.host`
- Port: `cfg.data.port`
- User: `cfg.permissions.defaults.deploy_user`

If connection fails, adds issue and skips remaining remote checks.

---

#### 4.2 Check `bonesremote` Availability

**Source:** `doctor.rs:121-124`

```rust
if ssh::run_cmd(&session, "command -v bonesremote").await.is_err() {
    issues.push("bonesremote is not available on the remote".into());
}
```

Verifies that `bonesremote` binary is installed globally on the remote server.

**Implementation:** Runs `command -v bonesremote` via SSH, which returns the path to the binary if found, or fails if not in PATH.

---

#### 4.3 Check Remote `.bones/` Directory

**Source:** `doctor.rs:126-133`

```rust
let check_bones = format!("test -d {repo_path}/{}", config::Constants::REMOTE_BONES_DIR);
if ssh::run_cmd(&session, &check_bones).await.is_err() {
    issues.push(format!(
        "{repo_path}/{}/ does not exist on remote (run 'bonesdeploy push')",
        config::Constants::REMOTE_BONES_DIR
    ));
}
```

Verifies that the `.bones/` directory exists in the bare Git repository on the remote.

**Location:** `<repo_path>/bones/` (e.g., `/home/git/myapp.git/bones/`)

**Why this matters:** The remote `bones/` directory contains:
- Server-side hooks (`hooks/pre-receive`, `hooks/post-receive`)
- Deployment scripts
- `bones.toml` configuration

If missing, `bonesdeploy push` needs to be run to sync local `.bones/` to remote.

---

#### 4.4 Check Local/Remote Sync Status

**Source:** `doctor.rs:135-136`, `doctor.rs:172-224`

```rust
check_rsync_sync(cfg, issues);
```

Compares local `.bones/` with remote `<repo_path>/bones/` to detect drift.

**Implementation:**
1. Runs `rsync` in dry-run mode with delete flag:
   ```rust
   let output = Command::new("rsync")
       .args([
           "-avnc",           // archive, verbose, dry-run, checksum
           "--delete",        // would delete files on remote not in local
           "-e",
           &format!("ssh -p {port}"),
           &format!("{}/", config::Constants::BONES_DIR),
           &dest,
       ])
       .output();
   ```

2. Parses rsync output to find files that would be changed:
   ```rust
   let changed: Vec<&str> = stdout
       .lines()
       .filter(|line| {
           let line = line.trim();
           !line.is_empty()
               && !line.starts_with("sending ")
               && !line.starts_with("sent ")
               && !line.starts_with("total ")
               && !line.ends_with('/')
       })
       .collect();
   ```

3. If changes detected, adds issue with file list:
   ```
   Local .bones/ is out of sync with remote (run 'bonesdeploy push'). Changed files:
      hooks/pre-receive
      deployment/02_build.sh
   ```

**What this catches:**
- New files not yet pushed
- Modified files not synced
- Deleted files not removed from remote

---

#### 4.5 Check Remote Hook Symlinks

**Source:** `doctor.rs:138-168`

Verifies that Git hooks in the bare repository are properly symlinked to the `bones/hooks/` directory.

**Implementation:**
1. Runs shell command to check all hooks in `bones/hooks/`:
   ```bash
   for hook in <repo_path>/bones/hooks/*; do
       name=$(basename "$hook")
       link="<repo_path>/hooks/$name"
       if [ ! -L "$link" ] || [ "$(readlink "$link")" != "$hook" ]; then
           echo "$name"
       fi
   done
   ```

2. For each hook that's not properly symlinked, adds an issue:
   ```
   <repo_path>/hooks/pre-receive is not properly symlinked to bones/hooks/pre-receive
   ```

**Expected State:**
```
<repo_path>/
├── hooks/
│   ├── pre-receive -> ../bones/hooks/pre-receive
│   └── post-receive -> ../bones/hooks/post-receive
└── bones/
    └── hooks/
        ├── pre-receive
        └── post-receive
```

---

#### 4.6 Close SSH Session

**Source:** `doctor.rs:169`

```rust
let _ = session.close().await;
```

Closes the SSH connection cleanly.

---

### 5. Report Results

**Source:** `doctor.rs:28-37`

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

**Success Case:**
```
bonesdeploy doctor

OK All checks passed.
```

**Failure Case:**
```
bonesdeploy doctor

  ! .bones/hooks/hooks.sh is missing
  ! .git/hooks/pre-push is not symlinked
  ! bonesremote is not available on the remote
Doctor found 3 issues
```

## Exit Codes

- **0**: All checks passed
- **1**: One or more issues found

## When to Run

1. **After `bonesdeploy init`**: Verify local setup is correct
2. **Before first deploy**: Ensure remote is properly configured
3. **After configuration changes**: Validate new settings
4. **Troubleshooting deployment failures**: Identify configuration issues
5. **CI/CD pipelines**: Validate environment before deployment

## Common Issues Detected

1. **Missing `.bones/` structure**: Run `bonesdeploy init`
2. **Deployment scripts without numeric prefix**: Rename scripts (e.g., `01_script.sh`)
3. **Pre-push hook not symlinked**: Run `bonesdeploy init` again
4. **Remote not accessible**: Check SSH configuration and server status
5. **`bonesremote` not installed**: Install `bonesremote` on the server
6. **Remote `.bones/` missing**: Run `bonesdeploy push`
7. **Local/remote out of sync**: Run `bonesdeploy push`
8. **Remote hooks not symlinked**: Run `bonesdeploy push`

## Related Commands

- `bonesdeploy init` - Initializes local setup
- `bonesdeploy push` - Syncs local `.bones/` to remote
- `bonesremote doctor` - Server-side environment validation
