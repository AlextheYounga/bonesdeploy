use std::fs;
use std::path::Path;
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

pub fn run(options: UpdateOptions) -> Result<()> {
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

    println!("Building binaries from {}...", style(SOURCE_REPO_URL).cyan());
    let target_version = build_master_binaries(temp_path)?;
    println!("Built master version: {}", style(&target_version).cyan());

    if !options.skip_local {
        println!("{}", style("Updating local bonesdeploy...").cyan());
        update_release::update_local_binary(temp_path, &target_version)?;
        println!("{} Local update complete.", style("Done!").green());
    }

    if !options.skip_remote {
        println!("{}", style("Updating remote bonesremote...").cyan());
        update_release::update_remote_binary(temp_path, &target_version)?;
        println!("{} Remote update complete.", style("Done!").green());
    }

    println!("\n{} All updates complete.", style("Done!").green());

    Ok(())
}

fn build_master_binaries(temp_path: &Path) -> Result<String> {
    let target = update_release::target_triple();
    let source_dir = temp_path.join("source");

    let clone_status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", SOURCE_BRANCH, SOURCE_REPO_URL])
        .arg(&source_dir)
        .status()
        .context("Failed to clone bonesdeploy repository")?;

    if !clone_status.success() {
        bail!("Failed to clone {SOURCE_REPO_URL} branch {SOURCE_BRANCH}");
    }

    let build_status = Command::new("cargo")
        .args(["build", "--release", "-p", "bonesdeploy", "-p", "bonesremote"])
        .current_dir(&source_dir)
        .status()
        .context("Failed to build bonesdeploy binaries")?;

    if !build_status.success() {
        bail!("Failed to build bonesdeploy binaries from {SOURCE_BRANCH}");
    }

    let built_version = read_built_version(&source_dir.join("target").join("release").join("bonesdeploy"))?;

    copy_built_binary(
        &source_dir,
        temp_path,
        "bonesdeploy",
        &binary_asset_name("bonesdeploy", &target, &built_version),
    )?;
    copy_built_binary(
        &source_dir,
        temp_path,
        "bonesremote",
        &binary_asset_name("bonesremote", &target, &built_version),
    )?;

    Ok(built_version)
}

fn read_built_version(binary: &Path) -> Result<String> {
    let output = Command::new(binary)
        .arg("version")
        .output()
        .with_context(|| format!("Failed to run {} version", binary.display()))?;

    if !output.status.success() {
        bail!("Failed to read built bonesdeploy version from {}", binary.display());
    }

    parse_bonesdeploy_version(&String::from_utf8_lossy(&output.stdout))
}

fn parse_bonesdeploy_version(output: &str) -> Result<String> {
    output
        .trim()
        .strip_prefix("bonesdeploy ")
        .map(ToOwned::to_owned)
        .filter(|version| !version.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Unexpected bonesdeploy version output: {output:?}"))
}

fn copy_built_binary(source_dir: &Path, temp_path: &Path, binary: &str, asset_name: &str) -> Result<()> {
    let source = source_dir.join("target").join("release").join(binary);
    let destination = temp_path.join(asset_name);

    fs::copy(&source, &destination)
        .with_context(|| format!("Failed to copy built binary {} to {}", source.display(), destination.display()))?;

    Ok(())
}

fn binary_asset_name(binary: &str, target: &str, version: &str) -> String {
    format!("{binary}-{target}-{version}")
}

#[cfg(test)]
mod tests {
    use super::{SOURCE_BRANCH, SOURCE_REPO_URL, binary_asset_name, parse_bonesdeploy_version};

    #[test]
    fn update_uses_master_branch_source_repository() {
        assert_eq!(SOURCE_REPO_URL, "https://github.com/AlextheYounga/bonesdeploy.git");
        assert_eq!(SOURCE_BRANCH, "master");
    }

    #[test]
    fn built_binary_asset_names_match_existing_install_layout() {
        assert_eq!(binary_asset_name("bonesdeploy", "x86_64-linux", "0.2.7"), "bonesdeploy-x86_64-linux-0.2.7");
        assert_eq!(binary_asset_name("bonesremote", "x86_64-linux", "0.2.7"), "bonesremote-x86_64-linux-0.2.7");
    }

    #[test]
    fn parses_built_master_version_from_bonesdeploy_binary_output() -> anyhow::Result<()> {
        assert_eq!(parse_bonesdeploy_version("bonesdeploy 0.2.8\n")?, "0.2.8");
        Ok(())
    }

    #[test]
    fn rejects_unexpected_bonesdeploy_binary_version_output() {
        let result = parse_bonesdeploy_version("0.2.8\n");
        assert!(result.is_err());
    }
}
