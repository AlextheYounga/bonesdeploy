# bonesdeploy init

## Overview

Initializes bonesdeploy in the current Git repository by creating the base `.bones/` scaffold structure, configuring deployment settings (interactively or non-interactively), and setting up Git integration. This command must be run from within a Git repository and serves as the entry point for setting up deployment infrastructure.

## Non-Interactive Mode

For CI/CD pipelines, agents, or scripted setups, use `--non-interactive` to skip all TTY prompts:

```bash
bonesdeploy init \
  --non-interactive \
  --project-name myapp \
  --host 203.0.113.10
```

Required flags in non-interactive mode:
- `--project-name` — project name (defaults to directory name if omitted, but explicit is preferred for agents)
- `--host` — server hostname or IP

Optional flags:

| Flag | Default | Description |
|------|---------|-------------|
| `--remote`, `-r` | `production` | Deployment remote name |
| `--branch` | `main` | Git branch to deploy |
| `--port` | `22` | SSH port |
| `--setup-remote` | `false` | Run `bonesdeploy remote setup` after init (skips confirmation prompt) |

Example with automatic remote setup:
```bash
bonesdeploy init \
  --non-interactive \
  --project-name myapp \
  --host 203.0.113.10 \
  --setup-remote
```

In interactive mode (no `--non-interactive`), these flags pre-fill prompt defaults. `--setup-remote` skips the final confirmation and goes straight to machine bootstrap. Framework runtime selection now happens in `bonesdeploy remote runtime`.

## Detailed Execution Steps

### 1. Git Repository Verification

**Source:** `init.rs:29`

```rust
git::ensure_git_repository()?;
```

Validates that the current directory is a Git repository. This check ensures the command is being run in the correct context.

**Implementation:** Checks for the presence of a `.git` directory in the current working directory. If not found, the command exits with an error.

---

### 2. Scaffold Extraction

**Source:** `init.rs:31-52`

#### 2.1 Check for Existing `.bones/` Directory

```rust
let bones_dir = Path::new(config::Constants::BONES_DIR);
if bones_dir.exists() {
    println!(".bones/ already exists, skipping scaffold extraction.");
}
```

If `.bones/` already exists, skips the scaffold extraction step entirely. This prevents overwriting existing configuration.

#### 2.2 Scaffold Creation

```rust
println!("Creating .bones/ scaffold...");
embedded::scaffold(bones_dir)?;
```

Extracts embedded assets to `.bones/`:
- `.bones/bones.toml` - Configuration file (empty/defaults)
- `.bones/hooks/hooks.sh` - Helper functions for hooks
- `.bones/hooks/` - Directory for Git hooks
  - `hooks/pre-receive` - Server-side hook
  - `hooks/post-receive` - Server-side hook
  - `hooks/pre-push` - Client-side hook
- `.bones/deployment/` - Directory for deployment scripts
This base scaffold contains hooks, deployment scripts, and shared config. Framework-specific runtime assets are created later by `bonesdeploy remote runtime`.

---

### 3. Update `.gitignore`

**Source:** `init.rs:36-37`, `init.rs:178-195`

```rust
update_gitignore()?;
```

Ensures `.bones/` is added to `.gitignore` to prevent committing deployment configuration to the repository.

**Implementation Details:**
1. Checks if `.gitignore` exists
2. If it exists, reads the content and checks if `.bones` is already present
3. If not present, appends `.bones` to the file (with newline handling)
4. If `.gitignore` doesn't exist, creates it with `.bones` as the first entry

---

### 4. Configuration Collection

**Source:** `init.rs:54-55`

```rust
let bones_toml = Path::new(config::Constants::BONES_TOML);
let cfg = load_or_collect_config(bones_toml)?;
```

#### 4.1 Load or Prompt Logic

If `.bones/bones.toml` exists and is complete (has required fields), loads it directly. Otherwise, prompts the user for missing configuration values.

**Required fields for "complete":**
- `remote_name` (e.g., "production")
- `project_name` (e.g., "myapp")
- `host` (e.g., "deploy.example.com")
- `repo_path` (e.g., "/home/git/myapp.git")

#### 4.2 Interactive Prompts

**Source:** `init.rs:161-218`

If configuration is incomplete, prompts for:

1. **Project Name**
   - Default: Current directory name
   - Used to derive default service user, web_root, and project_root

2. **Branch**
   - Default: `main` (interactive) / `main` (non-interactive default)
   - The Git branch to deploy

3. **Remote Name**
   - Default: Pre-selects `production` if it exists among existing git remotes
    - The remote you choose must point to a **fresh VPS** that will serve as your production deployment target
    - Each remote is shown with its URL so you can distinguish code hosts from deployment targets
    - `origin` is marked with `— not a deployment remote` because it typically points to GitHub/GitLab/etc., not to your VPS
    - If a suitable remote doesn't exist yet, select `Create new deployment remote` at the bottom
    - This is the name of the Git remote that `bonesdeploy` will manage. It will be created locally if it doesn't exist, constructed from the deploy user, host, and repo path you provide
    - Example: choosing `production` creates `git@<host>:/home/git/<project>.git`

