use std::path::{Path, PathBuf};

use anyhow::Result;
use bonesdeploy_core::config::RuntimeBackend;

pub mod build_user;
pub mod container;
pub mod ownership;
pub mod promote;
pub mod run_scripts;
pub mod tree;

pub(crate) use build_user::{
    ensure_build_user_ready, is_build_containment_error, terminate_build_user, validate_build_cache,
};

use crate::privileges;
use crate::release::SiteMutation;
use crate::runtime::docker;

pub fn run(_mutation: &SiteMutation, snapshot: &super::DeploymentSnapshot, context: &Path) -> Result<()> {
    privileges::ensure_root("bonesremote release build")?;
    if snapshot.config.runtime.backend == RuntimeBackend::Docker {
        return docker::command::prepare_candidate(&snapshot.site, &snapshot.project_root, context);
    }
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
