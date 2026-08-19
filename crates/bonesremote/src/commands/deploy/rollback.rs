use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::commands::{ensure_site_idle, service};
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state as release_state;

pub fn rollback(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release rollback")?;
    // Rollback mutates live site state, so it must be serialized with deploy,
    // cancellation, pruning, and config receive like every other mutation. It
    // also runs against an idle site so it never repoints `current` under an
    // in-flight or interrupted deployment.
    let mutation = SiteMutation::acquire(site)?;
    ensure_site_idle(&mutation)?;

    let cfg = mutation.config();

    let releases = release_state::list_releases_sorted(&cfg.project_root)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = release_state::current_release_name(&cfg.project_root)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;

    if current_idx == 0 {
        bail!("Current release is already the oldest release; cannot roll back");
    }

    let previous_name = releases[current_idx - 1].clone();
    switch_and_verify(&cfg.project_root, &current_name, &previous_name, || service::run(&mutation))?;

    println!("Rollback complete: {current_name} -> {previous_name}");
    Ok(())
}

/// Repoints `current` to `previous_name`, restarts (via `restart`) and verifies
/// the result, then restores `current_name` and restarts it again if
/// verification fails. This keeps rollback reversible when service restart
/// reports failure.
pub fn switch_and_verify(
    project_root: &str,
    current_name: &str,
    previous_name: &str,
    restart: impl Fn() -> Result<()>,
) -> Result<()> {
    let current_link = PathBuf::from(project_root).join(paths::CURRENT_LINK);
    let previous_dir = release_state::release_dir(project_root, previous_name);
    let current_dir = release_state::release_dir(project_root, current_name);

    release_state::point_symlink_atomically(&current_link, &previous_dir)?;

    if let Err(error) = restart() {
        let mut error = error;
        if let Err(restore_error) = release_state::point_symlink_atomically(&current_link, &current_dir) {
            error =
                error.context(format!("Failed to restore the previous release '{current_name}': {restore_error:#}"));
        } else if let Err(restart_error) = restart() {
            error = error.context(format!("Failed to restart the restored release: {restart_error:#}"));
        }
        return Err(error.context(format!(
            "Service restart failed after rolling back to '{previous_name}'; restored '{current_name}'"
        )));
    }

    Ok(())
}
