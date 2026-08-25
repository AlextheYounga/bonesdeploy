use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::privileges;
use crate::release::SiteMutation;

pub fn run(mutation: &SiteMutation, _snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release wire")?;

    let release_name = mutation.required_staged_release()?;
    let release_dir = mutation.release_dir(&release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let shared_dir = mutation.shared_dir();
    if !shared_dir.is_dir() {
        bail!(
            "Shared root is missing: {}. Run 'bonesdeploy site setup' or site runtime provisioning first.",
            shared_dir.display()
        );
    }

    let shared_env = shared_dir.join(paths::DOT_ENV);
    if !shared_env.is_file() {
        bail!(
            "Shared environment file is missing: {}. Run 'bonesdeploy site setup' or secrets provisioning first.",
            shared_env.display()
        );
    }

    link_relative(&release_dir, paths::DOT_ENV, &shared_env)?;

    Ok(())
}

pub fn link_relative(release_dir: &Path, relative: &str, target: &Path) -> Result<()> {
    let link_path = release_dir.join(relative);
    remove_if_present(&link_path)?;
    symlink(target, &link_path)
        .with_context(|| format!("Failed to link {} -> {}", link_path.display(), target.display()))?;
    Ok(())
}

pub fn remove_if_present(path: &Path) -> Result<()> {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return Ok(());
    };

    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).with_context(|| format!("Failed to remove {}", path.display()))?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory {}", path.display()))?;
    }
    Ok(())
}
