use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config::BonesConfig;

pub fn pending_release_path(cfg: &BonesConfig) -> PathBuf {
    Path::new(&cfg.data.git_dir)
        .join("bones")
        .join(".pending_release")
}

pub fn read_pending_release(cfg: &BonesConfig) -> Result<String> {
    let path = pending_release_path(cfg);
    let value = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read pending release state at {}", path.display()))?;
    let release = value.trim().to_string();

    if release.is_empty() {
        bail!("Pending release state file is empty: {}", path.display());
    }

    Ok(release)
}

pub fn write_pending_release(cfg: &BonesConfig, release: &str) -> Result<()> {
    let path = pending_release_path(cfg);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create pending release state dir: {}",
                parent.display()
            )
        })?;
    }

    fs::write(&path, format!("{release}\n"))
        .with_context(|| format!("Failed to write pending release state: {}", path.display()))
}

pub fn clear_pending_release(cfg: &BonesConfig) -> Result<()> {
    let path = pending_release_path(cfg);
    if path.exists() {
        fs::remove_file(&path).with_context(|| {
            format!("Failed to remove pending release state: {}", path.display())
        })?;
    }
    Ok(())
}

pub fn release_dir(cfg: &BonesConfig, release: &str) -> PathBuf {
    releases_dir(cfg).join(release)
}

pub fn releases_dir(cfg: &BonesConfig) -> PathBuf {
    Path::new(&cfg.data.deploy_root).join("releases")
}

pub fn shared_dir(cfg: &BonesConfig) -> PathBuf {
    Path::new(&cfg.data.deploy_root).join("shared")
}

pub fn current_link(cfg: &BonesConfig) -> PathBuf {
    Path::new(&cfg.data.deploy_root).join("current")
}

pub fn point_symlink_atomically(link_path: &Path, target_path: &Path) -> Result<()> {
    let Some(parent) = link_path.parent() else {
        bail!("Invalid symlink path: {}", link_path.display());
    };

    fs::create_dir_all(parent)
        .with_context(|| format!("Failed to create symlink parent: {}", parent.display()))?;

    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("System clock is before UNIX_EPOCH")?
        .as_nanos();
    let temp_name = format!(".tmp_current_{}_{}", std::process::id(), nanos);
    let temp_link = parent.join(temp_name);

    if fs::symlink_metadata(&temp_link).is_ok() {
        fs::remove_file(&temp_link).with_context(|| {
            format!("Failed to cleanup stale temp link: {}", temp_link.display())
        })?;
    }

    symlink(target_path, &temp_link).with_context(|| {
        format!(
            "Failed to create temporary symlink {} -> {}",
            temp_link.display(),
            target_path.display()
        )
    })?;

    fs::rename(&temp_link, link_path).with_context(|| {
        format!(
            "Failed to atomically switch symlink {} -> {}",
            link_path.display(),
            target_path.display()
        )
    })
}
