use std::path::Path;

use anyhow::Result;

use crate::config;
use crate::privileges;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_not_root("bonesremote release activate")?;

    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);

    if !release_dir.exists() {
        anyhow::bail!("Staged release directory does not exist: {}", release_dir.display());
    }

    let current_link = release_state::current_link(&cfg);
    release_state::point_symlink_atomically(&current_link, &release_dir)?;

    release_state::clear_staged_release(&cfg)?;

    println!("Activated release: {release_name}");

    Ok(())
}
