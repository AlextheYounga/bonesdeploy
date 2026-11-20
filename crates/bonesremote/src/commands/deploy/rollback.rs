use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle::activate::{ActivationFailure, ReleaseActivation};
use crate::release::state as release_state;
use crate::runtime::service;

pub fn rollback(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release rollback")?;
    // Rollback mutates live site state, so it must be serialized with deploy,
    // cancellation, pruning, and config receive like every other mutation. It
    // also runs against an idle site so it never repoints `current` under an
    // in-flight or interrupted deployment.
    let mutation = SiteMutation::acquire(site)?;
    mutation.ensure_idle()?;

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
    switch_and_verify(&cfg.project_root, &current_name, &previous_name, || service::run_for_release(&mutation))?;

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
    restart: impl FnMut() -> Result<()>,
) -> Result<()> {
    let previous_dir = release_state::release_dir(project_root, previous_name);
    let current_dir = release_state::release_dir(project_root, current_name);
    match ReleaseActivation::new(Path::new(project_root), &previous_dir, &current_dir).run(|_| Ok(()), restart) {
        Ok(()) => Ok(()),
        Err(ActivationFailure::Aborted(error) | ActivationFailure::RecoveryRequired(error)) => {
            Err(error.context(format!("Rollback to '{previous_name}' failed")))
        }
    }
}
