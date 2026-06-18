# bonesdeploy push

## Overview

Synchronizes the local `.bones/` directory to the remote bare Git repository using rsync, then configures the remote hooks by symlinking them into the bare repository's `hooks/` directory. This command ensures the remote server has the latest deployment configuration, hooks, and scripts.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `push.rs:11-12`

```rust
let bones_toml = Path::new(config::Constants::BONES_TOML);
let cfg = config::load(bones_toml)?;
```

Loads the deployment configuration from `.bones/bones.toml`. Required for:
- Remote server connection details (`host`, `port`)
- Git directory path (`repo_path`)
- Deploy user (`deploy_user`)

---

### 2. Calculate Remote Paths

**Source:** `push.rs:14-15`

```rust
let repo_path = &cfg.data.repo_path;
let remote_bones = format!("{repo_path}/{}/", config::Constants::REMOTE_BONES_DIR);
```

Constructs the destination path on the remote server:
- `repo_path`: Path to bare Git repository (e.g., `/home/git/myapp.git`)
- `remote_bones`: `<repo_path>/bones/` (e.g., `/home/git/myapp.git/bones/`)

---

### 3. Sync `.bones/` to Remote

**Source:** `push.rs:18-19`, `push.rs:53-77`

```rust
println!("Syncing .bones/ to {remote_bones}...");
rsync_bones(&cfg)?;
```

Uses rsync to transfer the local `.bones/` directory to the remote server.

#### 3.1 Construct Rsync Command

**Source:** `push.rs:60-70`

```rust
let status = Command::new("rsync")
    .args([
        "-av",         // archive mode, verbose
        "--delete",    // delete files on remote that don't exist locally
        "-e",
        &format!("ssh -p {port}"),
        &format!("{}/", config::Constants::BONES_DIR),  // source with trailing slash
        &dest,                                             // destination
    ])
    .status()
    .context("Failed to run rsync — is it installed?")?;
```

**Rsync Flags Explained:**
- `-a`: Archive mode (preserves permissions, times, symlinks, etc.)
- `-v`: Verbose output
- `--delete`: Remove files on destination that don't exist in source
- `-e "ssh -p {port}"`: Use SSH with custom port

**Source Path:** `.bones/` (with trailing slash means "copy contents of")
**Destination:** `{deploy_user}@{host}:{repo_path}/bones/`

**Example:**
```bash
rsync -av --delete -e "ssh -p 22" .bones/ git@deploy.example.com:/home/git/myapp.git/bones/
```

#### 3.2 Handle Rsync Failure

**Source:** `push.rs:72-74`

```rust
if !status.success() {
    bail!("rsync failed");
}
```

If rsync exits with non-zero status, the command fails immediately.

**Common failure reasons:**
- rsync not installed on local machine
- SSH connection failed
- Permission denied on remote
- Disk space issues on remote

---

### 4. Establish SSH Connection

**Source:** `push.rs:21-22`

```rust
let session = ssh::connect(&cfg).await?;
```

Opens an SSH session to the remote server for post-sync configuration.

---

### 5. Clean Sample Hooks

**Source:** `push.rs:24-30`

```rust
println!("Cleaning sample hooks from remote...");
let cmd = format!(
    "find {repo_path}/{}/ -maxdepth 1 -name '*.sample' -delete 2>/dev/null; true",
    config::Constants::REMOTE_HOOKS_DIR
);
ssh::run_cmd(&session, &cmd).await?;
```

Removes Git's default sample hook files from `<repo_path>/hooks/`:
- `pre-commit.sample`
- `post-update.sample`
- `pre-receive.sample`
- etc.

**Why?** Git creates these sample files when initializing a bare repository. They can interfere with hook symlinking and create confusion.

**Implementation:**
- Uses `find` to locate all `*.sample` files
- `-maxdepth 1` only searches the hooks directory (not subdirectories)
- `-delete` removes the files
- `2>/dev/null; true` suppresses errors and ensures success even if no files found

---

### 6. Symlink Hooks into Bare Repository

**Source:** `push.rs:32-44`

```rust
println!("Symlinking hooks...");
let cmd = format!(
    "for hook in {repo_path}/{}/{}/{}; do \
        name=$(basename \"$hook\"); \
        ln -sf \"$hook\" \"{repo_path}/{}/$name\"; \
      done",
    config::Constants::REMOTE_BONES_DIR,
    config::Constants::REMOTE_HOOKS_DIR,
    "*",
    config::Constants::REMOTE_HOOKS_DIR
);
ssh::run_cmd(&session, &cmd).await?;
```

