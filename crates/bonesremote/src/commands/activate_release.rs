use std::path::Path;

use anyhow::{Result, bail};
use shared::paths;

use crate::config;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    let current_link = std::path::PathBuf::from(cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).current);

    if !release_dir.exists() {
        anyhow::bail!("Staged release directory does not exist: {}", release_dir.display());
    }

    if current_link.exists() && !current_link.is_symlink() {
        bail!("current exists and is not a symlink: {}", current_link.display());
    }

    release_state::point_symlink_atomically(&current_link, &release_dir)?;

    release_state::clear_staged_release(&cfg)?;

    println!("Activated release: {release_name}");

    Ok(())
}
