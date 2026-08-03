use std::path::{Path, PathBuf};

use anyhow::Result;

mod build_user;
mod container;
mod ownership;
mod promote;
mod run_scripts;
mod tree;

pub(crate) use build_user::{ensure_build_user_ready, validate_build_cache};
pub(crate) use container::remove_build_container;

use crate::privileges;
use crate::release::state as release_state;

pub fn run(site: &str, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;
    let cfg = super::load_site_config(site)?;
    run_scripts::run(site, context, &cfg)
}

pub fn promote(site: &str, context: &Path) -> Result<PathBuf> {
    privileges::ensure_root("bonesremote release promote")?;
    let cfg = super::load_site_config(site)?;
    promote::run(site, context, &cfg)
}

pub fn finalize(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release finalize")?;
    let cfg = super::load_site_config(site)?;
    promote::finalize(site, &cfg)
}

pub(super) fn staged_release_name(site: &str) -> Result<String> {
    release_state::read_staged_release(site)
}

pub(super) fn release_directory(project_root: &str, release_name: &str) -> PathBuf {
    release_state::release_dir(project_root, release_name)
}
