# bonesdeploy remote setup

## Overview

Provisions the remote server for deployment by running an Ansible playbook that sets up the required infrastructure, including user accounts, directory structures, permissions, and machine-level dependencies. This command is typically run once per project during initial setup.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `remote_setup.rs:13-14`

```rust
let bones_yaml = Path::new(config::Constants::BONES_YAML);
let cfg = config::load(bones_yaml)?;
```

Loads deployment configuration from `.bones/bones.yaml` to determine:
- Remote server connection details
- User accounts and permissions
- Directory paths
- SSL configuration

---

### 2. Verify Playbook Exists

**Source:** `remote_setup.rs:16-19`

```rust
let playbook = Path::new(config::Constants::BONES_REMOTE_SETUP_PLAYBOOK);
if !playbook.is_file() {
    bail!("Missing remote setup playbook: {}", playbook.display());
}
```

Checks for the existence of `.bones/setup/playbooks/setup.yml`.

**Expected location:** `.bones/setup/playbooks/setup.yml`

This playbook is responsible for:
- Creating deploy user
- Creating service user
- Setting up directory structure
- Configuring permissions
- Installing dependencies
- Bootstrapping shared machine roles like common tools, users, firewall, and SSL tooling

---

### 3. Ensure Ansible is Installed

**Source:** `remote_setup.rs:21`, `remote_setup.rs:139-170`

```rust
ensure_ansible_playbook_installed()?;
```

#### 3.1 Check for `ansible-playbook` Binary

**Source:** `remote_setup.rs:144-147`

```rust
if ansible_playbook_available(Path::new("ansible-playbook"))? {
    return Ok(PathBuf::from("ansible-playbook"));
}
```

First checks if `ansible-playbook` is available in system PATH.

#### 3.2 Check User Local Install

**Source:** `remote_setup.rs:149-154`

```rust
if let Some(local_ansible_playbook) = user_ansible_playbook_path()?
    && ansible_playbook_available(&local_ansible_playbook)?
{
    return Ok(local_ansible_playbook);
}
```

If not in system PATH, checks for user-local installation at `~/.local/bin/ansible-playbook`.

#### 3.3 Auto-Install Ansible

**Source:** `remote_setup.rs:155-158`

```rust
ensure_local_python3_available()?;
ensure_pip_available()?;
install_ansible_with_pip()?;
```

If Ansible is not found, automatically installs it:

1. **Check Python 3** (`remote_setup.rs:182-191`)
   ```bash
   python3 --version
   ```
   Ensures Python 3 is installed and functional.

2. **Ensure pip Available** (`remote_setup.rs:193-212`)
   ```bash
   python3 -m ensurepip --upgrade
   ```
   Installs pip if not available.

3. **Install Ansible** (`remote_setup.rs:223-239`)
   ```bash
   python3 -m pip install --user ansible
   ```
   Installs Ansible to user-local directory (`~/.local/`).

**User Notification:**
```
ansible-playbook not found. Installing Ansible with python3 -m pip install --user ansible...
```

#### 3.4 Final Verification

**Source:** `remote_setup.rs:159-168`

After installation, verifies `ansible-playbook` is now available. If still not found, fails with helpful message:
```
Installed Ansible with pip, but ansible-playbook is still unavailable. 
Ensure ~/.local/bin is in PATH.
```

---

### 4. Print Setup Header

**Source:** `remote_setup.rs:23-28`

```rust
println!(
    "Running {} against {} as {}...",
    style("site setup").cyan().bold(),
    style(&cfg.data.host).cyan(),
    style(&cfg.permissions.defaults.deploy_user).cyan(),
);
```

Displays target host and deploy SSH user.

---

### 5. Ensure Remote Python 3 is Available

**Source:** `remote_setup.rs:30`, `remote_setup.rs:108-137`

```rust
ensure_remote_python3_available(cfg, ssh_user)?;
```

Ansible requires Python 3 on the target host. This step ensures it's installed.

#### 5.1 Load Bootstrap Script

**Source:** `remote_setup.rs:110`

```rust
let script = embedded::read_asset(config::Constants::PYTHON_BOOTSTRAP_SCRIPT_ASSET)?;
```

Loads an embedded shell script designed to install Python 3 if missing.

**Asset:** `crates/bonesdeploy/scripts/bootstrap_python3.sh`

#### 5.2 Execute Bootstrap via SSH

**Source:** `remote_setup.rs:114-130`

```rust
let mut child = Command::new("ssh")
    .arg("-p")
    .arg(&cfg.data.port)
    .arg("-o")
    .arg("StrictHostKeyChecking=accept-new")
    .arg("-T")
    .arg(host)
    .arg("bash -s")
    .stdin(Stdio::piped())
    .spawn()
    .context("Failed to start remote python3 bootstrap command over SSH")?;

let mut stdin = child.stdin.take().context("Failed to open stdin for SSH process")?;
stdin.write_all(script.as_bytes()).context("Failed to send python3 bootstrap script over SSH")?;
drop(stdin);

let status = child.wait().context("Failed to run remote python3 bootstrap command over SSH")?;
```

