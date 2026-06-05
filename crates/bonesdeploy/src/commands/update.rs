use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use serde::Deserialize;
use sha2::Digest;
use tempfile::TempDir;

use crate::commands::update_release;

const GITHUB_API_RELEASES_URL: &str = "https://api.github.com/repos/anomalyco/bonesdeploy/releases/latest";
const GITHUB_RELEASES_URL: &str = "https://github.com/anomalyco/bonesdeploy/releases/download";

pub struct UpdateOptions {
    pub skip_local: bool,
    pub skip_remote: bool,
}

pub async fn run(options: UpdateOptions) -> Result<()> {
    println!("{}", style("bonesdeploy update").bold());

    let current_local = update_release::current_local_version();
    let current_remote = update_release::current_remote_version();

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
        update_release::update_local_binary(temp_path, &target_version)?;
        println!("{} Local update complete.", style("Done!").green());
    }

    if remote_needs_update {
        println!("{}", style("Updating remote bonesremote...").cyan());
        update_release::update_remote_binary(temp_path, &target_version)?;
        println!("{} Remote update complete.", style("Done!").green());
    }

    println!("\n{} All updates complete.", style("Done!").green());

    Ok(())
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
    let target = update_release::target_triple();
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

fn verify_downloads(temp_path: &Path) -> Result<()> {
    let target = update_release::target_triple();

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
