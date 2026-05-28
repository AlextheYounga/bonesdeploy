# bonesdeploy update

## Overview

Updates `bonesdeploy` (local) and `bonesremote` (server) binaries to the latest release from GitHub. The command performs atomic, zero-downtime updates using symlink flipping, ensuring safe production updates with instant rollback capability.

## Command Signature

```bash
bonesdeploy update [--skip-local] [--skip-remote]
```

**Flags:**
- `--skip-local`: Skip updating the local `bonesdeploy` binary
- `--skip-remote`: Skip updating the remote `bonesremote` binary

## Design Principles

### Control-Plane Only

The update command only updates the BonesDeploy control-plane binaries:
- **Local**: `bonesdeploy` CLI on the developer machine
- **Remote**: `bonesremote` binary on the deployment server

Site runtime components (nginx, application code, databases) are never touched by this command. This separation ensures:
- Zero risk to production traffic during updates
- Clear boundary between deployment tooling and deployed applications
- Safe updates even for actively serving sites

### Atomic Symlink Flip

Both local and remote updates use the same atomic pattern:

```
/opt/bonesdeploy/
├── versions/
│   ├── 0.1.0/
│   │   └── bonesdeploy
│   ├── 0.2.0/
│   │   └── bonesdeploy
│   └── 0.3.0/
│       └── bonesdeploy    # New version
├── current/
│   └── bonesdeploy        # -> versions/0.2.0/bonesdeploy (old)
└── .bonesdeploy_swap      # Temp symlink for atomic swap
```

**Atomic Swap Process:**
1. Create temp symlink pointing to new version
2. Atomically rename temp symlink to final name
3. `mv -T` ensures atomic replacement

This pattern guarantees:
- No window where the binary is missing
- Instant rollback via symlink repoint
- No partial states

### Stable Symlink Path

`bonesremote init` resolves the sudoers path via `which bonesremote`, meaning:
- The sudoers entry references whatever path `bonesremote` is at during init
- Moving `bonesremote` without updating sudoers breaks sudo access
- Solution: Maintain a stable symlink path at `/opt/bonesdeploy/current/bonesremote`

The update command preserves this stable path by updating the symlink target, not the symlink location.

## Detailed Execution Steps

### 1. Print Header

**Source:** `update.rs:30`

```rust
println!("{}", style("bonesdeploy update").bold());
```

Displays the command header.

---

### 2. Determine Current Versions

**Source:** `update.rs:32-33`

```rust
let current_local = get_current_local_version();
let current_remote = get_current_remote_version();
```

**Local Version:**
- Read from compile-time constant `CARGO_PKG_VERSION`
- No filesystem lookup needed

**Remote Version:**
- SSH to remote server
- Run `bonesremote version`
- Parse output for version string
- Returns `"unknown"` if unreachable or not installed

---

### 3. Fetch Latest Release

**Source:** `update.rs:38`

```rust
let release = fetch_latest_release().await?;
```

**Implementation:**
- Query GitHub API: `https://api.github.com/repos/anomalyco/bonesdeploy/releases/latest`
- Parse JSON response for `tag_name`
- Strip leading `v` from tag (e.g., `v0.3.0` → `0.3.0`)

**Error Handling:**
- Network failure → clear error message
- Non-200 response → report status code
- Invalid JSON → parse error

---

### 4. Check Update Necessity

**Source:** `update.rs:43-49`

```rust
let local_needs_update = !options.skip_local && current_local != target_version;
let remote_needs_update = !options.skip_remote && current_remote != target_version;

if !local_needs_update && !remote_needs_update {
    println!("{}", style("Already up to date.").green());
    return Ok(());
}
```

Skips download/verification if already at target version.

---

### 5. Create Temp Directory

**Source:** `update.rs:51-52`

```rust
let temp_dir = TempDir::new().context("Failed to create temp directory")?;
let temp_path = temp_dir.path();
```

Creates a temporary directory for download artifacts. Automatically cleaned up on scope exit.

---

### 6. Download Release Assets

**Source:** `update.rs:55`

```rust
download_release_assets(&release, temp_path).await?;
```

