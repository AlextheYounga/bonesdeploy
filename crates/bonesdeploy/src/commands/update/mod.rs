use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use tempfile::TempDir;

use shared::paths;

use crate::ui::output;

mod release;
mod sync;
mod version;

const SOURCE_REPO_URL: &str = "https://github.com/AlextheYounga/bonesdeploy.git";
const SOURCE_BRANCH: &str = "master";

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

    let source_dir = clone_master_source(temp_path)?;
    let master_versions = read_master_versions(&source_dir)?;

    let mut updated = false;

    if !options.skip_local {
        if current_local != master_versions.bonesdeploy {
            println!("{}", style("Updating bonesdeploy").cyan().bold());
            release::update_local_from_source(SOURCE_REPO_URL)?;
            updated = true;
        }

        sync::refresh_local_bones_from_source(&source_dir, Path::new(paths::LOCAL_BONES_DIR))?;
    }

    if !options.skip_remote && current_remote != master_versions.bonesremote {
        println!("{}", style("Updating bonesremote").cyan().bold());
        release::update_remote_from_source(SOURCE_REPO_URL, &master_versions.bonesremote).await?;
        updated = true;
    }

    if updated {
        println!("{} Update complete.", output::success_marker());
    } else {
        println!("{} Already up to date.", output::success_marker());
    }

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
    let bonesdeploy = version::read_package_version(&source_dir.join("crates/bonesdeploy/Cargo.toml"))?;
    let bonesremote = version::read_package_version(&source_dir.join("crates/bonesremote/Cargo.toml"))?;

    Ok(MasterVersions { bonesdeploy, bonesremote })
}
