use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use tempfile::TempDir;

use bonesdeploy_core::paths;

use crate::config;
use crate::infra::git;
use crate::ui::output;

pub mod release;
pub mod sync;
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
    let current_remote = release::current_remote_version().await;

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

        if Path::new(paths::DOT_ENV).exists() {
            bonesinfra::run(&[
                "patches",
                "apply",
                "--env-file",
                paths::DOT_ENV,
                "--target-version",
                &release_versions.bonesdeploy,
                "--scope",
                "local",
            ])?;
        }
        let template = if Path::new(paths::DOT_ENV).exists() {
            let template = config::load(Path::new(paths::DOT_ENV))?.runtime.template;
            (!template.is_empty()).then_some(template)
        } else {
            None
        };
        sync::refresh_local_infrastructure(&source_dir, Path::new("."), template.as_deref())?;
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

pub fn parse_release_tag(tag: Option<&str>) -> Result<String> {
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

    git::clone_repository(SOURCE_REPO_URL, &tag, &source_dir)?;

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
