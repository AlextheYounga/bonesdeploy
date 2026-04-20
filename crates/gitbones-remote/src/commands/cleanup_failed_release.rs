use std::fs;
use std::path::Path;

use anyhow::{Context, Result};

use crate::config;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let pending_path = release_state::pending_release_path(&cfg);

    if !pending_path.exists() {
        println!("No pending release state found. Nothing to clean.");
        return Ok(());
    }

    let release_name = release_state::read_pending_release(&cfg)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);

    if release_dir.exists() {
        fs::remove_dir_all(&release_dir).with_context(|| {
            format!("Failed to remove failed release {}", release_dir.display())
        })?;
        println!("Removed failed release: {release_name}");
    }

    release_state::clear_pending_release(&cfg)?;
    println!("Cleared pending release state.");
    Ok(())
}