**Assets Downloaded:**
- `bonesdeploy-{target}-{version}.tar.gz` — Local binary tarball
- `bonesremote-{target}-{version}` — Remote binary
- `checksums-{version}.txt` — SHA256 checksums

**Target Triple Format:** `{arch}-{os}` (e.g., `aarch64-macos`, `x86_64-linux`)

**Download Process:**
1. Build URL: `https://github.com/anomalyco/bonesdeploy/releases/download/v{version}/{asset_name}`
2. Stream response to temp file
3. Extract tarball if present

---

### 7. Verify Downloads

**Source:** `update.rs:58`

```rust
verify_downloads(temp_path)?;
```

**Verification Steps:**
1. Read `checksums-{version}.txt`
2. For each line `hash  filename`:
   - Read file bytes
   - Compute SHA256 digest
   - Compare hex-encoded hash
3. Bail on mismatch with clear error

**Checksum Format:**
```
e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855  bonesdeploy-aarch64-macos-0.3.0.tar.gz
```

---

### 8. Update Local Binary

**Source:** `update.rs:61-64`, `update.rs:244-291`

```rust
if local_needs_update {
    update_local_binary(temp_path, &target_version)?;
}
```

**Local Update Process:**

1. **Locate Binary in Temp Dir**
   ```rust
   let binary_name = format!("bonesdeploy-{target}-{version}");
   let source_binary = temp_path.join(&binary_name);
   ```

2. **Create Versioned Directory**
   ```rust
   let target_version_dir = versions_dir.join(version);
   fs::create_dir_all(&target_version_dir)?;
   ```

3. **Copy Binary to Versioned Location**
   ```rust
   let dest_binary = target_version_dir.join("bonesdeploy");
   fs::copy(&source_binary, &dest_binary)?;
   fs::set_permissions(&dest_binary, fs::Permissions::from_mode(0o755))?;
   ```

4. **Verify Binary Works**
   ```rust
   verify_binary(&dest_binary)?;
   ```
   Runs `bonesdeploy version` to confirm the binary executes.

5. **Atomic Symlink Flip**
   ```rust
   let temp_link = current_dir.join(".bonesdeploy_swap");
   symlink_file(&target_version_dir, &temp_link)?;
   fs::rename(&temp_link, current_dir.join("bonesdeploy"))?;
   ```

6. **Update Global Bin Link**
   ```rust
   let global_link = Path::new("/usr/local/bin/bonesdeploy");
   symlink_file(&current_dir.join("bonesdeploy"), global_link)?;
   ```

**Result:** `/usr/local/bin/bonesdeploy` → `/opt/bonesdeploy/current/bonesdeploy` → `/opt/bonesdeploy/versions/0.3.0/bonesdeploy`

---

### 9. Update Remote Binary

**Source:** `update.rs:67-70`, `update.rs:293-335`

```rust
if remote_needs_update {
    update_remote_binary(temp_path, &target_version)?;
}
```

**Remote Update Process:**

1. **Load Configuration**
   ```rust
   let cfg = config::load(bones_yaml)?;
   ```
   Requires `.bones/bones.yaml` to exist.

2. **Verify Local Binary**
   ```rust
   verify_binary(&source_binary)?;
   ```
   Verify before upload to catch issues early.

3. **Materialize Ansible Assets**
   ```rust
   let playbook_path = update_assets::materialize_playbook(ansible_temp.path())?;
   ```
   Extract embedded playbooks from the `bonesdeploy` binary.

4. **Upload Binary to Remote**
   ```rust
   let remote_staging = format!("/tmp/bonesremote-{version}");
   Command::new("scp")
       .args(["-P", &cfg.data.port])
       .arg(&source_binary)
       .arg(format!("{host}:{remote_staging}"))
       .status()?;
   ```

5. **Run Ansible Playbook**
   ```rust
   run_update_playbook(&cfg, &playbook_path, &remote_staging, version)?;
   ```

**Ansible Playbook Execution:**

The embedded playbook (`playbooks/update-bonesremote.yml`) performs:

1. **Validate Inputs**
   ```yaml
   - name: Validate required variables
     ansible.builtin.assert:
       that:
         - bonesremote_staging_path is defined
         - bonesremote_target_version is defined
   ```

