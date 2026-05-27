# bonesremote release wire

## Overview

Creates symlinks from the build workspace to the shared directory for all configured shared paths. This ensures that files and directories that should persist across releases (like `.env`, `storage/`, logs) are available in the build workspace. The command runs with root privileges to manage symlinks and ownership.

## Command Signature

```bash
sudo bonesremote release wire --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

**Note:** Must be run as root (via sudo).

---

## Detailed Execution Steps

### 1. Verify Root Privileges

**Source:** `wire_release.rs:13`

```rust
privileges::ensure_root("bonesremote release wire")?;
```

Ensures the command is running as root. Required for:
- Managing symlinks
- Changing file ownership
- Moving files between directories

---

### 2. Load Configuration

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

### 4. Define Build and Shared Paths

**Source:** `wire_release.rs:17-18`

```rust
let build_root = release_state::build_root(&cfg);
let shared_dir = release_state::shared_dir(&cfg);
```

**Paths:**
- `build_root`: `/srv/deployments/myapp/build/workspace`
- `shared_dir`: `/srv/deployments/myapp/shared`

---

### 5. Wire Shared Files and Directories

**Source:** `wire_release.rs:20-23`

```rust
for shared_file in &cfg.releases.shared_files {
    wire_path(&cfg, &build_root, &shared_dir, shared_file)?;
}

for shared_dir_path in &cfg.releases.shared_dirs {
    wire_path(&cfg, &build_root, &shared_dir, shared_dir_path)?;
}
```

Iterates through `releases.shared_files` and `releases.shared_dirs` and wires each entry.

**Example shared files and directories:**
```yaml
releases:
  shared_files:
    - .env
  shared_dirs:
    - storage
    - logs
```

---

### 6. Wire Path Logic

**Source:** `wire_release.rs:28-61`

For each shared path, performs the following steps:

#### 6.1 Define Paths

```rust
let release_path = release_dir.join(relative_path);
let shared_path = shared_dir.join(relative_path);
```

**Example for `.env`:**
- `release_path`: `/srv/deployments/myapp/build/workspace/.env`
- `shared_path`: `/srv/deployments/myapp/shared/.env`

#### 6.2 Move Existing File to Shared (if needed)

**Source:** `wire_release.rs:32-41`

```rust
if path_exists(&release_path) && !path_exists(&shared_path) {
    ensure_parent_exists(&shared_path)?;
    fs::rename(&release_path, &shared_path).with_context(|| {
        format!(
            "Failed to move release path into shared path: {} -> {}",
            release_path.display(),
            shared_path.display()
        )
    })?;
}
```

**Scenario:** First deployment, file exists in release but not in shared.

**Action:**
- Move file from release to shared
- Prevents losing the file on subsequent deployments

**Example:**
- `.env` exists in `build/workspace/.env`
- Doesn't exist in `shared/.env`
- Move: `build/workspace/.env` → `shared/.env`

#### 6.3 Create Default Shared Target (if needed)

**Source:** `wire_release.rs:43-45`, `wire_release.rs:63-75`

```rust
if !path_exists(&shared_path) {
    create_default_shared_target(&shared_path, relative_path)?;
}
```

**Implementation:**
```rust
fn create_default_shared_target(shared_path: &Path, relative_path: &str) -> Result<()> {
    ensure_parent_exists(shared_path)?;

    if looks_like_file(relative_path) {
        fs::File::create(shared_path)
            .with_context(|| format!("Failed to create shared file: {}", shared_path.display()))?;
    } else {
        fs::create_dir_all(shared_path)
            .with_context(|| format!("Failed to create shared directory: {}", shared_path.display()))?;
    }

    Ok(())
}
```

**Creates the shared file/directory if it doesn't exist:**
- File: Creates empty file
- Directory: Creates directory (and parents)

**File vs Directory Heuristic:** `wire_release.rs:77-82`
```rust
fn looks_like_file(relative_path: &str) -> bool {
    PathBuf::from(relative_path).file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.starts_with('.') || name.contains('.')
    })
}
```

**Rules:**
- Starts with `.` → File (e.g., `.env`, `.htaccess`)
- Contains `.` → File (e.g., `config.json`, `app.db`)
- Otherwise → Directory (e.g., `storage`, `logs`)

**Examples:**
- `.env` → File (starts with `.`)
- `config.json` → File (contains `.`)
- `storage` → Directory (no `.`)

#### 6.4 Set Ownership

**Source:** `wire_release.rs:47`

```rust
permissions::chown_paths_to_deploy_user(cfg, &[shared_path.as_path()], true)?;
```

Changes ownership of shared path to `deploy_user:group`.

**Why?** Deploy user needs to read/write the shared files.

#### 6.5 Remove Existing Release Path

**Source:** `wire_release.rs:49-52`

```rust
ensure_parent_exists(&release_path)?;
if path_exists(&release_path) {
    remove_path(&release_path)?;
}
```

**Removes** the file/directory/symlink from the release path to make room for the symlink.

**Why remove?**
- Path might be a file from the repository
- Path might be an old symlink
- Need clean location for new symlink

#### 6.6 Create Symlink

**Source:** `wire_release.rs:54-58`

```rust
symlink(&shared_path, &release_path).with_context(|| {
    format!("Failed to create shared symlink {} -> {}", release_path.display(), shared_path.display())
})?;
```

**Creates symlink:** `release_path` → `shared_path`

**Example:**
```
build/workspace/.env -> ../../shared/.env
build/workspace/storage -> ../../shared/storage
build/workspace/logs -> ../../shared/logs
```

**Result:** The build workspace now has symlinks to the shared files/directories. When code is checked out or deployment scripts run, they'll interact with the actual shared files.

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
Linked shared path: /srv/deployments/myapp/build/workspace/.env -> /srv/deployments/myapp/shared/.env
Linked shared path: /srv/deployments/myapp/build/workspace/storage -> /srv/deployments/myapp/shared/storage
Wired build workspace for staged release: 20260507_150432
```

---

## Directory Structure After Wiring

```
/srv/deployments/myapp/
├── build/
│   └── workspace/
│       ├── .env -> ../../shared/.env           # Symlink
│       ├── storage -> ../../shared/storage     # Symlink
│       ├── logs -> ../../shared/logs           # Symlink
│       └── (other files from git checkout)
├── releases/
│   └── 20260507_150432/    # (empty, waiting for build)
├── shared/
│   ├── .env                # Actual file
│   ├── storage/            # Actual directory
│   └── logs/               # Actual directory
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
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.yaml

# 2. Check out code
git --work-tree=/srv/deployments/myapp/build/workspace \
    --git-dir=/home/git/myapp.git \
    checkout -f master

# 3. Wire shared paths
sudo bonesremote release wire --config /home/git/myapp.git/bones/bones.yaml

# 4. Build and deploy
# (deployment scripts run in build/workspace with symlinks active)

# 5. Activate release
bonesremote release activate --config /home/git/myapp.git/bones/bones.yaml
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
