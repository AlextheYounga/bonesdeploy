# BonesDeploy v3

A Rust CLI that compiles into a single binary, containing embeds of boilerplate scripts along with other git remote helpers. It produces two executables: `bonesdeploy` (local CLI for setup and management) and `bonesremote` (server-side tool for remote operations, installed on the deployment host).

## Deployment Methodology
We have an SSH deployment user (normally `git`) that handles deployment concerns. This user has a home folder, restricted sudo ability, but no password login. We also have a per-project service user (defaults to the project name). This user has no home folder, no login, and no sudo ability. This is ultimately who we want to own our project files to limit attack scope.

### Common Problems
- Shared groups have too many logic traps. My apps should not have 660 or 770 permissions on all files so that a `git` user can have read/write.
- I don't like ACLs; they're far too opaque.
- Setting up inotify systems are cumbersome.

### Proposed Solution of This Project
We create a `bonesremote` executable that does not require a password and allows it to change ownership to a deploy user and harden back the permissions based on what is configured under `permissions` in bones.toml. Running `bonesremote init` on the remote (as sudo) installs a drop-in file at `/etc/sudoers.d/bonesdeploy`, allowing the `git` user to run bonesremote without password.

## Bones Scaffolding
.bones  
├── bones.toml  
├── hooks.sh
├── server
│   ├── nginx
│   │   └── site.conf.j2
│   ├── playbooks
│   │   └── setup.yml
│   └── roles
│       └── nginx
│           └── defaults
│               └── index.html.j2
├── deployment  
│   ├── 01_run_deployment_concerns.sh
│   └── 02_permissions_lockup.sh (example)  
└── hooks  
    ├── post-receive
    ├── pre-push  
    └── pre-receive  

### Bones Toml
This stores crucial data we will need and is collected on running `bonesdeploy init` via user prompts.  
Collects only the following project information from the user:
- `remote_name`: selection from existing git remotes
- `project_name`: str
- `branch`: str

Everything else is inferred or defaulted for Debian/Ubuntu-first usability:
- `host`: inferred from selected remote URL
- `port`: defaults to `22`
- `git_dir`: inferred from selected remote URL when possible, else defaults to `/home/git/{project_name}.git`
- `live_root`: defaults to `/var/www/{project_name}`
- `deploy_root`: defaults to `/srv/deployments/{project_name}`
- `deploy_on_push`: defaults to `true`
- `permissions.defaults.deploy_user`: defaults to `git`
- `permissions.defaults.service_user`: defaults to `{project_name}`
- `permissions.defaults.group`: defaults to `www-data`
- `permissions.defaults.dir_mode`: defaults to `750`
- `permissions.defaults.file_mode`: defaults to `640`
- `releases.keep`: defaults to `5`
- `releases.shared_paths`: defaults to [`.env`, `storage`]

Example `bones.toml`:
```toml
[data]
remote_name = "production"
project_name = "lawsnipe"
host = "deploy.example.com"
port = "22"

git_dir = "/home/git/lawsnipe.git"  
live_root = "/var/www/lawsnipe"  
deploy_root = "/srv/deployments/lawsnipe"
branch = "master"  
deploy_on_push = true

# These are the permissions that ultimately get applied to every file post-deployment.  
[permissions.defaults]  
deploy_user = "git"  
service_user = "lawsnipe"  
group = "www-data"  
dir_mode   = "750"  
file_mode  = "640"  

# Path overrides for fine-grained permission control.
# recursive = true: applies 'mode' to the directory and all files/directories under it.
# type = "dir": applies 'mode' to just that directory.
# type = "file": applies 'mode' to just that file.
[[permissions.paths]]
path      = "storage"
mode      = "770"
recursive = true  

[[permissions.paths]]  
path      = "bootstrap/cache"  
mode      = "770"  
recursive = true  

[[permissions.paths]]  
path      = "database"  
mode      = "770"  
type      = "dir"  

[[permissions.paths]]  
path      = "database/database.sqlite"  
mode      = "660"  
type      = "file"  

[releases]
keep = 5
shared_paths = [".env", "storage"]

# Optional runtime launcher settings (only for service/landlock-managed apps).
# [runtime]
# command = ["/usr/bin/node", "server.js"]
# working_dir = "."
# writable_paths = []

[ssl]
enabled = true
domain = "app.example.com"
email = "ops@example.com"
```

### Hooks
Hooks are static shell scripts embedded in the `bonesdeploy` binary. They are written to `.bones/hooks/` once during `bonesdeploy init`, and they source shared functions from `.bones/hooks.sh`. After that, they belong to the user and can be edited freely. They are synced to the remote bare repo via `bonesdeploy push`.

