use std::fs;

use anyhow::{Context, Result};
use shared::paths;
use shared::registry;

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;

    let staged_path = release_state::staged_release_path(site);
    if !staged_path.exists() {
        println!("No staged release state found. Nothing to clean.");
        return Ok(());
    }

    let Ok(release_name) = release_state::read_staged_release(site) else {
        release_state::clear_staged_release(site).ok();
        println!("Cleared staged release state.");
        return Ok(());
    };

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let release_dir = release_state::release_dir(&cfg, &release_name);
    if release_dir.exists() {
        fs::remove_dir_all(&release_dir)
            .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
        println!("Removed failed release: {release_name}");
    }

    release_state::clear_staged_release(site)?;
    println!("Cleared staged release state.");
    Ok(())
}
