# bonesdeploy remote setup

## Overview

Provisions the remote server for deployment by running a pyinfra deploy script (`.bones/infra/setup.py`) that sets up the required infrastructure: user accounts, Git bare repository, directory structure, firewall (UFW), and machine-level dependencies. Runs as `root` (or `BONES_BOOTSTRAP_SSH_USER` if set) since it needs to create system users and install packages. This command is typically run once per project during initial setup.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `remote_setup.rs`

```rust
let bones_yaml = Path::new(config::Constants::BONES_YAML);
let cfg = config::load(bones_yaml)?;
```

Loads deployment configuration from `.bones/bones.yaml` to determine:
- Remote server connection details (`host`, `port`)
- User accounts and permissions (`deploy_user`, `service_user`, `group`)
- Directory paths (`repo_path`, `project_root`, `web_root`)

---

### 2. Verify Deploy Script Exists

**Source:** `remote_setup.rs`

```rust
let deploy_path = Path::new(config::Constants::BONES_REMOTE_SETUP_DEPLOY);
if !deploy_path.is_file() {
    bail!("Missing remote setup deploy: {}", deploy_path.display());
}
```

Checks for the existence of `.bones/infra/setup.py`. This pyinfra deploy script is responsible for:
- Installing system packages
- Creating the bare Git repository
- Creating deploy and service users
- Setting up the directory structure
- Configuring firewall (UFW)
- Installing `bonesremote` from source

---

### 3. Resolve Bootstrap SSH User

**Source:** `remote_setup.rs`

```rust
fn resolve_bootstrap_ssh_user() -> String {
    if let Ok(user) = std::env::var("BONES_BOOTSTRAP_SSH_USER") {
        return user;
    }
    "root".to_string()
}
```

Defaults to `root` because setup operations need elevated privileges (user creation, package installation, firewall). Override with `BONES_BOOTSTRAP_SSH_USER` if your host uses a different initial user.

---

### 4. Ensure pyinfra is Installed

**Source:** `remote_setup.rs`

```rust
ensure_pyinfra_installed()?;
```

#### 4.1 Check for `pyinfra` Binary

First checks system PATH for `pyinfra`, then a bonesdeploy-managed environment at `~/.local/state/bonesdeploy/pyinfra/.venv/bin/pyinfra` (respecting `XDG_STATE_HOME` when set).

#### 4.2 Auto-Install pyinfra

If not found, automatically installs it into an isolated managed virtualenv:

1. Checks that `python3` is available
2. Checks that `python3 -m venv` is available
3. Creates a managed virtualenv under the bonesdeploy state root
4. Installs `pyinfra` into the virtualenv with its own `pip`
5. Verifies the managed `pyinfra` binary is now available

If any step fails, BonesDeploy prints explicit instructions for manual installation (e.g. `uv tool install pyinfra` or `pipx install pyinfra`).

Unlike the Ansible era, no remote Python bootstrap is needed — pyinfra installs are purely local.

---

### 5. Build Deployment Data

**Source:** `remote_setup.rs`

Constructs data variables passed to pyinfra via repeated `--data key=value` CLI flags. Nested objects (like `DeploymentPaths`) are flattened into dotted keys (e.g. `--data paths.repo=/home/git/myapp.git`). The deploy scripts unflatten these back to nested dicts before use.

The deploy user's public SSH key is resolved from `BONES_DEPLOY_PUBLIC_KEY_PATH` (falls back to `~/.ssh/id_ed25519.pub` → `id_ecdsa.pub` → `id_rsa.pub`). This key is installed as an authorized key for the deploy user so future connections (runtime, push, deploy) can connect without root.

---

### 6. Run pyinfra Deploy

**Source:** `remote_setup.rs`

Invokes pyinfra with the host directly (no temporary inventory file) and data passed as CLI flags:

```bash
pyinfra <host> .bones/infra/setup.py --ssh-user root --ssh-port 22 --data ssh_port=22 --data deploy_user=git --data paths.repo=/home/git/myapp.git --data paths.project_root=/srv/deployments/myapp ... -vv
```

The pyinfra deploy script performs these operations in order:

