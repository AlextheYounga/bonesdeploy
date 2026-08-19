use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use bonesdeploy_core::paths;

pub fn release_dir(project_root: &str, release: &str) -> PathBuf {
    PathBuf::from(project_root).join(paths::RELEASES_DIR).join(release)
}

pub fn releases_dir(project_root: &str) -> PathBuf {
    PathBuf::from(project_root).join(paths::RELEASES_DIR)
}

pub fn shared_dir(project_root: &str) -> PathBuf {
    PathBuf::from(project_root).join(paths::SHARED_DIR)
}

pub fn current_release_dir(project_root: &str) -> Result<PathBuf> {
    let current_link = PathBuf::from(project_root).join(paths::CURRENT_LINK);
    let active_target =
        fs::read_link(&current_link).with_context(|| format!("Failed to read {}", current_link.display()))?;

    if active_target.is_absolute() {
        return Ok(active_target);
    }

    let parent = current_link
        .parent()
        .with_context(|| format!("Current release link has no parent: {}", current_link.display()))?;
    Ok(parent.join(active_target))
}

pub fn current_release_name(project_root: &str) -> Result<String> {
    let current_release = current_release_dir(project_root)?;
    current_release
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve current release name from {}", current_release.display()))
}

pub fn list_releases_sorted(project_root: &str) -> Result<Vec<String>> {
    let releases_dir = releases_dir(project_root);
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&releases_dir)
        .with_context(|| format!("Failed to read releases dir: {}", releases_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != paths::PLACEHOLDER_RELEASE_NAME {
                names.push(name);
            }
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
