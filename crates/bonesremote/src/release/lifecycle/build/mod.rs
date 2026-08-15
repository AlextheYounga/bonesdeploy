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

pub fn run(snapshot: &super::DeploymentSnapshot, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;
    run_scripts::run(snapshot, context)
}

pub fn promote(snapshot: &super::DeploymentSnapshot, context: &Path) -> Result<PathBuf> {
    privileges::ensure_root("bonesremote release promote")?;
    promote::run(snapshot, context)
}

pub fn finalize(snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release finalize")?;
    promote::finalize(snapshot)
}

pub(super) fn staged_release_name(site: &str) -> Result<String> {
    release_state::read_staged_release(site)
}

pub(super) fn release_directory(project_root: &Path, release_name: &str) -> PathBuf {
    release_state::release_dir(&project_root.to_string_lossy(), release_name)
}