**Process:**
1. Opens SSH connection to remote host
2. Pipes the bootstrap script to remote `bash -s`
3. Script checks for Python 3 and installs if missing
4. Waits for completion

**Bootstrap Script Logic:**
```bash
if ! command -v python3 >/dev/null 2>&1; then
    apt-get update && apt-get install -y python3 python3-pip
fi
```

---

### 6. Run Ansible Playbook

**Source:** `remote_setup.rs:30`, `remote_setup.rs:37-106`

```rust
run_ansible_playbook(&cfg, &cfg.permissions.defaults.deploy_user, extra_vars, &playbook_args)?;
```

#### 6.1 Verify Playbook and Roles

**Source:** `remote_setup.rs:38-46`

```rust
let playbook = Path::new(config::Constants::BONES_REMOTE_SETUP_PLAYBOOK);
if !playbook.is_file() {
    bail!("Missing remote setup playbook: {}", playbook.display());
}

    let roles_dir = Path::new(config::Constants::BONES_REMOTE_ROLES_DIR);
    if !roles_dir.is_dir() {
        bail!("Missing setup roles directory: {}", roles_dir.display());
    }
```

Ensures both the playbook and roles directory exist before proceeding.

#### 6.2 Resolve Paths and Variables

**Source:** `remote_setup.rs:48-57`

```rust
let project_root_parent = resolve_project_root_parent(&cfg.data.project_root);
let inventory = format!("{},", cfg.data.host);
let roles_path = env::var("ANSIBLE_ROLES_PATH")
    .ok()
    .filter(|value| !value.is_empty())
    .map_or_else(|| roles_dir.display().to_string(), |existing| format!("{}:{existing}", roles_dir.display()));
```

**Variables:**
- `project_root_parent`: Parent directory of the deployment root (e.g., `/srv/deployments` from `/srv/deployments/myapp`)
- `inventory`: Comma-separated host list (trailing comma required for single host)
- `roles_path`: Colon-separated path to Ansible roles (respects existing `ANSIBLE_ROLES_PATH`)

---

### 7. Construct Ansible Command

**Source:** `remote_setup.rs:57-96`

```rust
let ansible_playbook_binary = resolve_ansible_playbook_binary()?;
let mut command = Command::new(&ansible_playbook_binary);
command.env("ANSIBLE_ROLES_PATH", roles_path);
command
    .arg("-i")
    .arg(&inventory)
    .arg("-u")
    .arg(ssh_user)
    .arg("-e")
    .arg(format!("ansible_port={}", cfg.data.port))
    .arg("-e")
    .arg(format!("deploy_user={}", cfg.permissions.defaults.deploy_user))
    .arg("-e")
    .arg(format!("service_user={}", cfg.permissions.defaults.service_user))
    .arg("-e")
    .arg(format!("group={}", cfg.permissions.defaults.group))
    .arg("-e")
    .arg(format!("project_root_parent={project_root_parent}"))
    .arg("-e")
    .arg(format!("web_root={}", cfg.data.web_root))
    .arg("-e")
    .arg(format!("project_root={}", cfg.data.project_root))
    .arg("-e")
    .arg(format!("project_name={}", cfg.data.project_name))
    .arg("-e")
    .arg(format!("repo_path={}", cfg.data.repo_path));
```

#### 7.1 Ansible Flags

- `-i {inventory}`: Inventory file (single host)
- `-u {ssh_user}`: SSH user to connect as
- `-e {key}={value}`: Extra variables passed to playbook

#### 7.2 Variables Passed to Playbook

| Variable | Value | Purpose |
|----------|-------|---------|
| `ansible_port` | `{port}` | SSH port |
| `deploy_user` | `git` (default) | User that runs deployments |
| `service_user` | `{project_name}` | User that runs the application |
| `group` | `www-data` | Group for files |
| `project_root_parent` | `/srv/deployments` | Parent of deployment root |
| `web_root` | `public` | Relative path served from `current` |
| `project_root` | `/srv/deployments/{project}` | Deployment root with `current`, `releases`, `shared`, and `build` |
| `project_name` | `{project}` | Project identifier |
| `repo_path` | `/home/git/{project}.git` | Bare repository path |

---

### AppArmor Verification Runbook (Linux Host)

After `bonesdeploy remote setup` and `bonesdeploy remote runtime` succeed, verify AppArmor provisioning and service binding:

```bash
bonesdeploy remote setup
ssh -p <port> <bootstrap-user>@<host> "systemctl is-active apparmor"
ssh -p <port> <bootstrap-user>@<host> "cat /sys/module/apparmor/parameters/enabled"
ssh -p <port> <bootstrap-user>@<host> "grep '^profile bonesdeploy-<project>-nginx ' /etc/apparmor.d/bonesdeploy-<project>-nginx"
ssh -p <port> <bootstrap-user>@<host> "systemctl cat <project>-nginx.service | grep -E 'AppArmorProfile|After=|Requires='"
ssh -p <port> <bootstrap-user>@<host> "systemctl is-active <project>-nginx"
```