- `pre-push` => Local hook, symlinked to `.git/hooks/pre-push`. This checks to see if we are pushing to our bonesdeploy designated remote. If so, then we run `bonesdeploy doctor` and we fail if the doctor command expresses any warning or errors.  
- `pre-receive` => Runs `bonesremote doctor --config ...` and fails on issues, then runs `sudo bonesremote release stage --config ...` to prepare build/runtime directories and write staged release state.
- `post-receive` => Runs the full deployment pipeline by calling, in order, `bonesremote hooks post-receive --config ...`, `bonesremote hooks deploy --config ...`, and `bonesremote hooks post-deploy --config ...`.

### Deployment Folder
This folder stores deployment scripts that are run by `bonesremote hooks deploy`. Files in this folder must be ordered sequentially like `01_run_deployment_concerns.sh` and `02_lockup_permissions.sh`. They are named in numerical order and all of these scripts are always run.

## Crate Structure
This Cargo workspace has two crates under `crates/`, each with its own dependencies. There is no shared lib crate; the `bones.toml` structs are duplicated since each binary discovers and uses config differently.

```
bonesdeploy/
├── Cargo.toml                  # workspace root
├── kit/                        # embedded assets (scaffolding templates)
│   ├── bones.toml
│   ├── deployment/
│   └── hooks/
├── crates/
│   ├── bonesdeploy/               # local CLI binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs         # clap setup, command dispatch
│   │       ├── commands/
│   │       │   ├── mod.rs
│   │       │   ├── init.rs
│   │       │   ├── doctor.rs
│   │       │   ├── push.rs
│   │       │   └── version.rs
│   │       ├── config.rs       # bones.toml structs + load/save + local file discovery
│   │       ├── embedded.rs     # rust-embed from kit/, scaffold writing
│   │       ├── git.rs          # git CLI operations: remote validation, repo checks
│   │       ├── prompts.rs      # interactive user input collection, returns config
│   │       └── ssh.rs          # openssh session management + rsync
│   └── bonesremote/        # server-side binary
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── commands/
│           │   ├── mod.rs
│           │   ├── init.rs
│           │   ├── doctor.rs
│           │   ├── stage_release.rs
│           │   ├── wire_release.rs
│           │   ├── activate_release.rs
│           │   ├── drop_failed_release.rs
│           │   ├── rollback.rs
│           │   ├── post_receive.rs
│           │   ├── deploy.rs
│           │   ├── post_deploy.rs
│           │   └── version.rs
│           ├── config.rs       # bones.toml structs + remote file discovery
│           └── permissions.rs  # chown/chmod logic
└── docs/
```

### BonesDeploy CLI Commands
- **init**:
  - Informs the user that there should be a remote git url set up, explains what's going to happen, and then asks the user for permission to proceed.
  - Gets or creates the `.bones` folder with our default scaffolding.
  - Updates `.gitignore` to add .bones folder.
  - Loads existing config from `.bones/bones.toml` or collects minimal user input via prompts (`project_name`, `branch`, deployment remote).
  - Infers host and git directory from the selected remote URL when possible, and applies opinionated defaults for all other settings.
  - Creates upstream bare repository on remote using the url set in `git remote [remote_name]`, setting it up if it doesn't exist. We fail if no git remote URL is set.
  - Builds and uploads post-receive hook to remote.
  - Saves config to `.bones/bones.toml`.

- **doctor**
  - This command checks all concerns in your local environment.
  - Loads config from `.bones/bones.toml`
  - Runs local checks:
    - `.bones` folder is set up correctly. Deployment scripts are named appropriately.
    - Local `pre-push` hook is symlinked properly.
  - Runs remote checks (skipped with `--local`):
    - `bonesremote` executable exists on remote and can be run globally.
    - `{project_name}.git/bones` folder exists on remote (needs `bonesdeploy push` warning)
    - `{project_name}.git/bones/hooks` matches with `{project_name}.git/hooks` inside the remote bare repo.
  - The `--local` flag skips all remote checks. The `pre-push` hook uses this flag since the remote is independently validated by `bonesremote doctor` in the `pre-receive` hook.

- **push**
  - Uses an `rsync -av --delete` command to push up our local `.bones` folder to the bare repo.
  - We will create a `bones` folder under our `{project_name}.git/` folder so that everything is self-contained inside git.
  - Deletes sample git hooks in bare repo, so that our files will be the only files to worry about in the `{project_name}.git/hooks` folder.
  - Runs commands on remote that symlinks our `{project_name}.git/bones/hooks` files are symlinked with `{project_name}.git/hooks` properly.

- **deploy**
  - Manually runs remote `pre-receive` and `post-receive` hooks over SSH without pushing commits.
  - Sets `BONES_FORCE_DEPLOY=1` so manual deploy runs even when `deploy_on_push = false`.

