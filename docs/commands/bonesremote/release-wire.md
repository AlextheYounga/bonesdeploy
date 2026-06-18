# bonesremote release wire

## Overview

Creates symlinks from the build workspace to the shared directory for all configured shared paths (defined in `runtime.toml`). Called internally by `bonesremote deploy --config <path>` (the recommended unified command) after git checkout.

Use this subcommand directly when composing a custom pipeline from individual building blocks.

## Command Signature

```bash
bonesremote release wire --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.toml` configuration file (required)

---

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `wire_release.rs:15`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration to get:
- Staged release name
- Shared paths configuration
- Directory locations

---

### 3. Read Staged Release Name

**Source:** `wire_release.rs:16`

```rust
let release_name = release_state::read_staged_release(&cfg)?;
```

Reads the release name from `<repo_path>/bones/.staged_release`.

**Example:** `20260507_150432`

**Fails if:**
- State file doesn't exist (no release staged)
- State file is empty
- File is not readable

---

### 2. Define Build and Shared Paths

**Source:** `wire_release.rs:17-18`

```rust
let build_root = release_state::build_root(&cfg);
let shared_dir = release_state::shared_dir(&cfg);
```

**Paths:**
- `build_root`: `/srv/sites/myapp/build/workspace`
- `shared_dir`: `/srv/sites/myapp/shared`

---

### 3. Load and Wire Shared Paths

**Source:** `wire_release.rs:19-23`

```rust
let shared_paths = load_runtime_shared_paths(config_path)?;
for shared_path in &shared_paths {
    validate_shared_path(&shared_path.path)?;
    wire_path(&build_root, &shared_dir, &shared_path.path)?;
}
```

Shared paths are loaded from `runtime.toml` (sibling of `bones.toml` in the bare repo's `bones/` dir):

```toml
[shared]
paths = [
    { path = ".env", type = "file" },
    { path = "storage", type = "dir" },
]
```

---

### 6. Wire Path Logic

**Source:** `wire_release.rs:28-61`

For each shared path, performs the following steps:

#### 3.1 Validate Path

```rust
fn validate_shared_path(relative_path: &str) -> Result<()> {
    if relative_path.is_empty() { bail!("shared path must not be empty"); }
    if relative_path.starts_with('/') { bail!("shared path must be relative, got: {relative_path}"); }
    if relative_path.contains("..") { bail!("shared path must not contain .., got: {relative_path}"); }
    Ok(())
}
```

Rejects absolute paths, empty paths, and parent-directory traversal — shared paths must live under the project tree.

#### 3.2 Wire Symlink

```rust
let release_path = build_root.join(relative_path);
let shared_path = shared_dir.join(relative_path);

if !shared_path_exists(&shared_path) {
    bail!("shared path does not exist: {}", shared_path.display());
}

ensure_parent_exists(&release_path)?;
if release_path_is_resolved(&release_path) {
    replace_workspace_path_with_shared_symlink(&release_path)?;
}

symlink(&shared_path, &release_path)?;
```

**Example for `.env`:**
- `release_path`: `/srv/sites/myapp/build/workspace/.env`
- `shared_path`: `/srv/sites/myapp/shared/.env`

**Flow:**
1. Verify the shared target already exists (provisioned by `setup.py` or previous deploy)
2. Ensure parent directories exist in the build workspace
3. If a file, dir, or old symlink exists at the target path in the workspace, replace it with `replace_workspace_path_with_shared_symlink` (safe because this is a disposable workspace — never `current`, `releases/`, or `shared/`)
4. Create a symlink from the workspace path to the shared path

**Result:**

```
build/workspace/.env -> ../../shared/.env
build/workspace/storage -> ../../shared/storage
```

**Why no chown?** The shared directory is already owned by the runtime user (provisioned during `bonesdeploy remote setup`). The workspace is owned by the deploy user. Symlinks carry their own permissions (always `0777`), and reads go through the target's ownership — no permission change needed.

#### 6.7 Print Progress

**Source:** `wire_release.rs:58`

```rust
println!("Linked shared path: {} -> {}", release_path.display(), shared_path.display());
```

---

### 7. Print Success Message

**Source:** `wire_release.rs:24`

```rust
println!("Wired build workspace for staged release: {release_name}");
```

**Example Output:**
```
Linked shared path: /srv/sites/myapp/build/workspace/.env -> /srv/sites/myapp/shared/.env
Linked shared path: /srv/sites/myapp/build/workspace/storage -> /srv/sites/myapp/shared/storage
Wired build workspace for staged release: 20260507_150432
```

---

## Directory Structure After Wiring

```
/srv/sites/myapp/
├── build/
│   └── workspace/
│       ├── .env -> ../../shared/.env           # Symlink
│       ├── storage -> ../../shared/storage     # Symlink
│       └── (other files from git checkout)
├── releases/
│   └── 20260507_150432/    # (empty, waiting for build)
├── shared/
│   ├── .env                # Actual file, runtime-user-owned
│   └── storage/            # Actual directory, runtime-user-owned
└── current -> releases/20260507_140000/
```

---

## Why Shared Paths?

### Persistence Across Releases

Shared paths solve the problem of data that should persist between deployments:

1. **Configuration files** (`.env`)
   - Contains secrets and environment-specific settings
   - Should not be in version control
   - Needs to persist across releases

2. **User uploads** (`storage/`, `uploads/`)
   - User-generated content
   - Cannot be lost during deployment
   - Grows over time

3. **Logs** (`logs/`, `var/log/`)
   - Application logs
   - Useful for debugging
   - Should persist for analysis

4. **Cache** (`cache/`, `tmp/`)
   - Application cache
   - Improves performance
   - Can be lost, but better to preserve

### Atomic Updates

When a new release is activated:
- Symlink is switched atomically
- Shared files remain accessible
- No downtime for file access
- Old release still has access to shared files (until cleaned up)

---

## Typical Workflow

```bash
# 1. Stage release
bonesremote release stage --config /home/git/myapp.git/bones/bones.toml

# 2. Check out code (done by post-receive hook)
git --work-tree=/srv/sites/myapp/build/workspace \
    --git-dir=/home/git/myapp.git \
    checkout -f master

# 3. Wire shared paths
bonesremote release wire --config /home/git/myapp.git/bones/bones.toml

# 4. Build and deploy
# (deployment scripts run in build/workspace with symlinks active)

# 5. Activate release
bonesremote release activate --config /home/git/myapp.git/bones/bones.toml
```

---

## Edge Cases

### First Deployment

**Scenario:** No shared files exist yet.

**Behavior:**
1. Files from repo (e.g., `.env.example`) moved to shared
2. Renamed to `.env` (if needed)
3. Symlink created

### File Exists in Both Locations

**Scenario:** `release_path` and `shared_path` both exist.

**Behavior:**
- `release_path` is removed (the version from repository)
- Symlink created to `shared_path` (preserves existing shared file)
- Repository version is lost (expected behavior)

### Shared Directory Already Exists

**Scenario:** `shared/storage` already exists with files.

**Behavior:**
- No files moved
- Symlink created
- All existing files remain accessible

---

## Related Commands

- `bonesremote release stage` - Stage a new release
- `bonesremote release activate` - Activate the release
- `bonesremote hooks post-receive` - Orchestrates staging and wiring
- `bonesremote hooks deploy` - Runs deployment scripts
