use std::io::{Read, stdin};
use std::path::PathBuf;

use anyhow::{Context, Result};
use bonesdeploy_core::config::{self, RemoteDeploymentConfig};

use crate::commands::ensure_site_idle;
use crate::git;
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle;
use crate::release::lifecycle::build::ensure_build_user_ready;

use super::coordinator::DeploymentLifecycleCoordinator;

pub fn run_full(site: &str, revision: Option<&str>, config_stdin: bool) -> Result<()> {
    privileges::ensure_root("bonesremote deploy")?;

    if !config_stdin {
        anyhow::bail!("bonesremote deploy requires --config-stdin");
    }
    let bones = read_config_descriptor(site)?;

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

/// Reads the deployment config descriptor from stdin and applies it to
/// site-derived identity and paths.
fn read_config_descriptor(site: &str) -> Result<config::Bones> {
    let mut input = String::new();
    stdin().read_to_string(&mut input).context("Failed to read deployment config from stdin")?;

    let descriptor: RemoteDeploymentConfig =
        serde_json::from_str(&input).context("Failed to parse deployment config descriptor from stdin")?;

    if descriptor.branch.is_empty() {
        anyhow::bail!("Deployment config descriptor has an empty branch");
    }
    config::validate_runtime(&descriptor.runtime)?;

    Ok(descriptor.into_site_config(site))
}