1. **System packages** — apt-get installs `build-essential`, `ca-certificates`, `curl`, `git`, `rsync`, `nginx`, `apparmor`, `apparmor-utils`, `certbot`, `ufw`, and any template-specific extras
2. **Git bare repository** — creates the parent directory, `git init --bare`, creates `bones/` subdirectory
3. **Placeholder release** — creates `project_root` structure, seeds a placeholder `index.html`, symlinks `current` → placeholder
4. **Rust toolchain & bonesremote** — installs `rustup`/`cargo`, builds and installs `bonesremote` from source, runs `bonesremote init --deploy-user <user>` to set up sudoers
5. **Users & groups** — creates `deploy_user` (shell access, home dir), `service_user` (system user, no login), and `group`, adds service user to group, creates web root parent with mode `2775`
6. **SSH key** — installs the deploy user's public key (if one was resolved)
7. **Firewall (UFW)** — enables UFW, allows SSH on the configured port, default-deny, rate-limited SSH

---

### 7. Handle Result and Print

If pyinfra exits non-zero, the command fails. On success:

```
Done! Site setup complete.
```

---

## What Gets Created

### Server Filesystem

```
/home/git/myapp.git/                 # bare Git repository
/home/git/myapp.git/bones/           # bones deploy config directory
/srv/deployments/myapp/              # project deployment root
/srv/deployments/myapp/releases/     # release directories
/srv/deployments/myapp/releases/19700101_000000/  # placeholder release
/srv/deployments/myapp/releases/19700101_000000/public/index.html
/srv/deployments/myapp/current -> releases/19700101_000000
/usr/local/bin/bonesremote           # server-side binary
/etc/sudoers.d/bonesdeploy           # sudoers drop-in for deploy user
```

### System Users

- `git` (deploy user) — shell access, ssh key auth, restricted sudo
- `myapp` (service user) — no shell, no login, owns deployed files

---

## When to Run

1. **First-time setup**: After `bonesdeploy init` and before the first deployment
2. **Server migration**: When moving to a new server
3. **Adding a deploy key**: When updating the deploy user's authorized key

---

## Typical Setup Workflow

```bash
# 1. Initialize project
bonesdeploy init

# 2. Provision server
bonesdeploy remote setup

# 3. Sync configuration to remote
bonesdeploy push

# 4. Configure runtime
bonesdeploy remote runtime

# 5. Deploy application
git push production master
```

---

## Prerequisites

### Local Machine
- Python 3 with venv support (e.g. `python3` + `python3-venv` on Debian/Ubuntu)
  - Or `uv` / `pipx` for manual pyinfra installation
- SSH client
- SSH key configured for root access to the target host

### Remote Server
- Debian/Ubuntu (the only supported platform)
- SSH root access (or `BONES_BOOTSTRAP_SSH_USER` configured)
- Internet access for package installation

---

## Customization

The setup process can be customized by editing `.bones/infra/setup.py`. The deploy script is written to `.bones/infra/` during `bonesdeploy init` and belongs to the user — edit freely.

### Adding System Packages

Edit the `SETUP_APT_PACKAGES` list in `.bones/infra/setup.py` or override via `setup_apt_packages` in your pyinfra data vars.

### Adding Pre-Package Hooks

Place a `pre_packages.py` file in `.bones/infra/` alongside `setup.py`. It will be loaded and its `pre_packages` function called before package installation.

---

## Error Scenarios

1. **pyinfra not installed**: Auto-installs into an isolated managed virtualenv
2. **Python 3 not available locally**: Install Python 3 first (with venv support)
3. **Python venv module missing**: Install `python3-venv` (Debian/Ubuntu) or equivalent, or install pyinfra manually via `uv`/`pipx`
4. **SSH connection failed**: Check host, port, and root SSH access
5. **pyinfra task failure**: pyinfra outputs detailed error message with target host
6. **Permission denied on remote**: Ensure bootstrap user has sudo

---

## Related Commands

- `bonesdeploy init` - Initialize project configuration
- `bonesdeploy remote runtime` - Configure per-site runtime after setup
- `bonesdeploy remote ssl` - Configure SSL certificates
- `bonesdeploy push` - Sync configuration to remote
- `bonesdeploy doctor` - Validate environment