Expected:
- `apparmor` service is `active`
- kernel parameter reports enabled (`Y`, `y`, `1`, or `yes`)
- `/etc/apparmor.d/bonesdeploy-<project>-nginx` exists and defines the profile
- `<project>-nginx.service` includes `AppArmorProfile=bonesdeploy-<project>-nginx` and declares `apparmor.service` in both `After=` and `Requires=`
- `<project>-nginx` is `active`

---

### 8. Add SSL Configuration (If Enabled)

**Source:** `remote_setup.rs:84-94`

```rust
if cfg.ssl.enabled && !cfg.ssl.domain.is_empty() {
    command
        .arg("-e")
        .arg(format!("nginx_server_name={}", cfg.ssl.domain))
        .arg("-e")
        .arg("nginx_ssl_enabled=true")
        .arg("-e")
        .arg(format!("nginx_ssl_certificate_path=/etc/letsencrypt/live/{}/fullchain.pem", cfg.ssl.domain))
        .arg("-e")
        .arg(format!("nginx_ssl_certificate_key_path=/etc/letsencrypt/live/{}/privkey.pem", cfg.ssl.domain));
}
```

If SSL is enabled and domain is configured, passes SSL-related variables to the playbook for Nginx configuration.

---

### 9. Execute Playbook

**Source:** `remote_setup.rs:96-99`

```rust
command.args(extra_args);
command.arg(playbook);

let status = command.status().context("Failed to run ansible-playbook")?;
```

**Example Command:**
```bash
ANSIBLE_ROLES_PATH=/path/to/.bones/setup/roles \
ansible-playbook \
  -i "deploy.example.com," \
  -u git \
  -e "ansible_port=22" \
  -e "deploy_user=git" \
  -e "service_user=myapp" \
  -e "group=www-data" \
  -e "project_root_parent=/var/www" \
  -e "web_root=/var/www/myapp" \
  -e "project_root=/srv/deployments/myapp" \
  -e "project_name=myapp" \
  -e "repo_path=/home/git/myapp.git" \
   .bones/setup/playbooks/setup.yml
```

#### 9.1 Playbook Execution

The playbook typically includes tasks for:

1. **User Management**
   - Create deploy user (`git`)
   - Create service user (`myapp`)
   - Configure sudoers for passwordless execution

2. **Git Repository**
   - Initialize bare git repository at `repo_path`
   - Set up directory structure for hooks

3. **Directory Structure**
    - Create `project_root` (`/srv/deployments/myapp`)
    - Create `current` and `releases` directories
   - Create initial placeholder release
   - Set up shared directory
   - Configure permissions

4. **Dependencies**
   - Install system packages
   - Install runtime dependencies (Node.js, PHP, etc.)
   - Configure package managers

5. **Per-Site Nginx**
   - Install Nginx
    - Configure machine-level dependencies and shared bootstrap roles

6. **SSL (if enabled)**
   - Set up SSL certificates

---

### 10. Handle Playbook Result

**Source:** `remote_setup.rs:101-103`

```rust
if !status.success() {
    bail!("ansible-playbook failed with status {status}");
}
```

If the playbook fails, the command exits with an error.

---

### 11. Print Success Message

**Source:** `remote_setup.rs:32`

```rust
println!("\n{} Site setup complete.", style("Done!").green().bold());
```

---

## When to Run

1. **First-time setup**: After `bonesdeploy init` and before the first deployment
2. **After adding SSL**: When enabling SSL configuration
3. **Server migration**: Setting up a new server
4. **Infrastructure changes**: When modifying `bones/site/` playbooks or roles

---

## Typical Setup Workflow

```bash
# 1. Initialize project
bonesdeploy init

# 2. Provision server
bonesdeploy remote setup

# 3. Sync configuration to remote
bonesdeploy push

# 4. Deploy application
git push production master
```

---

## Customization

The setup process is highly customizable through Ansible:

### Custom Playbooks

Edit `.bones/setup/playbooks/setup.yml` to add custom setup tasks:
```yaml
---
- hosts: all
  become: yes
  roles:
    - common
    - deploy_user
    - service_user
    - ssl
    - ssl
  tasks:
    - name: Custom task
      # Your custom setup task
```

### Custom Roles

Add roles to `.bones/setup/roles/`:
```
.bones/setup/roles/
├── common/
├── deploy_user/
├── service_user/
├── nginx/
├── ssl/
└── custom_role/
```

---

## Prerequisites

### Local Machine
- Python 3
- pip (or `python3-venv`)
- SSH client
- SSH key configured for remote access

### Remote Server
- SSH access
- sudo privileges (for initial setup)
- Internet access (for package installation)

---

## Error Scenarios

1. **Ansible not installed**: Auto-installs via pip
2. **Python 3 not available locally**: Install Python 3 first
3. **Python 3 not available remotely**: Bootstrap script installs it
4. **SSH connection failed**: Check SSH configuration
5. **Playbook syntax error**: Fix playbook errors
6. **Task failure**: Ansible outputs detailed error message

---

## Related Commands

- `bonesdeploy init` - Initialize project configuration
- `bonesdeploy remote ssl` - Configure SSL certificates
- `bonesdeploy push` - Sync configuration to remote
- `bonesdeploy doctor` - Validate environment
