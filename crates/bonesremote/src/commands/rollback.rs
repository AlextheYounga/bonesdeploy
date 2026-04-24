use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::privileges;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_not_root("bonesremote release rollback")?;

    let cfg = config::load(Path::new(config_path))?;
    let releases = release_state::list_releases_sorted(&cfg)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = release_state::current_release_name(&cfg)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in runtime/"))?;

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
