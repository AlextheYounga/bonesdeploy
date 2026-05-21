# bonesremote release stage

## Overview

Creates the directory structure and staging state for a new deployment release. This command runs with root privileges to create directories and set ownership to the deploy user. It prepares the build workspace where code will be checked out and a release directory where the final runtime will be placed.

## Command Signature

```bash
sudo bonesremote release stage --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

**Note:** Must be run as root (via sudo).

---

## Detailed Execution Steps

### 1. Verify Root Privileges

**Source:** `stage_release.rs:15`

```rust
privileges::ensure_root("bonesremote release stage")?;
```

Ensures the command is running as root. Required for:
- Creating directories
- Changing ownership
- Setting permissions

---

### 2. Load Configuration

**Source:** `stage_release.rs:17`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration from `bones.yaml`.

---

### 3. Define Directory Paths

**Source:** `stage_release.rs:19-22`

```rust
let deploy_root = Path::new(&cfg.data.deploy_root);
let build_root = release_state::build_root(&cfg);
let releases_dir = release_state::releases_dir(&cfg);
let shared_dir = release_state::shared_dir(&cfg);
```

**Paths:**

| Variable | Path | Purpose |
|----------|------|---------|
| `deploy_root` | `/srv/deployments/myapp` | Root for all deployment data |
| `build_root` | `/srv/deployments/myapp/build/workspace` | Where code is checked out |
| `releases_dir` | `/srv/deployments/myapp/runtime` | Contains all releases |
| `shared_dir` | `/srv/deployments/myapp/shared` | Shared files across releases |

---

### 4. Create Base Directories

**Source:** `stage_release.rs:24-31`

```rust
fs::create_dir_all(deploy_root)
    .with_context(|| format!("Failed to create deploy_root: {}", deploy_root.display()))?;
fs::create_dir_all(&releases_dir)
    .with_context(|| format!("Failed to create runtime dir: {}", releases_dir.display()))?;
fs::create_dir_all(&build_root)
    .with_context(|| format!("Failed to create build workspace: {}", build_root.display()))?;
fs::create_dir_all(&shared_dir)
    .with_context(|| format!("Failed to create shared dir: {}", shared_dir.display()))?;
```

**Creates:**
```
/srv/deployments/myapp/
├── build/
│   └── workspace/
├── runtime/
└── shared/
```

**`create_dir_all`:** Creates parent directories if they don't exist.

---

### 5. Generate Release Name

**Source:** `stage_release.rs:33`, `stage_release.rs:47-51`

```rust
let release_name = create_release_name()?;
```

**Implementation:**
```rust
fn create_release_name() -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = 
        format_description!("[year][month][day]_[hour][minute][second]");
    let now = OffsetDateTime::now_utc();
    now.format(TIMESTAMP_FORMAT).context("Failed to format release timestamp")
}
```

**Format:** `YYYYMMDD_HHMMSS` (UTC timestamp)

**Example:** `20260507_150432`

**Why timestamp?**
- Chronological ordering
- Unique identifier
- Easy to sort and identify
- Indicates deployment time

---

### 6. Create Release Directory

**Source:** `stage_release.rs:34-36`

```rust
let staged_release_dir = release_state::release_dir(&cfg, &release_name);
fs::create_dir_all(&staged_release_dir)
    .with_context(|| format!("Failed to create release dir: {}", staged_release_dir.display()))?;
