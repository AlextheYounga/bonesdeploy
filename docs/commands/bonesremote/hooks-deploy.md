# bonesremote hooks deploy

## Overview

Orchestrates the deployment sequence by running all deployment scripts in the `bones/deployment/` directory, publishing the build workspace to the release directory, and activating the release. This command is the main deployment driver that transforms a staged release into a running application.

## Command Signature

```bash
bonesremote hooks deploy --config <path>
```

**Flags:**
- `--config <path>`: Path to `bones.yaml` configuration file (required)

**Note:** Must NOT be run as root (runs as deploy user).

---

## Detailed Execution Steps

### 1. Verify Not Running as Root

**Source:** `deploy.rs:15`

```rust
privileges::ensure_not_root("bonesremote hooks deploy")?;
```

Ensures deployment runs as the deploy user, not root.

---

### 2. Load Configuration

**Source:** `deploy.rs:17`

```rust
let cfg = config::load(Path::new(config_path))?;
```

Loads deployment configuration.

---

### 3. Read Staged Release Name

**Source:** `deploy.rs:18`

```rust
let release_name = release_state::read_staged_release(&cfg)?;
```

Reads the staged release from `<git_dir>/bones/.staged_release`.

**Example:** `20260507_150432`

---

### 4. Define Directory Paths

**Source:** `deploy.rs:19-21`

```rust
let runtime_path = release_state::release_dir(&cfg, &release_name);
let build_root = release_state::build_root(&cfg);
let deployment_dir = Path::new(&cfg.data.git_dir).join("bones").join("deployment");
```

**Paths:**
- `runtime_path`: `/srv/deployments/myapp/runtime/20260507_150432`
- `build_root`: `/srv/deployments/myapp/build/workspace`
- `deployment_dir`: `/home/git/myapp.git/bones/deployment`

---

### 5. Verify Directories Exist

**Source:** `deploy.rs:23-29`

```rust
if !runtime_path.exists() {
    bail!("Staged runtime directory does not exist: {}", runtime_path.display());
}

if !build_root.exists() {
    bail!("Build workspace does not exist: {}", build_root.display());
}
```

Validates that staging and checkout were successful.

---

### 6. List Deployment Scripts

**Source:** `deploy.rs:31`, `deploy.rs:98-115`

```rust
let scripts = list_deployment_scripts(&deployment_dir)?;
```

**Implementation:**
```rust
fn list_deployment_scripts(deployment_dir: &Path) -> Result<Vec<PathBuf>> {
    if !deployment_dir.is_dir() {
        return Ok(Vec::new());
    }

    let mut scripts = Vec::new();
    for entry in fs::read_dir(deployment_dir)
        .with_context(|| format!("Failed to read deployment directory {}", deployment_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_file() {
            scripts.push(entry.path());
        }
    }

    scripts.sort();
    Ok(scripts)
}
```

**Process:**
1. Check if deployment directory exists
2. List all files in the directory
3. Sort by filename (alphanumeric)

**Example Scripts:**
```
/home/git/myapp.git/bones/deployment/
├── 01_install_dependencies.sh
├── 02_build_assets.sh
├── 03_run_migrations.sh
└── 04_clear_cache.sh
```

**Why numeric prefixes?**
- Ensures deterministic execution order
- `01_` runs before `02_`, etc.
- `bonesdeploy doctor` validates this convention

---

### 7. Run Deployment Scripts

**Source:** `deploy.rs:32-54`

#### 7.1 Check for Scripts

```rust
if scripts.is_empty() {
    println!("No deployment scripts found. Skipping deploy scripts.");
}
```

If no scripts exist, deployment proceeds to publishing.

#### 7.2 Execute Each Script

```rust
for script in scripts {
    let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
    println!("Running {script_name}...");

    let status = Command::new("bash")
        .arg(&script)
        .current_dir(&build_root)
        .status()
        .with_context(|| format!("Failed to execute deployment script {}", script.display()))?;

    if !status.success() {
        println!("Deployment script {script_name} failed.");
        drop_failed_release::run(config_path)
            .with_context(|| "Failed to drop staged release after deployment script failure")?;
        bail!("Deployment script {script_name} failed with status {status}");
    }
}

println!("All deployment scripts completed.");
```

