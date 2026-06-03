# bonesdeploy init

## Overview

Initializes bonesdeploy in the current Git repository by creating the `.bones/` scaffold structure, configuring deployment settings through interactive prompts, and setting up Git integration. This command must be run from within a Git repository and serves as the entry point for setting up deployment infrastructure.

## Detailed Execution Steps

### 1. Git Repository Verification

**Source:** `init.rs:15`

```rust
git::ensure_git_repository()?;
```

Validates that the current directory is a Git repository. This check ensures the command is being run in the correct context.

**Implementation:** Checks for the presence of a `.git` directory in the current working directory. If not found, the command exits with an error.

---

### 2. Scaffold Extraction

**Source:** `init.rs:17-34`

#### 2.1 Check for Existing `.bones/` Directory

```rust
let bones_dir = Path::new(config::Constants::BONES_DIR);
if bones_dir.exists() {
    println!(".bones/ already exists, skipping scaffold extraction.");
}
```

If `.bones/` already exists, skips the scaffold extraction step entirely. This prevents overwriting existing configuration.

#### 2.2 Template Selection

```rust
let available_templates = embedded::available_templates();
let selected_template = prompts::choose_template(&available_templates)?;
```

Presents the user with available templates (e.g., "node", "laravel", etc.) or allows choosing "none" for a build-from-scratch approach. Templates provide pre-configured hooks and deployment scripts for common frameworks.

#### 2.3 Scaffold Creation

```rust
println!("Creating .bones/ scaffold...");
embedded::scaffold(bones_dir)?;
```

Extracts embedded assets to `.bones/`:
- `.bones/bones.yaml` - Configuration file (empty/defaults)
- `.bones/hooks.sh` - Helper functions for hooks
- `.bones/hooks/` - Directory for Git hooks
  - `hooks/pre-receive` - Server-side hook
  - `hooks/post-receive` - Server-side hook
  - `hooks/pre-push` - Client-side hook
- `.bones/deployment/` - Directory for deployment scripts
- `.bones/remote/` - Directory for Ansible site setup
  - `site/playbooks/setup.yml` - Ansible playbook
  - `site/roles/` - Ansible roles directory

#### 2.4 Template Application

```rust
if let Some(template_name) = selected_template {
    embedded::scaffold_template(&template_name, bones_dir)?;
    println!("Applied template: {template_name}");
}
```

If a template was selected, applies template-specific files that may override or extend the base scaffold with framework-specific configurations and scripts.

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

**Source:** `init.rs:39-40`, `init.rs:163-176`

```rust
let bones_yaml = Path::new(config::Constants::BONES_YAML);
let cfg = load_or_collect_config(bones_yaml)?;
```

#### 4.1 Load or Prompt Logic

If `.bones/bones.yaml` exists and is complete (has required fields), loads it directly. Otherwise, prompts the user for missing configuration values.

**Required fields for "complete":**
- `remote_name` (e.g., "production")
- `project_name` (e.g., "myapp")
- `host` (e.g., "deploy.example.com")
- `repo_path` (e.g., "/home/git/myapp.git")

#### 4.2 Interactive Prompts

**Source:** `init.rs:64-116`

If configuration is incomplete, prompts for:

1. **Project Name** (`init.rs:69`)
   - Default: Current directory name
   - Used to derive default service user, web_root, and project_root

2. **Branch** (`init.rs:70`)
   - Default: `master`
   - The Git branch to deploy

3. **Remote Name** (`init.rs:71`)
    - Default: Pre-selects `production` if it exists among existing git remotes
    - The remote you choose must point to a **fresh VPS** that will serve as your production deployment target
    - Each remote is shown with its URL so you can distinguish code hosts from deployment targets
    - `origin` is marked with `— not a deployment remote` because it typically points to GitHub/GitLab/etc., not to your VPS
    - If a suitable remote doesn't exist yet, select `Create new deployment remote` at the bottom
    - This is the name of the Git remote that `bonesdeploy` will manage. It will be created locally if it doesn't exist, constructed from the deploy user, host, and repo path you provide
    - Example: choosing `production` creates `git@<host>:/home/git/<project>.git`

4. **Host** (`init.rs:72-74`)
   - Default: Inferred from existing remote URL (if available)
   - The deployment server hostname/IP

5. **Port** (`init.rs:75`)
   - Default: Inferred from existing remote URL (if available), otherwise `22`
   - SSH port for connecting to the server

6. **Git Directory** (`init.rs:76`, `init.rs:127-139`)
   - Default: `/home/git/<project_name>.git`
   - Path to the bare Git repository on the remote server
   - If remote already exists, infers from remote URL

7. **Public Path** (`init.rs:81`, `init.rs:144-161`)
    - Default: `/var/www/<project_name>`
    - Symlink pointing to the active release
    - Only stored in config if explicitly overridden

8. **Deploy Root** (`init.rs:82-83`)
   - Default: `/srv/deployments/<project_name>`
   - Directory containing all releases, build workspace, and shared files
   - Only stored in config if explicitly overridden

9. **Deploy on Push** (`init.rs:84`)
   - Default: `true`
   - Whether to trigger deployment on every push

10. **Permission Defaults** (`init.rs:85-89`)
    - Deploy User: `git` (user that runs deployment)
    - Service User: `<project_name>` (user that runs the application)
    - Group: `www-data`
    - Directory Mode: `750`
    - File Mode: `640`

11. **Releases** (`init.rs:90-94`)
    - Keep: `5` (number of releases to retain)
    - Shared Paths: `[".env", "storage"]` (paths to share across releases)

#### 4.3 Re-initialization Behavior

If `bones.yaml` already exists and is incomplete, the command:
- Loads the existing configuration
- Uses existing values as defaults for prompts
- Preserves any custom settings (path overrides, writable paths, etc.)
- Allows updating configuration without losing customizations

---

### 5. Save Configuration

**Source:** `init.rs:42-44`

```rust
config::save(&cfg, bones_yaml)?;
println!("Saved config to {}", config::Constants::BONES_YAML);
```

Writes the configuration to `.bones/bones.yaml` in YAML format.

**Special handling:**
- Omits `web_root` and `project_root` if they match project-derived defaults (keeps config clean)
- Includes runtime configuration comment block if runtime is default/empty
- Preserves SSL settings if previously configured

---

### 6. Configure Local Git Remote

**Source:** `init.rs:45`, `init.rs:220-229`

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

**Source:** `init.rs:48`, `init.rs:197-212`

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

### 8. Print Next Steps

**Source:** `init.rs:50-59`

Prints user guidance:
1. Run `bonesdeploy remote setup` before the first deploy (to provision the server)
2. Run `bonesdeploy push` after setup to sync `.bones/` to the remote

## Result

After successful execution:
- `.bones/` directory structure created
- `.bones/bones.yaml` configured with deployment settings
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
- `bonesdeploy remote setup` - Provisions server with Ansible
