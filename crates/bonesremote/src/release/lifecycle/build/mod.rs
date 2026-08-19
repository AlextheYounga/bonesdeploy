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
use crate::release::SiteMutation;

pub fn run(_mutation: &SiteMutation, snapshot: &super::DeploymentSnapshot, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;
    run_scripts::run(snapshot, context)
}

pub fn promote(mutation: &SiteMutation, snapshot: &super::DeploymentSnapshot, context: &Path) -> Result<PathBuf> {
    privileges::ensure_root("bonesremote release promote")?;
    promote::run(mutation, snapshot, context)
}

pub fn finalize(mutation: &SiteMutation, snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release finalize")?;
    promote::finalize(mutation, snapshot)
}

pub(super) fn staged_release_name(mutation: &SiteMutation) -> Result<String> {
    mutation.required_staged_release()
}

pub(super) fn release_directory(mutation: &SiteMutation, release_name: &str) -> PathBuf {
    mutation.release_dir(release_name)
}