**For each script:**

1. **Extract script name**
   ```rust
   let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
   ```

2. **Print progress**
   ```rust
   println!("Running {script_name}...");
   ```

3. **Execute script**
   ```rust
   let status = Command::new("bash")
       .arg(&script)
       .current_dir(&build_root)  // Run in build workspace
       .status()
       .with_context(|| format!("Failed to execute deployment script {}", script.display()))?;
   ```

   **Execution context:**
   - Working directory: `build_root` (`/srv/deployments/myapp/build/workspace`)
   - Shell: `bash`
   - Environment: Inherited from parent process
   - Shared paths: Accessible via symlinks (e.g., `.env`, `storage/`)

4. **Handle failure**
   ```rust
   if !status.success() {
       println!("Deployment script {script_name} failed.");
       drop_failed_release::run(config_path)
           .with_context(|| "Failed to drop staged release after deployment script failure")?;
       bail!("Deployment script {script_name} failed with status {status}");
   }
   ```

   **On failure:**
   - Drop failed release (clean up)
   - Abort deployment
   - Current symlink remains on previous release

5. **Continue on success**
   ```rust
   // Implicit: loop continues to next script
   ```

**Example Output:**
```
Running 01_install_dependencies.sh...
Running 02_build_assets.sh...
Running 03_run_migrations.sh...
Running 04_clear_cache.sh...
All deployment scripts completed.
```

---

### 8. Publish Runtime Tree

**Source:** `deploy.rs:56`, `deploy.rs:61-79`

```rust
publish_runtime_tree(&build_root, &runtime_path)?;
```

#### 8.1 Clear Release Directory

**Source:** `deploy.rs:81-96`

```rust
fn clear_directory(path: &Path) -> Result<()> {
    for entry in fs::read_dir(path).with_context(|| format!("Failed to read directory {}", path.display()))? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_type = entry.file_type().with_context(|| format!("Failed to inspect {}", entry_path.display()))?;

        if file_type.is_dir() {
            fs::remove_dir_all(&entry_path)
                .with_context(|| format!("Failed to remove directory {}", entry_path.display()))?;
        } else {
            fs::remove_file(&entry_path)
                .with_context(|| format!("Failed to remove {}", entry_path.display()))?;
        }
    }

    Ok(())
}
```

**Clears the release directory** (keeps the directory itself, removes contents).

**Why clear?**
- Release directory may have leftover files
- Ensures clean slate
- Only actual build artifacts will be present

#### 8.2 Copy Build to Release

**Source:** `deploy.rs:61-79`

```rust
fn publish_runtime_tree(build_root: &Path, runtime_path: &Path) -> Result<()> {
    clear_directory(runtime_path)?;

    let copy_source = build_root.join(".");
    let status = Command::new("cp")
        .arg("-a")
        .arg(&copy_source)
        .arg(runtime_path)
        .status()
        .with_context(|| {
            format!("Failed to copy build workspace {} to runtime tree {}", build_root.display(), runtime_path.display())
        })?;

    if !status.success() {
        bail!(
            "Failed to publish runtime tree from {} to {}: status {status}",
            build_root.display(),
            runtime_path.display()
        );
    }

    println!("Published runtime tree: {}", runtime_path.display());
    Ok(())
}
```

**Command executed:**
```bash
cp -a /srv/deployments/myapp/build/workspace/. /srv/deployments/myapp/runtime/20260507_150432/
```

**`cp -a` flags:**
- `-a`: Archive mode (preserves permissions, ownership, symlinks, timestamps)
- `source/.`: Copy contents of directory (not the directory itself)

**Result:** All files from `build/workspace/` are copied to `runtime/20260507_150432/`.

