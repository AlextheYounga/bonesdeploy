use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state as release_state;
use crate::runtime::docker;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;
    let mutation = SiteMutation::acquire(site)?;
    run_locked(&mutation)
}

pub fn run_locked(mutation: &SiteMutation) -> Result<()> {
    let staged = mutation.staged_release()?;
    let Some(release_name) = staged.filter(|name| !name.is_empty()) else {
        println!("No staged release state found. Nothing to clean.");
        return Ok(());
    };

    let release_dir = mutation.release_dir(&release_name);
    ensure_release_not_active(mutation, &release_name)?;
    if release_dir.exists() {
        let retained = release_state::list_releases_sorted(&mutation.config().project_root)?
            .into_iter()
            .filter(|release| release != &release_name)
            .map(|release| mutation.release_dir(&release))
            .collect::<Vec<_>>();
        docker::command::remove_unreferenced_release_images(&release_dir, &retained)?;
        fs::remove_dir_all(&release_dir)
            .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
        println!("Removed failed release: {release_name}");
    }

    mutation.clear_staged_release()?;
    println!("Cleared staged release state.");
    Ok(())
}

pub fn remove_unreferenced_manifest_images(
    mutation: &SiteMutation,
    tags: &[String],
    excluded_release: Option<&str>,
) -> Result<()> {
    let retained = release_state::list_releases_sorted(&mutation.config().project_root)?
        .into_iter()
        .filter(|release| Some(release.as_str()) != excluded_release)
        .map(|release| mutation.release_dir(&release))
        .collect::<Vec<PathBuf>>();
    docker::command::remove_unreferenced_image_tags(tags, &retained)
}

pub fn ensure_release_not_active(mutation: &SiteMutation, release: &str) -> Result<()> {
    let current = mutation.current_release_name().context("Failed to determine the active release before cleanup")?;
    if current == release {
        bail!("Refusing to remove active release {release}");
    }
    Ok(())
}