2. **Create Directory Structure**
   ```yaml
   - name: Ensure target version directory exists
     ansible.builtin.file:
       path: "{{ bonesremote_install_root }}/versions/{{ bonesremote_target_version }}"
       state: directory
   ```

3. **Copy and Verify Binary**
   ```yaml
   - name: Copy staged bonesremote binary
     ansible.builtin.copy:
       src: "{{ bonesremote_staging_path }}"
       dest: ".../bonesremote"
       remote_src: true
   
   - name: Verify staged binary
     ansible.builtin.command:
       argv: [".../bonesremote", "version"]
   ```

4. **Atomic Symlink Flip**
   ```yaml
   - name: Create atomic temp symlink
     ansible.builtin.file:
       src: ".../bonesremote"
       dest: "{{ bonesremote_stable_link }}/.bonesremote_swap_{{ epoch }}"
       state: link
   
   - name: Atomically switch bonesremote symlink
     ansible.builtin.command:
       argv: ["mv", "-T", ".bonesremote_swap_...", "bonesremote"]
   ```

5. **Verify Global Link**
   ```yaml
   - name: Verify global bonesremote works
     ansible.builtin.command:
       argv: ["/usr/local/bin/bonesremote", "version"]
   ```

6. **Run Doctor for Each Project**
   ```yaml
   - name: Run bonesremote doctor for each project
     ansible.builtin.command:
       argv: ["/usr/local/bin/bonesremote", "doctor"]
       chdir: "{{ item.path }}"
     loop: "{{ managed_projects.files }}"
   ```

---

### 10. Report Completion

**Source:** `update.rs:72`

```rust
println!("\n{} All updates complete.", style("Done!").green());
```

## Embedded Update Assets

**Location:** `crates/bonesdeploy/updates/`

The `bonesdeploy` binary embeds all files under `updates/` using `rust-embed`. This includes:

```
updates/
├── playbooks/
│   └── update-bonesremote.yml    # Main update playbook
├── roles/
│   └── bonesremote_update/
│       └── tasks/
│           └── main.yml          # Role entrypoint
└── migrations/
    └── manifest.yml             # Future migration registry
```

**Why Embedded:**
- Self-contained update: no external dependencies
- Works offline for local updates
- Atomic: assets versioned with the binary
- Simple: one artifact to distribute

**Materialization:**
```rust
pub fn materialize_playbook(temp_dir: &Path) -> Result<PathBuf> {
    for file_path in UpdateAssets::iter() {
        let Some(asset) = UpdateAssets::get(&file_path) else { continue; };
        let dest = temp_dir.join(file_path.as_ref());
        write_asset_file(&dest, asset.data.as_ref())?;
    }
    Ok(temp_dir.join("playbooks/update-bonesremote.yml"))
}
```

## Exit Codes

- **0**: All updates successful
- **1**: Update failed (network, verification, permission, etc.)

## When to Run

1. **After BonesDeploy releases**: Check for and install updates
2. **Before major deployments**: Ensure tooling is current
3. **When doctor recommends**: If `bonesdeploy doctor` suggests updating
4. **CI/CD pipelines**: Automated update checks

## Rollback

Both local and remote updates support instant rollback:

**Local Rollback:**
```bash
sudo ln -sf /opt/bonesdeploy/versions/0.2.0/bonesdeploy /opt/bonesdeploy/current/bonesdeploy
```

**Remote Rollback:**
```bash
ssh deploy@server 'sudo ln -sf /opt/bonesdeploy/versions/0.2.0/bonesremote /opt/bonesdeploy/current/bonesremote'
```

The old version directories are preserved until manually cleaned.

## Prerequisites

1. **Local**:
   - Write access to `/opt/bonesdeploy/` (or use `sudo`)
   - Write access to `/usr/local/bin/` (or use `sudo`)

2. **Remote**:
   - SSH access to deployment server
   - `ansible-playbook` installed locally
   - `sudo` configured on remote for `bonesremote` commands

## Related Commands

- `bonesdeploy doctor` — Check if updates are recommended
- `bonesdeploy version` — Show current version
- `bonesremote version` — Show remote version
- `bonesremote doctor` — Validate remote environment
