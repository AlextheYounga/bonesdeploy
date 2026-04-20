use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let releases = list_releases_sorted(&cfg)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = resolve_current_release_name(&cfg)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;

    if current_idx == 0 {
        bail!("Current release is already the oldest release; cannot roll back");
    }

    let previous_name = releases[current_idx - 1].clone();
    let previous_dir = release_state::release_dir(&cfg, &previous_name);
    let current_link = release_state::current_link(&cfg);
    release_state::point_symlink_atomically(&current_link, &previous_dir)?;

    println!("Rollback complete: {current_name} -> {previous_name}");
    Ok(())
}

fn resolve_current_release_name(cfg: &config::BonesConfig) -> Result<String> {
    let current_link = release_state::current_link(cfg);
    let target = fs::read_link(&current_link)
        .with_context(|| format!("Failed to read current symlink: {}", current_link.display()))?;

    let absolute_target = if target.is_absolute() {
        target
    } else {
        current_link
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .join(target)
    };

    absolute_target
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Failed to resolve current release name from {}",
                absolute_target.display()
            )
        })
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
