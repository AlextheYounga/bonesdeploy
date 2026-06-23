# bonesdeploy update

## Overview

Updates the BonesDeploy control-plane binaries from the `master` branch of `https://github.com/AlextheYounga/bonesdeploy.git`.

- Local update installs `bonesdeploy` with `cargo install --locked --git`.
- Remote update SSHes to the configured host as `root` and installs `bonesremote` with `cargo install --locked --git`.
- Existing site runtime components, application code, databases, nginx config, and release directories are not redeployed by this command.

There is no GitHub release API lookup, binary asset download, checksum file, or `/opt/bonesdeploy/versions` symlink flip in the current implementation.

## Command Signature

```bash
bonesdeploy update [--skip-local] [--skip-remote]
```

**Flags:**

- `--skip-local`: Skip updating the local `bonesdeploy` binary.
- `--skip-remote`: Skip updating the remote `bonesremote` binary.

## Detailed Execution Steps

### 1. Print Current Versions

**Source:** `crates/bonesdeploy/src/commands/update.rs:23-29`

```rust
let current_local = update_release::current_local_version();
let current_remote = update_release::current_remote_version();
```

Local version comes from `CARGO_PKG_VERSION`. Remote version is read by running `bonesremote version` over SSH when `.bones/bones.toml` is available; otherwise it reports `unknown`.

---

### 2. Clone the Source Repository

**Source:** `crates/bonesdeploy/src/commands/update.rs:36-43`

```rust
let source_dir = clone_master_source(temp_path)?;
let master_versions = read_master_versions(&source_dir)?;
```

The command creates a temporary directory and runs:

```bash
git clone --depth 1 --branch master https://github.com/AlextheYounga/bonesdeploy.git <temp>/source
```

It then reads package versions from:

- `crates/bonesdeploy/Cargo.toml`
- `crates/bonesremote/Cargo.toml`

---

### 3. Update Local bonesdeploy

Skipped when `--skip-local` is set. If the installed local version matches the cloned `bonesdeploy` version, the binary install is skipped.

When an update is needed:

```rust
update_release::update_local_from_source(SOURCE_REPO_URL)?;
```

which runs:

```bash
cargo install --locked --git https://github.com/AlextheYounga/bonesdeploy.git bonesdeploy --force
```

After the local update check, the command refreshes local scaffold files when `.bones/` exists:

```rust
refresh_local_bones_from_source(&source_dir, Path::new(paths::LOCAL_BONES_DIR))?;
```

This syncs:

- `crates/bonesdeploy/kit/hooks` -> `.bones/hooks`
- the selected runtime deployment template -> `.bones/deployment`
- or `crates/bonesdeploy/kit/deployment` when no selected runtime deployment template exists

It does not overwrite `.bones/bones.toml` or `.bones/runtime.toml`.

---

### 4. Update Remote bonesremote

Skipped when `--skip-remote` is set. If the remote version matches the cloned `bonesremote` version, the remote install is skipped.

When an update is needed:

```rust
update_release::update_remote_from_source(SOURCE_REPO_URL, &master_versions.bonesremote).await?;
```

The remote updater requires `.bones/bones.toml`, connects as `root`, and runs:

```bash
cargo install --locked --git https://github.com/AlextheYounga/bonesdeploy.git bonesremote --force --root /usr/local
```

It also ensures the project root parent exists with the current default permissions:

```bash
mkdir -p /srv/sites && chown root:root /srv/sites && chmod 711 /srv/sites
```

The installed binary is available through Cargo's install root at `/usr/local/bin/bonesremote`.

## What This Command Does Not Do

- Does not query the GitHub releases API.
- Does not download release tarballs or checksum files.
- Does not verify SHA256 checksum manifests.
- Does not maintain versioned binary directories under `/opt/bonesdeploy`.
- Does not perform an atomic symlink flip.
- Does not invoke pyinfra or bonesinfra.
- Does not deploy application code or restart application services.

## When to Run

1. After upstream BonesDeploy changes are available on `master`.
2. Before changing deployment workflows that depend on new command behavior.
3. When `bonesdeploy version` or `bonesremote version` shows an older installed version.

## Rollback

There is no built-in binary rollback mechanism in the current update implementation. To roll back, reinstall a known-good revision manually with Cargo, for example:

```bash
cargo install --locked --git https://github.com/AlextheYounga/bonesdeploy.git --rev <commit> bonesdeploy --force
```

For the remote binary, run the equivalent `cargo install` on the server as root for `bonesremote`.

## Prerequisites

### Local

- `git`
- Rust toolchain with `cargo`
- Network access to `https://github.com/AlextheYounga/bonesdeploy.git`

### Remote

- `.bones/bones.toml` in the local project when updating remote.
- SSH access to the configured host as `root`.
- Rust toolchain with `cargo` on the remote server.
- Network access from the remote server to GitHub.

## Related Commands

- `bonesdeploy doctor` — Validate local and remote environment.
- `bonesdeploy version` — Show current local version.
- `bonesremote version` — Show remote version.
- `bonesremote doctor` — Validate remote environment.