Creates symlinks from the bare repository's `hooks/` directory to the versioned hooks in `bones/hooks/`.

#### 6.1 Symlink Logic

**Expanded Command:**
```bash
for hook in /home/git/myapp.git/bones/hooks/*; do
    name=$(basename "$hook")
    ln -sf "$hook" "/home/git/myapp.git/hooks/$name"
done
```

**For each hook in `bones/hooks/`:**
1. Extracts the hook name (e.g., `pre-receive`)
2. Creates symlink: `<repo_path>/hooks/<name>` -> `<repo_path>/bones/hooks/<name>`

**`ln -sf` Flags:**
- `-s`: Create symbolic link
- `-f`: Force (overwrite existing symlink or file)

**Example Result:**
```
/home/git/myapp.git/
├── hooks/
│   ├── pre-receive -> ../bones/hooks/pre-receive
│   └── post-receive -> ../bones/hooks/post-receive
└── bones/
    └── hooks/
        ├── pre-receive
        └── post-receive
```

**Why Symlinks?**
- Hooks are versioned in `bones/hooks/` (synced via rsync)
- Git expects hooks in `<repo_path>/hooks/`
- Symlinks allow hooks to be updated by running `push` without manual intervention
- Hook changes are tracked in the project repository

---

### 7. Close SSH Session

**Source:** `push.rs:46`

```rust
session.close().await?;
```

Cleanly closes the SSH connection.

---

### 8. Print Success Message

**Source:** `push.rs:48`

```rust
println!("\n{} .bones/ synced to remote.", style("Done!").green().bold());
```

---

## What Gets Synced

### Files Transferred

```
.bones/
├── bones.toml              # Configuration
├── hooks.sh                # Helper functions
├── hooks/                  # Server-side Git hooks
│   ├── pre-receive         # Validates push
│   ├── post-receive        # Triggers deployment
│   └── pre-push            # (Not used on remote, but synced anyway)
├── deployment/             # Deployment scripts
│   ├── 01_install.sh
│   ├── 02_build.sh
│   └── 03_migrate.sh
└── site/                   # DEPRECATED — no longer used. See infra/ instead.
```

### Files Modified on Remote

**Removed:**
- `<repo_path>/hooks/*.sample` - Sample hooks

**Created/Updated:**
- `<repo_path>/bones/**` - All files from local `.bones/`

**Symlinked:**
- `<repo_path>/hooks/pre-receive` -> `../bones/hooks/pre-receive`
- `<repo_path>/hooks/post-receive` -> `../bones/hooks/post-receive`
- (Any other hooks in `bones/hooks/`)

---

## When to Run

1. **After `bonesdeploy init`**: Initial sync to remote
2. **After modifying hooks**: When you've changed `bones/hooks/*`
3. **After modifying deployment scripts**: When you've changed `bones/deployment/*`
4. **After configuration changes**: When `bones.toml` has been updated
5. **Before first deployment**: Ensure remote has latest configuration
6. **When `doctor` reports sync issues**: If local/remote are out of sync

---

## Interaction with Git Push

The `bonesdeploy push` command is **different from** `git push`:

| Command | What it does |
|---------|--------------|
| `bonesdeploy push` | Syncs `.bones/` directory to remote |
| `git push` | Pushes commits to remote bare repository |

**Typical Workflow:**
1. Make changes to `.bones/hooks/post-receive`
2. Run `bonesdeploy push` to sync hooks
3. Run `git push` to trigger deployment (which uses updated hooks)

**Pre-Push Hook:** The `bones/hooks/pre-push` hook (symlinked to `.git/hooks/pre-push`) can automatically run `bonesdeploy push` before `git push` if configured.

---

## Error Scenarios

1. **rsync not found**: Install rsync on local machine
   ```
   Failed to run rsync — is it installed?
   ```

2. **SSH connection failed**: Check host, port, and SSH access
   ```
   Cannot connect to remote: ...
   ```

3. **Permission denied**: Ensure deploy_user has write access to `repo_path`
   ```
   rsync failed
   ```

4. **Disk space**: Remote server out of disk space
   ```
   rsync failed
   ```

---

## Related Commands

- `bonesdeploy init` - Creates `.bones/` scaffold
- `bonesdeploy doctor` - Checks sync status
- `bonesdeploy deploy` - Runs deployment hooks manually
- `git push` - Pushes commits and triggers deployment
