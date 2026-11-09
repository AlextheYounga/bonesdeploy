use std::path::Path;

use anyhow::{Error, Result, bail};
use bonesdeploy_core::paths;

use crate::release::state::{DeploymentPhase, point_symlink_atomically};

fn point_current(project_root: &Path, release_dir: &Path) -> Result<()> {
    let current_link = project_root.join(paths::CURRENT_LINK);

    if !release_dir.is_dir() {
        anyhow::bail!("Promoted release directory does not exist: {}", release_dir.display());
    }

    if current_link.exists() && !current_link.is_symlink() {
        bail!("current exists and is not a symlink: {}", current_link.display());
    }

    point_symlink_atomically(&current_link, release_dir)
}

/// A failed cutover is disposable only after the original release is restored
/// and its services have been verified. Otherwise callers must retain evidence.
pub enum ActivationFailure {
    Aborted(Error),
    RecoveryRequired(Error),
}

/// Switches a release and verifies it, compensating every post-switch failure.
/// Persistence runs inside the same boundary as service verification, including
/// saving the restoration target before the link can change.
pub struct ReleaseActivation<'a> {
    project_root: &'a Path,
    candidate: &'a Path,
    previous: &'a Path,
}

impl<'a> ReleaseActivation<'a> {
    pub fn new(project_root: &'a Path, candidate: &'a Path, previous: &'a Path) -> Self {
        Self { project_root, candidate, previous }
    }

    pub fn run(
        self,
        mut persist_phase: impl FnMut(DeploymentPhase) -> Result<()>,
        mut restart: impl FnMut() -> Result<()>,
    ) -> Result<(), ActivationFailure> {
        persist_phase(DeploymentPhase::Sealed).map_err(ActivationFailure::Aborted)?;
        point_current(self.project_root, self.candidate).map_err(ActivationFailure::Aborted)?;
        let result = persist_phase(DeploymentPhase::Activated)
            .and_then(|()| restart())
            .and_then(|()| persist_phase(DeploymentPhase::Verified));
        let Err(error) = result else { return Ok(()) };

        if let Err(restore_error) = point_current(self.project_root, self.previous) {
            return Err(ActivationFailure::RecoveryRequired(
                error.context(format!("Failed to restore previous release: {restore_error:#}")),
            ));
        }
        if let Err(restart_error) = restart() {
            return Err(ActivationFailure::RecoveryRequired(
                error.context(format!("Failed to restart the restored release: {restart_error:#}")),
            ));
        }
        Err(ActivationFailure::Aborted(error.context("Restored and verified the previous release")))
    }
}
