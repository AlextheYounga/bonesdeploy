# bonesdeploy remote setup

## Overview

Provisions the remote server for deployment by delegating to the hidden `bonesinfra` checkout managed by `bonesdeploy`. This command prepares machine-level infrastructure: users, groups, the bare Git repository, deployment directories, firewall rules, system packages, and the server-side `bonesremote` binary.

The command normally connects as `root`. Set `BONES_BOOTSTRAP_SSH_USER` when the first SSH user is not `root` but has the privileges needed by bonesinfra.

## Detailed Execution Steps

### 1. Load Configuration

**Source:** `crates/bonesdeploy/src/commands/remote_setup.rs`

```rust
let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
let cfg = config::load(bones_toml)?;
let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;
```

Loads `.bones/bones.toml` and `.bones/runtime.toml` to gather connection details, project paths, identity names, and the selected runtime `web_root`.

---

### 2. Resolve Bootstrap SSH User

```rust
let ssh_user = bootstrap_ssh::resolve(Some(&cfg.ssh_user));
```

The bootstrap user resolves from `BONES_BOOTSTRAP_SSH_USER` when set, otherwise from `ssh_user` in `bones.toml`.

---

### 3. Build bonesinfra Input

```rust
let mut deploy_data = Value::Object(remote_data::base(&cfg, &runtime.web_root)?);
```

The command builds a JSON document from the local config and computed deployment paths, then adds the resolved `ssh_user` and `host`. This JSON is passed to bonesinfra on stdin; bonesdeploy does not flatten values into pyinfra `--data` flags.

---

### 4. Run bonesinfra Setup Apply

```rust
bonesinfra_cli::run_with_stdin(
    &["setup", "apply", "--config", bones_toml.to_str().unwrap_or(".bones/bones.toml")],
    &json,
)?;
```

The wrapper runs:

```bash
python -m bonesinfra setup apply --config .bones/bones.toml
```

with the deployment JSON on stdin. pyinfra is an internal implementation detail of the `bonesinfra` Python module, not something bonesdeploy invokes directly.

---

### 5. Handle Result

If bonesinfra exits non-zero, the command fails. On success:

```text
Done! Remote setup complete.
```

## What Gets Created

Exact operations live in the `bonesinfra` repo, but setup is responsible for the initial server contract:

```text
/home/git/myapp.git/                       # bare Git repository
/home/git/myapp.git/bones/                 # synced bones config directory
/srv/sites/myapp/                          # project deployment root
/srv/sites/myapp/releases/                 # release directories
/srv/sites/myapp/current                   # active release symlink
/usr/local/bin/bonesremote                 # server-side binary
/etc/sudoers.d/bonesdeploy                 # narrow sudoers drop-in
```

## System Identities

- `git` (deploy user by default) — shell access, owns the bare repo and release/build areas.
- `<project>` runtime user — no login, owns runtime-writable paths.
- `root` — owns system units, config directories, users, groups, and service provisioning.

## When to Run

1. First-time setup after `bonesdeploy init` and before the first deployment.
2. Server migration to a new host.
3. Re-applying machine-level provisioning after changing setup inputs.

## Typical Setup Workflow

```bash
bonesdeploy init
bonesdeploy remote setup
bonesdeploy remote runtime
bonesdeploy push
bonesdeploy deploy
```

## Prerequisites

### Local Machine

- Python available for running `python -m bonesinfra` through the bonesdeploy-managed bonesinfra checkout.
- SSH client.
- SSH access to the target host as `ssh_user` or `BONES_BOOTSTRAP_SSH_USER`.

### Remote Server

- Debian/Ubuntu.
- Internet access for package and Rust toolchain installation.
- Bootstrap user with privileges required for system provisioning.

## Customization

Setup behavior is customized in the `bonesinfra` repo (`https://github.com/AlextheYounga/bonesinfra.git`). bonesdeploy keeps a hidden checkout at `~/.config/bonesdeploy/bonesinfra/` and delegates setup through that module.

## Error Scenarios

1. **Missing local config**: run `bonesdeploy init` first.
2. **Missing runtime config**: ensure `.bones/runtime.toml` exists.
3. **bonesinfra failure**: inspect the streamed bonesinfra output.
4. **SSH connection failed**: check host, port, and bootstrap SSH user.
5. **Permission denied on remote**: ensure the bootstrap user has provisioning privileges.

## Related Commands

- `bonesdeploy init` - Initialize project configuration.
- `bonesdeploy remote runtime` - Configure per-site runtime after setup.
- `bonesdeploy remote ssl` - Configure SSL certificates.
- `bonesdeploy push` - Sync configuration to remote.
- `bonesdeploy doctor` - Validate environment.
