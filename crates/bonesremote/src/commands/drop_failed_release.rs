use std::fs;

use anyhow::{Context, Result, bail};
use shared::config;
use shared::paths;

use crate::privileges;
use crate::release::state as release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;

    let staged_path = release_state::staged_release_path(site);
    if !staged_path.exists() {
        println!("No staged release state found. Nothing to clean.");
        return Ok(());
    }

    let release_name = match release_state::read_staged_release(site) {
        Ok(name) => name,
        Err(error) => {
            release_state::clear_staged_release(site)
                .with_context(|| format!("Failed to clear invalid staged release state for {site}"))?;
            return Err(error).context("Staged release state was invalid and has been cleared");
        }
    };

    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let release_dir = release_state::release_dir(&cfg.project_root, &release_name);
    if release_dir.exists() {
        fs::remove_dir_all(&release_dir)
            .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
        println!("Removed failed release: {release_name}");
    }

    release_state::clear_staged_release(site)?;
    println!("Cleared staged release state.");
    Ok(())
}
