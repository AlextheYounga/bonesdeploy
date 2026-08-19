use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bonesdeploy_core::config::{runtime_group_for, runtime_user_for};

use super::{release_directory, staged_release_name, tree};
use crate::release::SiteMutation;
use crate::release::lifecycle::DeploymentSnapshot;

pub(super) fn run(mutation: &SiteMutation, snapshot: &DeploymentSnapshot, context: &Path) -> Result<PathBuf> {
    let release_name = staged_release_name(mutation)?;
    let release_dir = release_directory(mutation, &release_name);
    let runtime_user = runtime_user_for(&snapshot.config.project_name);
    let runtime_group = runtime_group_for(&snapshot.config.project_name);
    tree::prepare_release_tree(context, &release_dir, &runtime_user, &runtime_group)
        .with_context(|| format!("Failed to promote release {release_name}"))?;

    println!("Copied release {release_name} into {}", release_dir.display());
    Ok(release_dir)
}

pub(super) fn finalize(mutation: &SiteMutation, snapshot: &DeploymentSnapshot) -> Result<()> {
    let release_name = staged_release_name(mutation)?;
    let release_dir = release_directory(mutation, &release_name);
    let runtime_group = runtime_group_for(&snapshot.config.project_name);

    tree::seal_release_tree(&release_dir, &runtime_group)
        .with_context(|| format!("Failed to seal release {release_name}"))?;
    println!("Sealed release {release_name}.");
    Ok(())
}
