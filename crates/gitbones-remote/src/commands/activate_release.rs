use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let worktree = &cfg.data.worktree;
    let worktree_path = Path::new(worktree);

    // Derive git_dir from config path
    let config_abs = fs::canonicalize(config_path)
        .with_context(|| format!("Failed to resolve config path: {config_path}"))?;
    let git_dir = config_abs
        .parent()
        .and_then(|p| p.parent())
        .with_context(|| "Could not derive git_dir from config path")?;

    // Read release name from state file
    let state_file = git_dir.join("bones").join(".current_release");
    let release_name = fs::read_to_string(&state_file)
        .with_context(|| format!("Failed to read {}", state_file.display()))?
        .trim()
        .to_string();

    let release_dir = worktree_path.join("releases").join(&release_name);
    let shared_dir = worktree_path.join("shared");

    if !release_dir.exists() {
        bail!("Release directory does not exist: {}", release_dir.display());
    }

    // Process shared paths
    if let Some(ref releases) = cfg.releases {
        for shared_path in &releases.shared_paths {
            let release_entry = release_dir.join(shared_path);
            let shared_entry = shared_dir.join(shared_path);

            // If path exists in release dir but not in shared: move it to shared (seeds from first deploy)
            if release_entry.exists() && !shared_entry.exists() {
                if let Some(parent) = shared_entry.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::rename(&release_entry, &shared_entry).with_context(|| {
                    format!(
                        "Failed to move {} to {}",
                        release_entry.display(),
                        shared_entry.display()
                    )
                })?;
                println!("Moved {shared_path} to shared/");
            }

            // Create in shared if it doesn't exist
            if !shared_entry.exists() {
                if shared_path.ends_with('/') || !shared_path.contains('.') {
                    fs::create_dir_all(&shared_entry)?;
                } else {
                    if let Some(parent) = shared_entry.parent() {
                        fs::create_dir_all(parent)?;
                    }
                    fs::File::create(&shared_entry)?;
                }
                println!("Created shared/{shared_path}");
            }

            // Remove path from release dir if it still exists, then symlink
            if release_entry.exists() || release_entry.is_symlink() {
                if release_entry.is_dir() && !release_entry.is_symlink() {
                    fs::remove_dir_all(&release_entry)?;
                } else {
                    fs::remove_file(&release_entry)?;
                }
            }

            // Create symlink: release/{path} → ../../shared/{path}
            let link_target = Path::new("../../shared").join(shared_path);
            unix_fs::symlink(&link_target, &release_entry).with_context(|| {
                format!(
                    "Failed to symlink {} → {}",
                    release_entry.display(),
                    link_target.display()
                )
            })?;
            println!("Linked {shared_path} → shared/{shared_path}");
        }
    }

    // Atomic symlink swap: create tmp link, then rename
    let current_link = worktree_path.join("current");
    let current_tmp = worktree_path.join("current.tmp");

    // Relative symlink target: releases/{timestamp}
    let link_target = Path::new("releases").join(&release_name);

    // Remove stale tmp if it exists
    if current_tmp.exists() || current_tmp.is_symlink() {
        fs::remove_file(&current_tmp)?;
    }

    unix_fs::symlink(&link_target, &current_tmp)
        .with_context(|| format!("Failed to create temp symlink {}", current_tmp.display()))?;

    fs::rename(&current_tmp, &current_link)
        .with_context(|| "Failed to atomically swap current symlink")?;

    println!("Activated release: {release_name}");

    // Prune old releases
    if let Some(ref releases) = cfg.releases {
        prune_releases(worktree_path, &current_link, releases.keep)?;
    }

    // Clean up state file
    let _ = fs::remove_file(&state_file);

    Ok(())
}

fn prune_releases(worktree: &Path, current_link: &Path, keep: u32) -> Result<()> {
    let releases_dir = worktree.join("releases");

    // Resolve what current actually points to
    let current_target = fs::read_link(current_link)
        .ok()
        .and_then(|p| p.file_name().map(|f| f.to_string_lossy().to_string()));

    let mut entries: Vec<String> = fs::read_dir(&releases_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .collect();

    entries.sort();

    if entries.len() <= keep as usize {
        return Ok(());
    }

    let to_remove = entries.len() - keep as usize;
    for name in entries.iter().take(to_remove) {
        // Never remove what current points to
        if Some(name.as_str()) == current_target.as_deref() {
            continue;
        }
        let path = releases_dir.join(name);
        fs::remove_dir_all(&path)
            .with_context(|| format!("Failed to remove old release {}", path.display()))?;
        println!("Pruned old release: {name}");
    }

    Ok(())
}
