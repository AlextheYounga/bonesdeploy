use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_pending_release(&cfg)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);

    if !release_dir.exists() {
        anyhow::bail!(
            "Pending release directory does not exist: {}",
            release_dir.display()
        );
    }

    let current_link = release_state::current_link(&cfg);
    release_state::point_symlink_atomically(&current_link, &release_dir)?;

    let pruned = prune_old_releases(&cfg, &release_name)?;
    release_state::clear_pending_release(&cfg)?;

    println!("Activated release: {release_name}");
    if !pruned.is_empty() {
        println!("Pruned releases: {}", pruned.join(", "));
    }

    Ok(())
}

fn prune_old_releases(cfg: &config::BonesConfig, active_release: &str) -> Result<Vec<String>> {
    let mut releases = list_releases_sorted(cfg)?;
    let keep = cfg.releases.keep.max(1);

    let mut pruned = Vec::new();
    while releases.len() > keep {
        let oldest = releases.remove(0);
        if oldest == active_release {
            releases.push(oldest);
            releases.sort();
            continue;
        }

        let path = release_state::release_dir(cfg, &oldest);
        if path.exists() {
            fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to prune old release {}", path.display()))?;
            pruned.push(oldest);
        }
    }

    Ok(pruned)
}

fn list_releases_sorted(cfg: &config::BonesConfig) -> Result<Vec<String>> {
    let releases_dir = release_state::releases_dir(cfg);
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