- **server setup**
  - Runs `.bones/server/playbooks/setup.yml` locally using `ansible-playbook` against the configured host.
  - Passes `project_name`, `deploy_user`, `service_user`, `group`, `live_root_parent`, `live_root`, `git_dir`, and `runtime_config_path` from `bones.toml` as playbook variables.
  - Installs nginx and provisions a project default site from `.bones/server/nginx/site.conf.j2`.
  - Seeds a placeholder page from `.bones/server/roles/nginx/defaults/index.html.j2` so the host serves a branded default page before first deployment.

- **server ssl**
  - Runs the SSL Ansible role against the configured host.
  - Uses certbot with a webroot challenge to obtain/renew certificates for the configured domain.
  - Re-renders `.bones/server/nginx/site.conf.j2` with TLS enabled, listening on 443 and redirecting HTTP to HTTPS.

- **version**:
  - Echoes "bonesdeploy 0.1.0".

### BonesDeployRemote CLI Commands
- **Release commands** live under `bonesremote release ...`
- **Hook commands** live under `bonesremote hooks ...`
- **init**:
  - Must be run as sudo.
  - Installs a drop-in file at `/etc/sudoers.d/bonesdeploy` to allow only privileged `bonesremote` commands without requiring password.
- **doctor**:
  - Checks to see if the server is set up properly:
    - `bonesremote` can be run without requiring password
    - `bonesremote` is globally available.
    - Landlock support is available on the host.
  - With `--config`, validates runtime readiness only when `runtime.command` is configured (`service_user`, runtime tree, and systemd unit).
- **release stage**
	- Creates a staged runtime tree under `runtime/`, ensures `build/workspace` and `shared/`, then writes staged release state before checkout.
- **release wire**
	- Wires shared paths into `build/workspace` after checkout.
- **release activate**
	- Atomically switches `current` to the staged release and clears staged release state.
- **release drop-failed**
	- Deletes a failed staged release and clears staged release state.
- **release rollback**
	- Repoints `current` to the previous release.
- **hooks post-receive**
	- Checks out the configured branch into `build/workspace` and wires shared paths.
- **hooks deploy**
	- Runs deployment scripts in `build/workspace`, copies runtime-ready output into staged `runtime/<timestamp>`, drops failed staged releases on error, and activates release on success.
- **landlock exec**
	- Resolves `live_root` to the active runtime tree, applies Landlock policy, and `exec`s `runtime.command`.
- **hooks post-deploy**
	- Runs a permissions hardening function setting all permissions back to the layout configured in `bones.toml`, like for instance setting everything back to be owned by the service user, then prunes old releases. 
- **version**:
  - Echoes "bonesdeploy 0.1.0".

## Security Notes
- Sudo access for the deployment user is strictly limited to passwordless execution of `bonesremote release stage` and `bonesremote hooks post-deploy` via /etc/sudoers configuration.
- No broader sudo privileges are granted.
- All operations are audited through system logs.

## Flow
- User runs `bonesdeploy init`, and the procedures outlined above are executed.
- User can make any changes to their deployment scripts or hooks in `.bones/` (e.g., customizing `deployment/` files or adding project-specific logic).
- User runs `bonesdeploy push` to sync the `.bones/` folder to the remote bare repo.
- User runs `git push production master` or some similar command where the remote name aligns with our bones.toml configuration.
- The `pre-push` hook checks to see if we are pushing to our bones remote (in this example, `production`). If so, it runs `bonesdeploy doctor` and fails on warnings/errors.

### Hook Event Order on `git push`

`pre-push -> pre-receive -> post-receive`

1. Git receives pack data in the remote bare repo and runs `pre-receive`.
2. If `deploy_on_push = true`, `pre-receive` runs `bonesremote doctor --config "$BONES_TOML"`, then `sudo bonesremote release stage --config "$BONES_TOML"`.
   - If `deploy_on_push = false`, `pre-receive` exits early and no deploy steps run.
3. If `pre-receive` exits successfully, Git updates refs.
4. Git runs `post-receive`.
5. `post-receive` runs `bonesremote hooks post-receive --config "$BONES_TOML"` (checkout to `build/workspace` + shared wiring).
6. Then `post-receive` runs `bonesremote hooks deploy --config "$BONES_TOML"` (deployment scripts in build workspace + runtime publish + activate/drop-failed).
7. Finally `post-receive` runs `sudo bonesremote hooks post-deploy --config "$BONES_TOML"` (permission hardening + pruning).

## Cargo Dependencies
- clap
- inquire
- rust-embed
- toml
- rsync
- openssh
- serde (derive)
- tokio
- console
- nix
- walkdir
- anyhow
