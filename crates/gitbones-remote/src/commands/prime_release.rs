use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use crate::config;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_pending_release(&cfg)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    let shared_dir = release_state::shared_dir(&cfg);

    for shared_path in &cfg.releases.shared_paths {
        prime_path(&release_dir, &shared_dir, shared_path)?;
    }

    println!("Primed release: {release_name}");
    Ok(())
}

fn prime_path(release_dir: &Path, shared_dir: &Path, relative_path: &str) -> Result<()> {
    let release_path = release_dir.join(relative_path);
    let shared_path = shared_dir.join(relative_path);

    if path_exists(&release_path) && !path_exists(&shared_path) {
        ensure_parent_exists(&shared_path)?;
        fs::rename(&release_path, &shared_path).with_context(|| {
            format!(
                "Failed to move release path into shared path: {} -> {}",
                release_path.display(),
                shared_path.display()
            )
        })?;
    }

    if !path_exists(&shared_path) {
        create_default_shared_target(&shared_path, relative_path)?;
    }

    ensure_parent_exists(&release_path)?;
    if path_exists(&release_path) {
        remove_path(&release_path)?;
    }

    symlink(&shared_path, &release_path).with_context(|| {
        format!("Failed to create shared symlink {} -> {}", release_path.display(), shared_path.display())
    })?;

    println!("Linked shared path: {} -> {}", release_path.display(), shared_path.display());

    Ok(())
}

fn create_default_shared_target(shared_path: &Path, relative_path: &str) -> Result<()> {
    ensure_parent_exists(shared_path)?;

    if looks_like_file(relative_path) {
        fs::File::create(shared_path)
            .with_context(|| format!("Failed to create shared file: {}", shared_path.display()))?;
    } else {
        fs::create_dir_all(shared_path)
            .with_context(|| format!("Failed to create shared directory: {}", shared_path.display()))?;
    }

    Ok(())
}

fn looks_like_file(relative_path: &str) -> bool {
    PathBuf::from(relative_path).file_name().is_some_and(|name| {
        let name = name.to_string_lossy();
        name.starts_with('.') || name.contains('.')
    })
}

fn ensure_parent_exists(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent directory: {}", parent.display()))?;
    }
    Ok(())
}

fn remove_path(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Failed to inspect path for removal: {}", path.display()))?;

    if metadata.file_type().is_symlink() || metadata.is_file() {
        fs::remove_file(path).with_context(|| format!("Failed to remove path: {}", path.display()))?;
    } else if metadata.is_dir() {
        fs::remove_dir_all(path).with_context(|| format!("Failed to remove directory: {}", path.display()))?;
    }

    Ok(())
}

fn path_exists(path: &Path) -> bool {
    fs::symlink_metadata(path).is_ok()
}
