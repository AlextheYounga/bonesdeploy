use std::env;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::commands::remote_setup::resolve_bootstrap_ssh_user;
use crate::config;
use crate::update_assets;

pub fn current_local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

pub fn current_remote_version() -> String {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    if !bones_yaml.exists() {
        return String::from("unknown");
    }

    let Ok(cfg) = config::load(bones_yaml) else {
        return String::from("unknown");
    };

    let host = format!("{}@{}", cfg.permissions.defaults.deploy_user, cfg.data.host);
    let output = Command::new("ssh").args(["-p", &cfg.data.port]).args([&host, "bonesremote", "version"]).output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().strip_prefix("bonesremote ").unwrap_or("unknown").to_string()
        }
        _ => String::from("unknown"),
    }
}

pub fn target_triple() -> String {
    env::consts::ARCH.to_string() + "-" + env::consts::OS
}

pub fn update_local_binary(temp_path: &Path, version: &str) -> Result<()> {
    let target = target_triple();
    let binary_name = format!("bonesdeploy-{target}-{version}");

    let source_binary = temp_path.join(&binary_name);
    if !source_binary.exists() {
        bail!("Local binary not found in release: {binary_name}");
    }

    let install_root = Path::new("/opt/bonesdeploy");
    let versions_dir = install_root.join("versions");
    let target_version_dir = versions_dir.join(version);
    let current_dir = install_root.join("current");

    fs::create_dir_all(&target_version_dir)
        .with_context(|| format!("Failed to create {}", target_version_dir.display()))?;

    let dest_binary = target_version_dir.join("bonesdeploy");
    fs::copy(&source_binary, &dest_binary)
        .with_context(|| format!("Failed to copy binary to {}", dest_binary.display()))?;

    fs::set_permissions(&dest_binary, fs::Permissions::from_mode(0o755))
        .with_context(|| format!("Failed to set permissions on {}", dest_binary.display()))?;

    verify_binary(&dest_binary)?;

    // Use atomic rename to prevent broken symlinks during update
    let temp_link = current_dir.join(".bonesdeploy_swap");
    if temp_link.exists() {
        fs::remove_file(&temp_link)?;
    }
    symlink_file(&target_version_dir, &temp_link)?;

    fs::rename(&temp_link, current_dir.join("bonesdeploy")).context("Failed to atomically switch local symlink")?;

    let global_link = Path::new("/usr/local/bin/bonesdeploy");
    if global_link.exists() {
        fs::remove_file(global_link)?;
    }
    symlink_file(&current_dir.join("bonesdeploy"), global_link)?;

    println!("Local version: {}", current_local_version());

    Ok(())
}

pub fn update_remote_binary(temp_path: &Path, version: &str) -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    if !bones_yaml.exists() {
        bail!("No .bones/bones.yaml found. Run from a bonesdeploy project directory.");
    }

    let cfg = config::load(bones_yaml)?;

    let target = target_triple();
    let binary_name = format!("bonesremote-{target}-{version}");
    let source_binary = temp_path.join(&binary_name);

    if !source_binary.exists() {
        bail!("Remote binary not found in release: {binary_name}");
    }

    verify_binary(&source_binary)?;

    let ansible_temp = tempfile::TempDir::new().context("Failed to create Ansible temp directory")?;
    let playbook_path = update_assets::materialize_playbook(ansible_temp.path())?;

    let remote_staging = format!("/tmp/bonesremote-{version}");

    println!("Uploading bonesremote to remote host...");
    let host = format!("{}@{}", cfg.permissions.defaults.deploy_user, cfg.data.host);
    let status = Command::new("scp")
        .args(["-P", &cfg.data.port])
        .arg(&source_binary)
        .arg(format!("{host}:{remote_staging}"))
        .status()
        .context("Failed to upload bonesremote via scp")?;

    if !status.success() {
        bail!("Failed to upload bonesremote binary");
    }

    println!("Running remote update playbook...");
    run_update_playbook(&cfg, &playbook_path, &remote_staging, version)?;

    Ok(())
}

pub fn run_update_playbook(
    cfg: &config::BonesConfig,
    playbook: &Path,
    staging_path: &str,
    version: &str,
) -> Result<()> {
    let roles_dir = playbook
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("roles"))
        .ok_or_else(|| anyhow::anyhow!("Invalid playbook path structure"))?;

    let inventory = format!("{},", cfg.data.host);
    let ssh_user = resolve_bootstrap_ssh_user();

    let ansible_playbook = resolve_ansible_playbook()?;

    let mut command = Command::new(&ansible_playbook);
    command
        .env("ANSIBLE_ROLES_PATH", roles_dir.display().to_string())
        .arg("-i")
        .arg(&inventory)
        .arg("-u")
        .arg(&ssh_user)
        .arg("-e")
        .arg(format!("ansible_port={}", cfg.data.port))
        .arg("-e")
        .arg(format!("bonesremote_staging_path={staging_path}"))
        .arg("-e")
        .arg(format!("bonesremote_target_version={version}"))
        .arg(playbook);

    println!("Running: {command:?}");

    let status = command.status().context("Failed to run ansible-playbook")?;

    if !status.success() {
        bail!("Remote update playbook failed with status {status}");
    }

    Ok(())
}

fn resolve_ansible_playbook() -> Result<PathBuf> {
    if ansible_playbook_available(Path::new("ansible-playbook")) {
        return Ok(PathBuf::from("ansible-playbook"));
    }

    let home = env::var("HOME").context("HOME is not set")?;
    let local_ansible = Path::new(&home).join(".local/bin/ansible-playbook");

    if ansible_playbook_available(&local_ansible) {
        return Ok(local_ansible);
    }

    bail!("ansible-playbook not found. Install Ansible first.");
}

fn ansible_playbook_available(binary: &Path) -> bool {
    Command::new(binary).arg("--version").status().is_ok_and(|s| s.success())
}

fn verify_binary(path: &Path) -> Result<()> {
    let output = Command::new(path)
        .arg("version")
        .output()
        .with_context(|| format!("Failed to run {} version", path.display()))?;

    if !output.status.success() {
        bail!("Binary verification failed: {}", path.display());
    }

    Ok(())
}

fn symlink_file(target: &Path, link: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        symlink(target, link)
            .with_context(|| format!("Failed to create symlink {} -> {}", link.display(), target.display()))?;
    }
    #[cfg(not(unix))]
    {
        bail!("Symlinks are only supported on Unix systems");
    }

    Ok(())
}
