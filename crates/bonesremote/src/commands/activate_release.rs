use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use shared::paths;
use shared::registry;

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release activate")?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg =
        registry::load(&registry_path).map_err(|error| anyhow::anyhow!("Failed to load remote site state: {error}"))?;

    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg, &release_name);
    let current_link = PathBuf::from(&cfg.current_path);

    if !release_dir.exists() {
        anyhow::bail!("Promoted release directory does not exist: {}", release_dir.display());
    }

    if current_link.exists() && !current_link.is_symlink() {
        bail!("current exists and is not a symlink: {}", current_link.display());
    }

    release_state::point_symlink_atomically(&current_link, Path::new(&release_dir))?;
    release_state::clear_staged_release(site)?;

    println!("Activated release: {release_name}");
    Ok(())
}
