use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::paths;

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state::point_symlink_atomically;

pub fn run(mutation: &SiteMutation, snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release activate")?;

    let release_name = mutation.required_staged_release()?;
    let release_dir = mutation.release_dir(&release_name);
    let current_link = snapshot.project_root.join(paths::CURRENT_LINK);

    if !release_dir.exists() {
        anyhow::bail!("Promoted release directory does not exist: {}", release_dir.display());
    }

    if current_link.exists() && !current_link.is_symlink() {
        bail!("current exists and is not a symlink: {}", current_link.display());
    }

    point_symlink_atomically(&current_link, Path::new(&release_dir))?;

    println!("Activated release: {release_name}");
    Ok(())
}