4. **Host**
   - Default: Inferred from existing remote URL (if available)
   - The deployment server hostname/IP

5. **Port**
   - Default: Inferred from existing remote URL (if available), otherwise `22`
   - SSH port for connecting to the server

6. **Git Directory**
   - Default: `/home/git/<project_name>.git`
   - Path to the bare Git repository on the remote server
   - If remote already exists, infers from remote URL

7. **Public Path**
   - Default: `public`
   - Symlink pointing to the active release
   - Only stored in config if explicitly overridden

8. **Deploy Root**
   - Default: `/srv/deployments/<project_name>`
   - Directory containing all releases, build workspace, and shared files
   - Only stored in config if explicitly overridden

9. **Deploy on Push**
   - Default: `true`
   - Whether to trigger deployment on every push

10. **Permission Defaults**
    - Deploy User: `git` (user that runs deployment)
    - Service User: `<project_name>` (user that runs the application)
    - Group: `www-data`
    - Directory Mode: `750`
    - File Mode: `640`

11. **Releases**
    - Keep: `5` (number of releases to retain)
    - Shared Paths: `[".env", "storage"]` (paths to share across releases)

#### 4.3 Re-initialization Behavior

If `bones.toml` already exists and is incomplete, the command:
- Loads the existing configuration
- Uses existing values as defaults for prompts
- Preserves any custom settings (path overrides, writable paths, etc.)
- Allows updating configuration without losing customizations

---

### 5. Save Configuration

**Source:** `init.rs:57-59`

```rust
config::save(&cfg, bones_toml)?;
println!("Saved config to {}", config::Constants::BONES_TOML);
```

Writes the configuration to `.bones/bones.toml` in YAML format.

**Special handling:**
- Omits `web_root` and `project_root` if they match project-derived defaults (keeps config clean)
- Includes runtime configuration comment block if runtime is default/empty
- Preserves SSL settings if previously configured

---

### 6. Configure Local Git Remote

**Source:** `init.rs:60`, `init.rs:311-320`

```rust
ensure_local_remote(&cfg)?;
```

Checks if the configured remote name exists in the local Git repository. If not, adds it locally.

**Implementation:**
1. Checks if remote exists via `git remote show <name>`
2. If missing, constructs remote URL: `<deploy_user>@<host>:<repo_path>`
   - Example: `git@deploy.example.com:/home/git/myapp.git`
3. Adds remote via `git remote add <name> <url>`
4. Prints confirmation message

This does not create the server-side git user or bare repository. That still happens during `bonesdeploy remote setup` on the VPS.

---

### 7. Symlink Pre-Push Hook

**Source:** `init.rs:62`, `init.rs:286-294`

```rust
symlink_pre_push()?;
```

Creates a symlink from `.git/hooks/pre-push` to `../../.bones/hooks/pre-push`.

**Implementation:**
1. Ensures `.git/hooks/` directory exists
2. Removes existing `pre-push` hook if present
3. Creates symlink: `.git/hooks/pre-push` -> `../../.bones/hooks/pre-push`
4. This allows the hook to be versioned in `.bones/hooks/` while Git finds it in the expected location

**Why symlink?** The hook is versioned in `.bones/hooks/` (which is in `.gitignore`), but Git expects hooks in `.git/hooks/`. The symlink bridges this gap, allowing the hook to be:
- Shared across team members (via `bonesdeploy push`)
- Modified without editing files in `.git/`

---

### 8. Remote Setup Decision

**Source:** `init.rs:64-70`

```rust
if args.setup_remote || (!args.non_interactive && prompts::confirm_remote_setup()?) {
    remote_setup::run()?;
} else {
    print_follow_up_hint();
}
```

Three paths:
1. **`--setup-remote` flag** → runs remote setup automatically (no prompt)
2. **Interactive, no flag** → prompts "Set up the server now? [y/N]"
3. **Non-interactive, no flag** → skips remote setup, prints follow-up hint

Prints user guidance:
1. Run `bonesdeploy remote setup` before the first deploy (to provision the server)
2. Run `bonesdeploy push` after setup to sync `.bones/` to the remote

## Result

After successful execution:
- `.bones/` directory structure created
- `.bones/bones.toml` configured with deployment settings
- `.gitignore` updated to exclude `.bones/`
- Git remote added (if it didn't exist)
- Pre-push hook symlinked
- User is ready to proceed with site setup

## Error Scenarios

1. **Not a Git repository**: Command exits early with error
2. **rsync not installed**: Discovered during `push` command (not during init)
3. **SSH connection issues**: Not validated during init (discovered during `push` or `doctor`)
4. **Incomplete configuration**: Prompts user for missing values

## Related Commands

- `bonesdeploy push` - Syncs `.bones/` to remote bare repository
- `bonesdeploy doctor` - Validates local and remote setup
- `bonesdeploy remote setup` - Provisions server with pyinfra
