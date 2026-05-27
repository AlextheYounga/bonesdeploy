# bonesdeploy deploy

## Overview

Manually triggers the deployment sequence on the remote server by running the `pre-receive` and `post-receive` hooks directly, without requiring a `git push`. This is useful for testing deployment scripts, re-running failed deployments, or deploying when you don't have new commits to push.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `deploy.rs:9-10`

```rust
let bones_yaml = Path::new(config::Constants::BONES_YAML);
let cfg = config::load(bones_yaml)?;
```

Loads deployment configuration to determine:
- Remote server connection details
- Git directory path
- Project name

---

### 2. Print Deployment Header

**Source:** `deploy.rs:14`

```rust
println!("Deploying {} on {}...", style(&cfg.data.project_name).cyan().bold(), style(&cfg.data.host).cyan());
```

Displays the project name and target host.

---

### 3. Establish SSH Connection

**Source:** `deploy.rs:16`

```rust
let session = ssh::connect(&cfg).await?;
```

Connects to the remote server via SSH.

---

### 4. Run Pre-Receive Hook

**Source:** `deploy.rs:18-27`

```rust
println!("Running pre-receive...");
ssh::stream_cmd(
    &session,
    &format!(
        "BONES_FORCE_DEPLOY=1 GIT_DIR='{repo_path}' '{repo_path}/{}/{}' </dev/null",
        config::Constants::REMOTE_HOOKS_DIR,
        config::Constants::PRE_RECEIVE_HOOK
    ),
)
.await?;
```

#### 4.1 Environment Setup

**Environment Variables:**
- `BONES_FORCE_DEPLOY=1`: Signals to hooks that this is a forced/manual deployment (bypasses normal checks)
- `GIT_DIR='{repo_path}'`: Sets the Git directory for the bare repository

#### 4.2 Hook Execution

**Command Executed:**
```bash
BONES_FORCE_DEPLOY=1 GIT_DIR='/home/git/myapp.git' '/home/git/myapp.git/bones/hooks/pre-receive' </dev/null
```

**What `pre-receive` does:**
1. Validates the incoming push
2. Checks for deployment prerequisites
3. Stages a new release
4. Sets up the build workspace

**Note:** Input is redirected from `/dev/null` because manual deployment doesn't have actual Git refs being pushed.

#### 4.3 Output Streaming

**Source:** Uses `ssh::stream_cmd` instead of `ssh::run_cmd`:
- Streams stdout/stderr in real-time to local terminal
- Allows user to see deployment progress
- Fails if hook exits with non-zero status

---

### 5. Run Post-Receive Hook

**Source:** `deploy.rs:29-38`

```rust
println!("Running post-receive...");
ssh::stream_cmd(
    &session,
    &format!(
        "BONES_FORCE_DEPLOY=1 GIT_DIR='{repo_path}' '{repo_path}/{}/{}' </dev/null",
        config::Constants::REMOTE_HOOKS_DIR,
        config::Constants::POST_RECEIVE_HOOK
    ),
)
.await?;
```

#### 5.1 Hook Execution

**Command Executed:**
```bash
BONES_FORCE_DEPLOY=1 GIT_DIR='/home/git/myapp.git' '/home/git/myapp.git/bones/hooks/post-receive' </dev/null
```

**What `post-receive` does:**
1. Checks out the latest code to build workspace
2. Wires shared paths (symlinks for `.env`, `storage/`, etc.)
3. Runs deployment scripts
4. Activates the release (atomically switches symlink)
5. Restarts the service (if configured)
6. Prunes old releases

#### 5.2 Deployment Script Execution

The `post-receive` hook typically calls `bonesremote hooks deploy`, which:
1. Lists all scripts in `bones/deployment/`
2. Sorts them by filename (hence numeric prefixes)
3. Executes each script in order
4. Fails fast if any script exits non-zero

**Example Deployment Scripts:**
```bash
01_install_dependencies.sh
02_build_assets.sh
03_run_migrations.sh
04_cache_clear.sh
```

---

### 6. Close SSH Session

**Source:** `deploy.rs:40`

```rust
session.close().await?;
```

Cleanly closes the SSH connection.

---

### 7. Print Success Message

**Source:** `deploy.rs:42`

```rust
println!("\n{} Deployment complete.", style("Done!").green().bold());
```

---

## Difference from `git push`

| Aspect | `git push` | `bonesdeploy deploy` |
|--------|-----------|---------------------|
| **Trigger** | Push commits to remote | Manual command |
| **Pre-receive input** | Git refs (old/new SHA) | Empty (force mode) |
| **Commit requirement** | Must have new commits | Works with existing commits |
| **Use case** | Normal deployment workflow | Testing, re-deployment, recovery |

---

## When to Use

1. **Testing deployment scripts**: Validate changes to deployment scripts without pushing new commits
2. **Re-running failed deployments**: After fixing deployment script issues
3. **Manual deployments**: When you want to control deployment timing separately from commits
4. **Recovery**: Re-deploy the current commit after a failed deployment
5. **Initial setup**: Deploy after `bonesdeploy push` before the first `git push`

---

## How `BONES_FORCE_DEPLOY=1` Affects Hooks

The `BONES_FORCE_DEPLOY` environment variable signals to the hooks that this is a manual deployment. This typically:

1. **Skips push validation**: Normally `pre-receive` validates incoming refs, but with force deploy, it skips these checks
2. **Uses current branch**: Instead of using pushed refs, uses the configured branch from `bones.yaml`
3. **Allows re-deployment**: Can deploy the same commit multiple times

---

## Typical Workflow

### Normal Deployment (via Git Push)
```bash
# Make changes
git add .
git commit -m "Add feature"
git push production master  # Triggers deployment automatically
```

### Manual Deployment
```bash
# No new commits needed
bonesdeploy deploy  # Deploys current state of configured branch
```

### Combined Workflow
```bash
# Push without triggering deployment
git push production master --no-verify

# Deploy manually later
bonesdeploy deploy
```

---

## What Happens on the Server

### Directory State Before Deploy
```
/srv/deployments/myapp/
├── build/workspace/     # Previous build workspace
├── releases/
│   ├── 20260507_120000/ # Old release
│   ├── 20260507_130000/ # Old release
│   └── 20260507_140000/ # Current release
├── shared/
│   ├── .env
│   └── storage/
└── current -> releases/20260507_140000/
```

### Directory State After Deploy
```
/srv/deployments/myapp/
├── build/workspace/     # New build (overwritten)
├── releases/
│   ├── 20260507_130000/ # Old release
│   ├── 20260507_140000/ # Previous current
│   └── 20260507_150000/ # New release
├── shared/
│   ├── .env
│   └── storage/
└── current -> releases/20260507_150000/  # Switched
```

---

## Error Handling

If deployment fails:

1. **Pre-receive failure**: Build workspace not created, no release staged
2. **Post-receive failure**: 
   - Build workspace may be in partial state
   - Failed release is dropped
   - Current symlink remains pointing to previous release
   - Service continues running on old release

**Recovery:**
```bash
# Fix the issue (e.g., deployment script error)
bonesdeploy push  # Sync updated scripts
bonesdeploy deploy  # Retry deployment
```

---

## Related Commands

- `bonesdeploy push` - Syncs `.bones/` to remote
- `bonesdeploy rollback` - Reverts to previous release
- `bonesdeploy doctor` - Validates environment
- `bonesremote hooks deploy` - Server-side deployment command
