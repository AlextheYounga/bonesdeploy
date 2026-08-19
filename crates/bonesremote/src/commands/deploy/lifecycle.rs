use std::path::PathBuf;

use anyhow::Result;
use bonesdeploy_core::config::build_user_for;

use crate::commands::ensure_site_idle;
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle;
use crate::release::lifecycle::build::ensure_build_user_ready;

use super::coordinator::DeploymentLifecycleCoordinator;

pub fn run_full(site: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_root("bonesremote deploy")?;
    let mutation = SiteMutation::acquire(site)?;
    ensure_site_idle(&mutation)?;

    let build_user = build_user_for(mutation.site());
    let project_root = PathBuf::from(&mutation.config().project_root);
    ensure_build_user_ready(&build_user, &project_root)?;

    let target_revision = revision.map_or_else(|| mutation.config().branch.clone(), ToOwned::to_owned);
    let repo_path = PathBuf::from(&mutation.config().repo_path);
    let revision_commit = lifecycle::checkout::resolve_revision_commit(&repo_path, &target_revision)?;
    let snapshot = lifecycle::DeploymentSnapshot::new(&mutation, revision_commit, PathBuf::new());
    DeploymentLifecycleCoordinator::new(&mutation, snapshot).run()
}