```

**Creates:** `/srv/deployments/myapp/runtime/20260507_150432/`

This directory will hold the final runtime after the build is complete.

---

### 7. Set Ownership to Deploy User

**Source:** `stage_release.rs:38-40`

```rust
permissions::chown_paths_to_deploy_user(&cfg, &[deploy_root, releases_dir.as_path()], false)?;
permissions::chown_paths_to_deploy_user(&cfg, &[build_root.as_path()], true)?;
permissions::chown_paths_to_deploy_user(&cfg, &[staged_release_dir.as_path()], true)?;
```

#### 7.1 Chown Logic

Changes ownership to `deploy_user:group` (e.g., `git:www-data`).

**First call:** `chown_paths_to_deploy_user(..., false)`
- Changes owner of directories themselves
- Does NOT change contents recursively
- For: `deploy_root`, `releases_dir`

**Second call:** `chown_paths_to_deploy_user(..., true)`
- Changes owner recursively
- For: `build_root`, `staged_release_dir`

**Why different?**
- `deploy_root` and `releases_dir` may contain existing releases owned by other users
- Don't want to change ownership of those
- `build_root` and `staged_release_dir` are new/empty, safe to chown recursively

#### 7.2 Why Deploy User?

The deploy user (`git`) will:
- Check out code into `build_root`
- Run deployment scripts
- Copy files to `staged_release_dir`

All these operations require write access.

---

### 8. Write Staged Release State

**Source:** `stage_release.rs:41`

```rust
release_state::write_staged_release(&cfg, &release_name)?;
```

**Implementation:** `release_state.rs:28-37`
```rust
pub fn write_staged_release(cfg: &BonesConfig, release: &str) -> Result<()> {
    let path = staged_release_path(cfg);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create staged release state dir: {}", parent.display()))?;
    }

    fs::write(&path, format!("{release}\n"))
        .with_context(|| format!("Failed to write staged release state: {}", path.display()))
}
```

**Creates:** `<git_dir>/bones/.staged_release`

**Example:** `/home/git/myapp.git/bones/.staged_release`

**Contents:**
```
20260507_150432
```

**Purpose:**
- Tracks which release is being built
- Allows other commands to find the staged release
- Cleared after activation

---

### 9. Print Success Message

**Source:** `stage_release.rs:43`

```rust
println!("Staged release: {release_name}");
```

**Example Output:**
```
Staged release: 20260507_150432
```

---

## Directory Structure After Staging

```
/srv/deployments/myapp/
├── build/
│   └── workspace/          # (empty, ready for checkout)
├── runtime/
│   ├── 20260507_130000/    # (existing release)
│   ├── 20260507_140000/    # (existing release)
│   └── 20260507_150432/    # (newly staged, empty)
├── shared/
│   ├── .env
│   └── storage/
└── current -> runtime/20260507_140000/

/home/git/myapp.git/bones/
├── bones.yaml
└── .staged_release         # Contains: 20260507_150432
```

---

## State File

**Location:** `<git_dir>/bones/.staged_release`

**Contents:** Release name (timestamp)

**Lifecycle:**
1. Created by `release stage`
2. Read by `release wire`, `hooks deploy`, `release activate`
3. Deleted by `release activate` (on success)
4. Deleted by `release drop-failed` (on failure)

---

## Why Root Privileges?

### What requires root?

1. **Create directories in `/srv/`** - Typically owned by root
2. **Change ownership** - Only root can change file owner
3. **Set permissions** - May need to set special permissions

### Why not run as deploy user?

- Deploy user may not have permission to create `/srv/deployments/`
- Deploy user cannot change ownership to itself
- Some operations require elevated privileges

### Security Model

1. **Root creates** directories and sets ownership
2. **Deploy user** performs all actual work (checkout, build, copy)
3. **Root** activates release and hardens permissions (post-deploy)

This minimizes the time running as root.

---

## Typical Workflow

```bash
# 1. Stage release (as root via sudo)
sudo bonesremote release stage --config /home/git/myapp.git/bones/bones.yaml

# 2. Check out code (as deploy user)
#    (done by post-receive hook)

# 3. Wire shared paths (as root via sudo)
sudo bonesremote release wire --config /home/git/myapp.git/bones/bones.yaml

# 4. Run deployment scripts (as deploy user)
#    (done by hooks deploy)

# 5. Activate release (as deploy user)
bonesremote release activate --config /home/git/myapp.git/bones/bones.yaml
```

---

## Error Scenarios

### Directory Creation Failed

```
Failed to create deploy_root: /srv/deployments/myapp
```

**Possible causes:**
- Permission denied (not running as root)
- Disk space issues
- Path component is a file, not directory

### Ownership Change Failed

```
Failed to chown path: /srv/deployments/myapp/build/workspace
```

**Possible causes:**
- User or group doesn't exist
- Not running as root

---

## Related Commands

- `bonesremote release wire` - Wire shared paths
- `bonesremote release activate` - Activate staged release
- `bonesremote release drop-failed` - Clean up failed release
- `bonesremote hooks post-receive` - Orchestrates staging and wiring
