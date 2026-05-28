use std::env;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use serde::Deserialize;
use sha2::Digest;
use tempfile::TempDir;

use crate::commands::remote_setup::resolve_bootstrap_ssh_user;
use crate::config;
use crate::update_assets;

const GITHUB_API_RELEASES_URL: &str = "https://api.github.com/repos/anomalyco/bonesdeploy/releases/latest";
const GITHUB_RELEASES_URL: &str = "https://github.com/anomalyco/bonesdeploy/releases/download";

const LOCAL_INSTALL_ROOT: &str = "/opt/bonesdeploy";
const LOCAL_BIN_LINK: &str = "/usr/local/bin/bonesdeploy";

pub struct UpdateOptions {
    pub skip_local: bool,
    pub skip_remote: bool,
}

pub async fn run(options: UpdateOptions) -> Result<()> {
    println!("{}", style("bonesdeploy update").bold());

    let current_local = get_current_local_version();
    let current_remote = get_current_remote_version();

    println!("Current local version: {}", style(&current_local).cyan());
    println!("Current remote version: {}", style(&current_remote).cyan());

    let release = fetch_latest_release().await?;
    let target_version = release.tag_name.trim_start_matches('v').to_string();

    println!("Latest release: {}", style(&target_version).cyan());

    let local_needs_update = !options.skip_local && current_local != target_version;
    let remote_needs_update = !options.skip_remote && current_remote != target_version;

    if !local_needs_update && !remote_needs_update {
        println!("{}", style("Already up to date.").green());
        return Ok(());
    }

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    println!("Downloading release assets...");
    download_release_assets(&release, temp_path).await?;

    println!("Verifying downloads...");
    verify_downloads(temp_path)?;

    if local_needs_update {
        println!("{}", style("Updating local bonesdeploy...").cyan());
        update_local_binary(temp_path, &target_version)?;
        println!("{} Local update complete.", style("Done!").green());
    }

    if remote_needs_update {
        println!("{}", style("Updating remote bonesremote...").cyan());
        update_remote_binary(temp_path, &target_version)?;
        println!("{} Remote update complete.", style("Done!").green());
    }

    println!("\n{} All updates complete.", style("Done!").green());

    Ok(())
}

fn get_current_local_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn get_current_remote_version() -> String {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    if !bones_yaml.exists() {
        return "unknown".to_string();
    }

    let Ok(cfg) = config::load(bones_yaml) else {
        return "unknown".to_string();
    };

    let host = format!("{}@{}", cfg.permissions.defaults.deploy_user, cfg.data.host);
    let output = Command::new("ssh").args(["-p", &cfg.data.port]).args([&host, "bonesremote", "version"]).output();

    match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().strip_prefix("bonesremote ").unwrap_or("unknown").to_string()
        }
        _ => "unknown".to_string(),
    }
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
}

async fn fetch_latest_release() -> Result<GitHubRelease> {
    let client = reqwest::Client::new();
    let response = client
        .get(GITHUB_API_RELEASES_URL)
        .header("Accept", "application/vnd.github.v3+json")
        .header("User-Agent", "bonesdeploy-update")
        .send()
        .await
        .context("Failed to fetch release info from GitHub")?;

    if !response.status().is_success() {
        bail!("GitHub API returned status {}", response.status());
    }

    let release: GitHubRelease = response.json().await.context("Failed to parse GitHub release response")?;

    Ok(release)
}

async fn download_release_assets(release: &GitHubRelease, temp_path: &Path) -> Result<()> {
    let target = get_target_triple();
    let version = release.tag_name.trim_start_matches('v');

    let bonesdeploy_asset_name = format!("bonesdeploy-{target}-{version}.tar.gz");
    let bonesremote_asset_name = format!("bonesremote-{target}-{version}");
    let checksums_name = format!("checksums-{version}.txt");

    let bonesdeploy_url = format!("{GITHUB_RELEASES_URL}/v{version}/{bonesdeploy_asset_name}");
    let bonesremote_url = format!("{GITHUB_RELEASES_URL}/v{version}/{bonesremote_asset_name}");
    let checksums_url = format!("{GITHUB_RELEASES_URL}/v{version}/{checksums_name}");

    download_file(&bonesdeploy_url, &temp_path.join(&bonesdeploy_asset_name)).await?;
    download_file(&bonesremote_url, &temp_path.join(&bonesremote_asset_name)).await?;
    download_file(&checksums_url, &temp_path.join(checksums_name)).await?;

    let archive_path = temp_path.join(&bonesdeploy_asset_name);
    extract_tarball(&archive_path, temp_path)?;

    Ok(())
}

