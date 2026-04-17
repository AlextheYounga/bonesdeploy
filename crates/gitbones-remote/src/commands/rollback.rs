use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::permissions;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let worktree = &cfg.data.worktree;
    let worktree_path = Path::new(worktree);

    let current_link = worktree_path.join("current");

    if !current_link.is_symlink() {
        bail!("No current release symlink found at {}", current_link.display());
    }

    // Read current symlink target to get active release name
    let current_target = fs::read_link(&current_link)
        .with_context(|| format!("Failed to read symlink {}", current_link.display()))?;
    let current_name = current_target
        .file_name()
        .with_context(|| "Could not get release name from current symlink")?
        .to_string_lossy()
        .to_string();

    // List releases, sort, find the one before current
    let releases_dir = worktree_path.join("releases");
    let mut entries: Vec<String> = fs::read_dir(&releases_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    entries.sort();

    let current_idx = entries
        .iter()
        .position(|e| e == &current_name)
        .with_context(|| format!("Current release {current_name} not found in releases/"))?;

    if current_idx == 0 {
        bail!("No previous release to roll back to. Current release: {current_name}");
    }

    let previous_name = &entries[current_idx - 1];

    // Atomic symlink swap to previous release
    let current_tmp = worktree_path.join("current.tmp");
    let link_target = Path::new("releases").join(previous_name);

    if current_tmp.exists() || current_tmp.is_symlink() {
        fs::remove_file(&current_tmp)?;
    }

    unix_fs::symlink(&link_target, &current_tmp)
        .with_context(|| "Failed to create temp symlink")?;

    fs::rename(&current_tmp, &current_link)
        .with_context(|| "Failed to atomically swap current symlink")?;

    println!("Rolled back: {current_name} → {previous_name}");

    // Harden permissions on the now-active release
    permissions::harden_release(&cfg)?;

    Ok(())
}
