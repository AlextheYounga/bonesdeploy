use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use tempfile::TempDir;

use bonesdeploy_core::paths;

use crate::config;
use crate::ui::output;

mod patches;
mod release;
mod sync;
mod version;

const SOURCE_REPO_URL: &str = "https://github.com/AlextheYounga/bonesdeploy.git";

#[derive(Clone, Copy)]
pub struct Options {
    pub skip_local: bool,
    pub skip_remote: bool,
}

pub async fn run(options: Options) -> Result<()> {
    println!("{}", style("Checking for updates").cyan().bold());
    let current_local = release::current_local_version();
    let current_remote = release::current_remote_version();

    if options.skip_local && options.skip_remote {
        println!("{} Already up to date.", output::success_marker());
        return Ok(());
    }

    let temp_dir = TempDir::new().context("Failed to create temp directory")?;
    let temp_path = temp_dir.path();

    let release_version = latest_release_version()?;
    let source_dir = clone_release_source(temp_path, &release_version)?;
    let release_versions = read_release_versions(&source_dir, &release_version)?;

    let mut updated = false;

    if !options.skip_local {
        if current_local != release_versions.bonesdeploy {
            println!("{}", style("Updating bonesdeploy").cyan().bold());
            release::update_local_from_crates_io(&release_versions.bonesdeploy)?;
            updated = true;
        }

        if Path::new(paths::local_bones_toml()).exists() {
            let cfg = config::load(Path::new(paths::local_bones_toml()))?;
            patches::run_local(&cfg, &release_versions.bonesdeploy)?;
        }
        sync::refresh_local_bones_from_source(&source_dir, Path::new(paths::local_bones_dir()))?;
    }

    if !options.skip_remote {
        if current_remote != release_versions.bonesremote {
            println!("{}", style("Updating bonesremote").cyan().bold());
            updated = true;
        }
        release::update_remote_from_release(&current_remote, &release_versions.bonesremote).await?;
    }

    if updated {
        println!("{} Update complete.", output::success_marker());
    } else {
        println!("{} Already up to date.", output::success_marker());
    }

    Ok(())
}

fn latest_release_version() -> Result<String> {
    let output = Command::new("curl")
        .args([
            "--fail",
            "--silent",
            "--show-error",
            "--location",
            "--proto",
            "=https",
            "--tlsv1.2",
            "https://api.github.com/repos/AlextheYounga/bonesdeploy/releases/latest",
        ])
        .output()
        .context("Failed to query the latest BonesDeploy GitHub release")?;
    if !output.status.success() {
        bail!("Failed to query the latest BonesDeploy GitHub release");
    }

    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).context("GitHub returned an invalid release response")?;
    parse_release_tag(value.get("tag_name").and_then(serde_json::Value::as_str))
}

fn parse_release_tag(tag: Option<&str>) -> Result<String> {
    let tag = tag.ok_or_else(|| anyhow::anyhow!("GitHub release response has no tag_name"))?;
    let Some(version) = tag.strip_prefix('v') else {
        bail!("Latest GitHub release tag must start with 'v', got '{tag}'");
    };
    if version.is_empty()
        || !version.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'-' | b'+'))
    {
        bail!("Latest GitHub release tag has an invalid version: '{tag}'");
    }
    Ok(version.to_string())
}

fn clone_release_source(temp_path: &Path, version: &str) -> Result<PathBuf> {
    let source_dir = temp_path.join("source");
    let tag = format!("v{version}");

    let clone_status = Command::new("git")
        .args(["clone", "--depth", "1", "--branch", &tag, SOURCE_REPO_URL])
        .arg(&source_dir)
        .status()
        .context("Failed to clone bonesdeploy repository")?;

    if !clone_status.success() {
        bail!("Failed to clone {SOURCE_REPO_URL} release tag {tag}");
    }

    Ok(source_dir)
}

struct ReleaseVersions {
    bonesdeploy: String,
    bonesremote: String,
}

fn read_release_versions(source_dir: &Path, release_version: &str) -> Result<ReleaseVersions> {
    let bonesdeploy = version::read_package_version(&source_dir.join("crates/bonesdeploy/Cargo.toml"))?;
    let bonesremote = version::read_package_version(&source_dir.join("crates/bonesremote/Cargo.toml"))?;
    if bonesdeploy != release_version || bonesremote != release_version {
        bail!(
            "Release tag v{release_version} must match bonesdeploy ({bonesdeploy}) and bonesremote ({bonesremote}) package versions"
        );
    }

    Ok(ReleaseVersions { bonesdeploy, bonesremote })
}

#[cfg(test)]
mod tests {
    use super::parse_release_tag;
    use anyhow::Result;

    #[test]
    fn release_tag_accepts_semver_versions() -> Result<()> {
        assert_eq!(parse_release_tag(Some("v0.7.3"))?, "0.7.3");
        assert_eq!(parse_release_tag(Some("v0.7.3-rc.1+build"))?, "0.7.3-rc.1+build");
        Ok(())
    }

    #[test]
    fn release_tag_rejects_unexpected_values() {
        assert!(parse_release_tag(Some("0.7.3")).is_err());
        assert!(parse_release_tag(Some("v0.7.3/tag")).is_err());
        assert!(parse_release_tag(None).is_err());
    }
}
