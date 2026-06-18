# bonesremote release stage

## Overview

Creates the directory structure and staging state for a new deployment release. Prepares the build workspace, release directory tree, and writes staged release state. Called internally by `bonesremote deploy --config <path>` (the recommended unified command) as part of the full pipeline.

Use this subcommand directly when composing a custom pipeline from individual building blocks.

## Command Signature

```bash
bonesremote release stage --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.toml` configuration file (required)

---

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `stage_release.rs:17`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration from `bones.toml`.

---

### 2. Define Directory Paths

**Source:** `stage_release.rs:19-22`

```rust
let project_root = Path::new(&cfg.data.project_root);
let build_root = release_state::build_root(&cfg);
let releases_dir = release_state::releases_dir(&cfg);
let shared_dir = release_state::shared_dir(&cfg);
```

**Paths:**

| Variable | Path | Purpose |
|----------|------|---------|
| `project_root` | `/srv/sites/myapp` | Root for all site data |
| `build_root` | `/srv/sites/myapp/build/workspace` | Where code is checked out |
| `releases_dir` | `/srv/sites/myapp/releases` | Contains all releases |
| `shared_dir` | `/srv/sites/myapp/shared` | Shared files across releases |

---

### 4. Create Base Directories

**Source:** `stage_release.rs:24-31`

```rust
fs::create_dir_all(project_root)
    .with_context(|| format!("Failed to create project_root: {}", project_root.display()))?;
fs::create_dir_all(&releases_dir)
    .with_context(|| format!("Failed to create release dir: {}", releases_dir.display()))?;
fs::create_dir_all(&build_root)
    .with_context(|| format!("Failed to create build workspace: {}", build_root.display()))?;
fs::create_dir_all(&shared_dir)
    .with_context(|| format!("Failed to create shared dir: {}", shared_dir.display()))?;
```

**Creates:**
```
/srv/sites/myapp/
├── build/
│   └── workspace/
├── releases/
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

**Creates:** `/srv/sites/myapp/releases/20260507_150432/`

This directory will hold the final runtime after the build is complete.

Ownership is inherited from `releases/` (`git:foo-release`) via the setgid bit — no chown needed.

---

### 8. Write Staged Release State

**Source:** `stage_release.rs:41`

```rust
release_state::write_staged_release(&cfg, &release_name)?;
```

**Implementation:** `release_state.rs:28-37`
```rust
pub fn write_staged_release(cfg: &Bones, release: &str) -> Result<()> {
    let path = staged_release_path(cfg);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create staged release state dir: {}", parent.display()))?;
    }

    fs::write(&path, format!("{release}\n"))
        .with_context(|| format!("Failed to write staged release state: {}", path.display()))
}
```

**Creates:** `<repo_path>/bones/.staged_release`

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
/srv/sites/myapp/
├── build/
│   └── workspace/          # (empty, ready for checkout)
├── releases/
│   ├── 20260507_130000/    # (existing release)
│   ├── 20260507_140000/    # (existing release)
│   └── 20260507_150432/    # (newly staged, empty)
├── shared/
│   ├── .env
│   └── storage/
└── current -> releases/20260507_140000/

/home/git/myapp.git/bones/
├── bones.toml
└── .staged_release         # Contains: 20260507_150432
```

---

## State File

**Location:** `<repo_path>/bones/.staged_release`

**Contents:** Release name (timestamp)

**Lifecycle:**
1. Created by `release stage`
2. Read by `release wire`, `hooks deploy`, `release activate`
3. Deleted by `release activate` (on success)
4. Deleted by `release drop-failed` (on failure)

---

## Why No Root?

The deploy user (`git`) owns `releases/`, `build/`, and `shared/` (or the runtime user owns shared). The setgid bit on `releases/` means new release dirs inherit the `foo-release` group automatically — no root ownership change needed.

The only command that requires root is `bonesremote service restart`, which is restricted via a narrow sudoers drop-in.

---

## Typical Workflow

```bash
# 1. Stage release
bonesremote release stage --config /home/git/myapp.git/bones/bones.toml

# 2. Check out code (done by post-receive hook)
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.toml --revision <sha>

# 3. Wire shared paths
bonesremote release wire --config /home/git/myapp.git/bones/bones.toml

# 4. Run deployment scripts and activate (done by hooks deploy)
bonesremote hooks deploy --config /home/git/myapp.git/bones/bones.toml
```

---

## Error Scenarios

### Directory Creation Failed

```
Failed to create project_root: /srv/deployments/myapp
```

**Possible causes:**
- Permission denied (not running as root)
- Disk space issues
- Path component is a file, not directory

### Release Directory Creation Failed

```
Failed to create release dir: /srv/sites/myapp/releases/20260507_150432
```

**Possible causes:**
- Permission denied (deploy user doesn't own parent)
- Disk space issues
- Path component is a file, not directory

---

## Related Commands

- `bonesremote release wire` - Wire shared paths
- `bonesremote release activate` - Activate staged release
- `bonesremote release drop-failed` - Clean up failed release
- `bonesremote hooks post-receive` - Orchestrates staging and wiring
