use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use tempfile::TempDir;

use crate::commands::update_release;

const SOURCE_REPO_URL: &str = "https://github.com/AlextheYounga/bonesdeploy.git";
const SOURCE_BRANCH: &str = "master";

#[derive(Clone, Copy)]
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

    if options.skip_local && options.skip_remote {
        println!("{} Nothing to update.", style("Done!").green());
        return Ok(());
    }

    println!("Source branch: {}", style(SOURCE_BRANCH).cyan());

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    println!("Checking master version from {}...", style(SOURCE_REPO_URL).cyan());
    let source_dir = clone_master_source(temp_path)?;
    let master_versions = read_master_versions(&source_dir)?;
    println!("Master bonesdeploy version: {}", style(&master_versions.bonesdeploy).cyan());
    println!("Master bonesremote version: {}", style(&master_versions.bonesremote).cyan());

    if !options.skip_local {
        if current_local == master_versions.bonesdeploy {
            println!("{} Local bonesdeploy is already current.", style("Done!").green());
        } else {
            println!("{}", style("Updating local bonesdeploy...").cyan());
            update_release::update_local_from_source(SOURCE_REPO_URL)?;
            println!("{} Local update complete.", style("Done!").green());
        }
    }

    if !options.skip_remote {
        if current_remote == master_versions.bonesremote {
            println!("{} Remote bonesremote is already current.", style("Done!").green());
        } else {
            println!("{}", style("Updating remote bonesremote...").cyan());
            update_release::update_remote_from_source(SOURCE_REPO_URL, &master_versions.bonesremote).await?;
            println!("{} Remote update complete.", style("Done!").green());
        }
    }

    println!("\n{} All updates complete.", style("Done!").green());

    Ok(())
}

fn clone_master_source(temp_path: &Path) -> Result<PathBuf> {
    let source_dir = temp_path.join("source");

    let clone_status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", SOURCE_BRANCH, SOURCE_REPO_URL])
        .arg(&source_dir)
        .status()
        .context("Failed to clone bonesdeploy repository")?;

    if !clone_status.success() {
        bail!("Failed to clone {SOURCE_REPO_URL} branch {SOURCE_BRANCH}");
    }

    Ok(source_dir)
}

struct MasterVersions {
    bonesdeploy: String,
    bonesremote: String,
}

fn read_master_versions(source_dir: &Path) -> Result<MasterVersions> {
    let bonesdeploy = read_package_version(&source_dir.join("crates/bonesdeploy/Cargo.toml"))?;
    let bonesremote = read_package_version(&source_dir.join("crates/bonesremote/Cargo.toml"))?;

    Ok(MasterVersions { bonesdeploy, bonesremote })
}

fn read_package_version(manifest: &Path) -> Result<String> {
    let content = fs::read_to_string(manifest).with_context(|| format!("Failed to read {}", manifest.display()))?;
    parse_package_version(&content)
        .with_context(|| format!("Failed to parse package version from {}", manifest.display()))
}

fn parse_package_version(manifest: &str) -> Result<String> {
    let mut in_package_section = false;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            in_package_section = true;
            continue;
        }
        if in_package_section && trimmed.starts_with('[') {
            break;
        }
        if in_package_section && let Some(version) = parse_version_line(trimmed) {
            return Ok(version);
        }
    }

    bail!("missing [package] version")
}

fn parse_version_line(line: &str) -> Option<String> {
    let value = line.strip_prefix("version")?.trim_start().strip_prefix('=')?.trim();
    let version = value.strip_prefix('"')?.strip_suffix('"')?;
    (!version.is_empty()).then(|| version.to_string())
}

#[cfg(test)]
mod tests {
    use super::{SOURCE_BRANCH, SOURCE_REPO_URL, parse_package_version};

    /// Verifies the update source repository and branch constants are set to the canonical values.
    #[test]
    fn update_uses_master_branch_source_repository() {
        assert_eq!(SOURCE_REPO_URL, "https://github.com/AlextheYounga/bonesdeploy.git");
        assert_eq!(SOURCE_BRANCH, "master");
    }

    /// Extracts the package version from the `[package]` section of a Cargo manifest.
    #[test]
    fn parses_package_version_from_manifest_package_section() -> anyhow::Result<()> {
        let manifest = r#"
[package]
name = "bonesdeploy"
version = "0.2.8"
edition = "2024"

[dependencies]
version = "not-this"
"#;

        assert_eq!(parse_package_version(manifest)?, "0.2.8");
        Ok(())
    }

    /// Returns an error when the manifest has no `[package]` section with a version field.
    #[test]
    fn rejects_manifest_without_package_version() {
        let result = parse_package_version("[dependencies]\nversion = \"0.2.8\"\n");
        assert!(result.is_err());
    }
}
