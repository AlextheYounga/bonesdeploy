use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use bonesdeploy_core::paths;

use crate::privileges;
use crate::release::state as release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release activate")?;

    let cfg = super::load_site_config(site)?;
    let release_name = release_state::read_staged_release(site)?;
    let release_dir = release_state::release_dir(&cfg.project_root, &release_name);
    let current_link = PathBuf::from(&cfg.project_root).join(paths::CURRENT_LINK);

    if !release_dir.exists() {
        anyhow::bail!("Promoted release directory does not exist: {}", release_dir.display());
    }

    if current_link.exists() && !current_link.is_symlink() {
        bail!("current exists and is not a symlink: {}", current_link.display());
    }

    release_state::point_symlink_atomically(&current_link, Path::new(&release_dir))?;

    println!("Activated release: {release_name}");
    Ok(())
}
