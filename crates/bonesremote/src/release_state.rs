use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config::BonesConfig;

pub fn staged_release_path(cfg: &BonesConfig) -> PathBuf {
    Path::new(&cfg.data.git_dir).join("bones").join(".staged_release")
}

pub fn read_staged_release(cfg: &BonesConfig) -> Result<String> {
    let path = staged_release_path(cfg);
    let value = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read staged release state at {}", path.display()))?;
    let release = value.trim().to_string();

    if release.is_empty() {
        bail!("Staged release state file is empty: {}", path.display());
    }

    Ok(release)
}

pub fn write_staged_release(cfg: &BonesConfig, release: &str) -> Result<()> {
    let path = staged_release_path(cfg);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create staged release state dir: {}", parent.display()))?;
    }

    fs::write(&path, format!("{release}\n"))
        .with_context(|| format!("Failed to write staged release state: {}", path.display()))
}

pub fn clear_staged_release(cfg: &BonesConfig) -> Result<()> {
    let path = staged_release_path(cfg);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Failed to remove staged release state: {}", path.display()))?;
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

pub fn current_release_dir(cfg: &BonesConfig) -> Result<PathBuf> {
    let current_link = current_link(cfg);
    let active_target =
        fs::read_link(&current_link).with_context(|| format!("Failed to read {}", current_link.display()))?;

    Ok(if active_target.is_absolute() {
        active_target
    } else {
        current_link.parent().unwrap_or_else(|| Path::new("/")).join(active_target)
    })
}

pub fn current_release_name(cfg: &BonesConfig) -> Result<String> {
    let current_release = current_release_dir(cfg)?;
    current_release
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve current release name from {}", current_release.display()))
}

pub fn list_releases_sorted(cfg: &BonesConfig) -> Result<Vec<String>> {
    let releases_dir = releases_dir(cfg);
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&releases_dir)
        .with_context(|| format!("Failed to read releases dir: {}", releases_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
    }

    names.sort();
    Ok(names)
}

pub fn point_symlink_atomically(link_path: &Path, target_path: &Path) -> Result<()> {
    let Some(parent) = link_path.parent() else {
        bail!("Invalid symlink path: {}", link_path.display());
    };

    fs::create_dir_all(parent).with_context(|| format!("Failed to create symlink parent: {}", parent.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    let temp_name = format!(".tmp_current_{}_{}", process::id(), nanos);
    let temp_link = parent.join(temp_name);

    if fs::symlink_metadata(&temp_link).is_ok() {
        fs::remove_file(&temp_link)
            .with_context(|| format!("Failed to cleanup stale temp link: {}", temp_link.display()))?;
    }

    symlink(target_path, &temp_link).with_context(|| {
        format!("Failed to create temporary symlink {} -> {}", temp_link.display(), target_path.display())
    })?;

    fs::rename(&temp_link, link_path).with_context(|| {
        format!("Failed to atomically switch symlink {} -> {}", link_path.display(), target_path.display())
    })
}
