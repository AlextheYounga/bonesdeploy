use std::path::PathBuf;

use anyhow::Result;
use bonesdeploy_core::config;

use crate::commands::ensure_site_idle;
use crate::control_plane;
use crate::git;
use crate::release::SiteMutation;
use crate::release::lifecycle;
use crate::release::lifecycle::build::ensure_build_user_ready;

use super::coordinator::DeploymentLifecycleCoordinator;

pub fn run_full(site: &str, revision: Option<&str>) -> Result<()> {
    let bones = control_plane::load(site)?.into_site_config(site);

    let mutation = SiteMutation::acquire_with_config(site, bones)?;
    ensure_site_idle(&mutation)?;

    let build_user = config::build_user_for(mutation.site());
    let project_root = PathBuf::from(&mutation.config().project_root);
    ensure_build_user_ready(&build_user, &project_root)?;

    let target_revision = revision.map_or_else(|| mutation.config().branch.clone(), ToOwned::to_owned);
    let repo_path = PathBuf::from(&mutation.config().repo_path);
    let revision_commit = git::resolve_revision_commit(&repo_path, &target_revision)?;
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, revision_commit, PathBuf::new());
    DeploymentLifecycleCoordinator::new(&mutation, snapshot).run()
}
