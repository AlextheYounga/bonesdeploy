# bonesdeploy deploy

## Overview

Manually triggers the full deployment sequence on the remote server by running `bonesremote deploy --config <remote_bones_toml>` over SSH. This is useful for testing deployment scripts, re-running failed deployments, or deploying when you don't have new commits to push.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `deploy.rs:9-12`

```rust
let bones_toml = Path::new(config::Constants::BONES_TOML);
let cfg = config::load(bones_toml)?;
let remote_bones_toml = cfg.data.deployment_paths().repo_bones_toml;
```

Loads local `.bones/bones.toml` and computes the remote config path (e.g., `/home/git/myapp.git/bones/bones.toml`).

---

### 2. Print Deployment Header

**Source:** `deploy.rs:14`

```rust
println!("Deploying {} on {}...",
    style(&cfg.data.project_name).cyan().bold(),
    style(&cfg.data.host).cyan());
```

---

### 3. Establish SSH Connection

**Source:** `deploy.rs:16`

```rust
let session = ssh::connect(&cfg).await?;
```

Connects to the remote server as the configured `deploy_user` (default: `git`) using the `openssh` crate.

---

### 4. Run Remote Deploy

**Source:** `deploy.rs:18-19`

```rust
println!("Running remote deploy...");
ssh::stream_cmd(&session,
    &format!("bonesremote deploy --config '{remote_bones_toml}'")).await?;
```

Runs `bonesremote deploy` on the remote host, streaming stdout/stderr to the local terminal. This is the main deployment command — it orchestrates the full server-side pipeline:

**Command Executed:**
```bash
bonesremote deploy --config '/home/git/myapp.git/bones/bones.toml'
```

#### 4.1 What `bonesremote deploy` Does (Server-Side Pipeline)

`bonesremote deploy` implements `run_full()` in `crates/bonesremote/src/commands/deploy.rs:21-43`:

1. **doctor** — Checks the server environment (binary availability, AppArmor, sudoers)
2. **stage_release** — Creates a timestamped release directory (e.g., `releases/20260507_150000/`), ensures `build/workspace` and `shared/` exist, writes staged release state
3. **post_receive** — Checks out the configured branch into `build/workspace` via:
   ```bash
   git --work-tree <project_root>/build/workspace --git-dir <repo_path> checkout -f <branch>
   ```
4. **wire_release** — Reads `runtime.toml` for shared path definitions, creates symlinks from `shared/` into `build/workspace` (e.g., `.env` → `shared/.env`)
5. **deploy** (inner `run` function) — See section 4.2
6. **restart_services** — Runs `sudo bonesremote service restart --config <config>` to restart the per-site nginx
7. **post_deploy** — Prunes old releases beyond the configured `releases.keep` count

On failure at any step from 3-7, **drop_failed_release** cleans up the staged release directory and state.

#### 4.2 Inner Deploy Step (`run()`)

**Source:** `crates/bonesremote/src/commands/deploy.rs:45-99`

1. Lists deployment scripts from `bones/deployment/` (sorted numerically)
2. For each script:
   - Runs it in `build/workspace` with environment variables (`PROJECT_NAME`, `PROJECT_ROOT`, `REPO_PATH`, `WEB_ROOT`)
   - Captures output to a log file in the release directory
   - Fails fast if any script exits non-zero
3. After all scripts succeed, copies `build/workspace` to the staged release directory via `cp -a`
4. Calls `activate_release::run()` to atomically switch the `current` symlink to the new release

**Example Deployment Scripts:**
```bash
01_install_dependencies.sh
02_build_assets.sh
03_run_migrations.sh
04_cache_clear.sh
```

---

### 5. Close SSH Session

**Source:** `deploy.rs:21`

```rust
session.close().await?;
```

---

### 6. Print Success Message

**Source:** `deploy.rs:23`

```rust
println!("\n{} Deployment complete.", style("Done!").green().bold());
```

---

## How `git push` Differs from `bonesdeploy deploy`

| Aspect | `git push` | `bonesdeploy deploy` |
|--------|-----------|---------------------|
| **Trigger** | Push commits to remote | Manual command |
| **Deploy mechanism** | `post-receive` hook calls `bonesremote deploy --revision <sha>` | Directly calls `bonesremote deploy --config <path>` (uses configured branch) |
| **Commit requirement** | Must have new commits | Works with existing commits (re-deployment) |
| **Pre-push doctor** | Runs locally via `pre-push` hook | Not run (user invokes deliberately) |
| **Use case** | Normal CI/CD workflow | Testing, re-deployment after failure, recovery |

---

## When to Use

1. **Testing deployment scripts**: Validate changes to `bones/deployment/` scripts without pushing new commits
2. **Re-running failed deployments**: After fixing deployment script issues
3. **Manual deployments**: When `deploy_on_push = false` or you want to control deployment timing separately from commits
4. **Recovery**: Re-deploy the current commit after a failed deployment
5. **Initial setup**: Deploy after `bonesdeploy push` before the first `git push`

---

## Typical Workflow

### Normal Deployment (via Git Push)
```bash
git add .
git commit -m "Add feature"
git push production master  # post-receive hook triggers bonesremote deploy
```

### Manual Deployment
```bash
bonesdeploy deploy  # SSH -> bonesremote deploy --config <path>
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
/srv/sites/myapp/
├── build/workspace/       # Previous build (git checkout)
├── releases/
│   ├── 20260507_120000/   # Old release
│   ├── 20260507_130000/   # Old release
│   └── 20260507_140000/   # Current (active) release
├── shared/                # Runtime-persistent files
│   ├── .env
│   └── storage/
└── current -> releases/20260507_140000/
```

### Directory State After Deploy
```
/srv/sites/myapp/
├── build/workspace/       # Overwritten with new checkout
├── releases/
│   ├── 20260507_130000/   # Old release
│   ├── 20260507_140000/   # Previous current
│   └── 20260507_150000/   # New release (timestamped)
├── shared/
│   ├── .env
│   └── storage/
└── current -> releases/20260507_150000/  # Atomically switched
```

---

## Error Handling

If the remote deploy fails:

1. **Pre-stage failure** (doctor/stage): No release state created, nothing to clean up
2. **Mid-deploy failure** (checkout/wire/scripts/activate):
   - `drop_failed_release` removes the staged release dir and clears state
   - `current` symlink remains pointing to the previous release
   - Service continues running on the old release

**Recovery:**
```bash
# Fix the issue (e.g., deployment script error)
bonesdeploy push     # Sync updated scripts to remote
bonesdeploy deploy   # Retry deployment
```

---

## Related Commands

- `bonesdeploy push` — Syncs `.bones/` to the remote bare repo
- `bonesdeploy rollback` — Reverts `current` symlink to the previous release
- `bonesdeploy doctor` — Validates local and remote environment
- `bonesremote deploy` — Server-side binary (runs remotely via SSH)