async fn download_file(url: &str, dest: &Path) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "bonesdeploy-update")
        .send()
        .await
        .with_context(|| format!("Failed to download {url}"))?;

    if !response.status().is_success() {
        bail!("Failed to download {}: status {}", url, response.status());
    }

    let bytes = response.bytes().await.with_context(|| format!("Failed to read response from {url}"))?;

    fs::write(dest, &bytes).with_context(|| format!("Failed to write {}", dest.display()))?;

    Ok(())
}

fn extract_tarball(archive_path: &Path, dest: &Path) -> Result<()> {
    let status = Command::new("tar")
        .args(["-xzf", &archive_path.display().to_string(), "-C", &dest.display().to_string()])
        .status()
        .context("Failed to extract tarball")?;

    if !status.success() {
        bail!("Failed to extract {}", archive_path.display());
    }

    Ok(())
}

fn get_target_triple() -> String {
    env::consts::ARCH.to_string() + "-" + env::consts::OS
}

fn verify_downloads(temp_path: &Path) -> Result<()> {
    let target = get_target_triple();

    let version_files: Vec<_> = fs::read_dir(temp_path)
        .context("Failed to read temp directory")?
        .filter_map(Result::ok)
        .filter(|entry| {
            let name = entry.file_name().to_string_lossy().to_string();
            name.starts_with("checksums-")
                && Path::new(&name).extension().is_some_and(|ext| ext.eq_ignore_ascii_case("txt"))
        })
        .collect();

    if version_files.is_empty() {
        bail!("No checksums file found");
    }

    let checksums_path = &version_files[0].path();
    let checksums_content = fs::read_to_string(checksums_path).context("Failed to read checksums file")?;

    for line in checksums_content.lines() {
        let parts: Vec<_> = line.splitn(2, "  ").collect();
        if parts.len() != 2 {
            continue;
        }

        let expected_hash = parts[0];
        let filename = parts[1];

        let file_path = temp_path.join(filename);
        if !file_path.exists() {
            if filename.contains("bonesdeploy") && filename.ends_with(".tar.gz") {
                continue;
            }
            if filename.contains(&target) {
                continue;
            }
            bail!("Missing file: {filename}");
        }

        let file_bytes = fs::read(&file_path).with_context(|| format!("Failed to read {filename}"))?;

        let actual_hash = format!("{:x}", sha2::Sha256::digest(&file_bytes));

        if actual_hash != expected_hash {
            bail!("Checksum mismatch for {filename}");
        }
    }

    Ok(())
}

fn update_local_binary(temp_path: &Path, version: &str) -> Result<()> {
    let target = get_target_triple();
    let binary_name = format!("bonesdeploy-{target}-{version}");

    let source_binary = temp_path.join(&binary_name);
    if !source_binary.exists() {
        let extracted_name = format!("bonesdeploy-{target}-{version}");
        let possible_path = temp_path.join(&extracted_name);
        if possible_path.exists() {
            return update_local_binary(temp_path, version);
        }
        bail!("Local binary not found in release: {binary_name}");
    }

    let install_root = Path::new(LOCAL_INSTALL_ROOT);
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

    let temp_link = current_dir.join(".bonesdeploy_swap");
    if temp_link.exists() {
        fs::remove_file(&temp_link)?;
    }
    symlink_file(&target_version_dir, &temp_link)?;

    fs::rename(&temp_link, current_dir.join("bonesdeploy")).context("Failed to atomically switch local symlink")?;

    let global_link = Path::new(LOCAL_BIN_LINK);
    if global_link.exists() {
        fs::remove_file(global_link)?;
    }
    symlink_file(&current_dir.join("bonesdeploy"), global_link)?;

    println!("Local version: {}", style(get_current_local_version()).cyan());

    Ok(())
}

fn update_remote_binary(temp_path: &Path, version: &str) -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    if !bones_yaml.exists() {
        bail!("No .bones/bones.yaml found. Run from a bonesdeploy project directory.");
    }

    let cfg = config::load(bones_yaml)?;

    let target = get_target_triple();
    let binary_name = format!("bonesremote-{target}-{version}");
    let source_binary = temp_path.join(&binary_name);

    if !source_binary.exists() {
        bail!("Remote binary not found in release: {binary_name}");
    }

    verify_binary(&source_binary)?;

    let ansible_temp = TempDir::new().context("Failed to create Ansible temp directory")?;
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

fn run_update_playbook(cfg: &config::BonesConfig, playbook: &Path, staging_path: &str, version: &str) -> Result<()> {
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