**Preserved:**
- Symlinks to shared paths
- File permissions
- Directory structure
- Timestamps

---

### 9. Activate Release

**Source:** `deploy.rs:58`

```rust
activate_release::run(config_path)?;
```

Calls `bonesremote release activate` to:
1. Switch `current` symlink to new release
2. Clear staged release state

**After this point, the new release is live.**

---

## Deployment Script Context

When deployment scripts run, they have access to:

### Environment

- **Working directory**: `build_root` (`/srv/deployments/myapp/build/workspace`)
- **Shell**: `bash`
- **User**: Deploy user (e.g., `git`)
- **Environment variables**: Inherited from hook process

### Shared Files (via symlinks)

```
build/workspace/
├── .env -> ../../shared/.env           # Environment configuration
├── storage -> ../../shared/storage     # User uploads
└── logs -> ../../shared/logs           # Application logs
```

### Project Files (from git checkout)

```
build/workspace/
├── src/
├── public/
├── package.json
├── composer.json
└── ... (all files from repository)
```

---

## Common Deployment Script Patterns

### Install Dependencies

**`01_install_dependencies.sh`:**
```bash
#!/bin/bash
set -e

# Node.js project
npm ci --production

# Or PHP project
# composer install --no-dev --optimize-autoloader
```

### Build Assets

**`02_build_assets.sh`:**
```bash
#!/bin/bash
set -e

# Build frontend assets
npm run build

# Or for Laravel
# php artisan optimize
```

### Run Migrations

**`03_run_migrations.sh`:**
```bash
#!/bin/bash
set -e

# Laravel
php artisan migrate --force

# Or Node.js/Prisma
# npx prisma migrate deploy
```

### Clear Cache

**`04_clear_cache.sh`:**
```bash
#!/bin/bash
set -e

# Laravel
php artisan cache:clear
php artisan config:clear
php artisan view:clear

# Or custom cache
# rm -rf storage/cache/*
```

---

## Error Handling

### Script Fails

```
Running 03_run_migrations.sh...
Deployment script 03_run_migrations.sh failed.
Removed failed release: 20260507_150432
Cleared staged release state.
Deployment script 03_run_migrations.sh failed with status 1
```

**What happens:**
1. Script exits with non-zero status
2. `drop_failed_release` called automatically
3. Release directory removed
4. Staged release state cleared
5. Deployment aborts
6. Current symlink unchanged (previous release still active)

### Recovery

1. Fix the script issue
2. Push changes (or use `bonesdeploy push`)
3. Redeploy

---

## Typical Workflow

```bash
# 1. Post-receive hook runs
bonesremote hooks post-receive --config /home/git/myapp.git/bones/bones.yaml

# 2. Post-receive calls deploy
bonesremote hooks deploy --config /home/git/myapp.git/bones/bones.yaml

# 3. Deploy runs scripts
# - 01_install_dependencies.sh
# - 02_build_assets.sh
# - 03_run_migrations.sh
# - 04_clear_cache.sh

# 4. Deploy publishes to release directory
# cp -a build/workspace/. runtime/20260507_150432/

# 5. Deploy activates release
# current -> runtime/20260507_150432

# 6. Post-deploy runs (separate command)
sudo bonesremote hooks post-deploy --config /home/git/myapp.git/bones/bones.yaml
```

---

## Directory State After Deploy

```
/srv/deployments/myapp/
├── build/
│   └── workspace/           # Build artifacts (retained for debugging)
├── runtime/
│   ├── 20260507_140000/     # Previous release
│   └── 20260507_150432/     # New release (populated)
├── shared/
│   ├── .env
│   └── storage/
└── current -> runtime/20260507_150432/  # Activated

/var/www/myapp -> /srv/deployments/myapp/current
```

---

## Related Commands

- `bonesremote hooks post-receive` - Checkout and wire release
- `bonesremote release activate` - Activate release
- `bonesremote release drop-failed` - Cleanup failed release
- `bonesremote hooks post-deploy` - Post-deployment tasks
