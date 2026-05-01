use std::path::Path;

use anyhow::{Result, bail};

use crate::config;
use crate::privileges;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_not_root("bonesremote release activate")?;

    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    let live_root = Path::new(&cfg.data.live_root);

    if !release_dir.exists() {
        anyhow::bail!("Staged release directory does not exist: {}", release_dir.display());
    }

    if live_root.exists() && !live_root.is_symlink() {
        bail!("live_root exists and is not a symlink: {}", live_root.display());
    }

    let current_link = release_state::current_link(&cfg);

    // Enforce runtime entrypoint wiring just-in-time during activation.
    release_state::point_symlink_atomically(live_root, &current_link)?;
    release_state::point_symlink_atomically(&current_link, &release_dir)?;

    release_state::clear_staged_release(&cfg)?;

    println!("Activated release: {release_name}");

    Ok(())
}
