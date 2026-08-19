use std::path::PathBuf;

use anyhow::Result;
use bonesdeploy_core::config::Bones;

use crate::release::lifecycle;
use crate::release::state::{self, DeploymentLock, DeploymentRecord};

/// A site-scoped mutation guard: the deployment lock plus the validated site
/// configuration.
///
/// Every operation that changes per-site state takes this object instead of
/// locking or loading config on its own, so the deployment serialization lock
/// and the confused-deputy check (`project_name == site`) are always applied
/// together and on the same configuration snapshot.
///
/// Acquiring a `SiteMutation` does **not** require the site to be idle:
/// cancellation and drop-failed operate on sites that are deliberately
/// non-idle. Callers that need to run against an idle site (deploy, rollback,
/// site import) call `ensure_site_idle` themselves after acquiring.
pub struct SiteMutation {
    site: String,
    config: Bones,
    _lock: DeploymentLock,
}

impl SiteMutation {
    /// Acquires the serialization lock and loads the validated site config.
    ///
    /// Fails if another deployment holds the lock or if the site config is
    /// unreadable or does not belong to `site`.
    pub(crate) fn acquire(site: &str) -> Result<Self> {
        let _lock = DeploymentLock::acquire(site)?;
        let config = lifecycle::load_site_config(site)?;
        Ok(Self::new(site, config, _lock))
    }

    /// Acquires the serialization lock and adopts an already-validated
    /// configuration.
    ///
    /// Builds the guard from an already-loaded config and a held lock.
    ///
    /// Used by cancellation, which must stop a live deployment process before
    /// its serialization lock becomes available: the config is loaded and the
    /// site identity verified *before* terminating, then the lock is taken and
    /// the guard assembled for all subsequent file mutations.
    pub(crate) fn adopt(site: &str, config: Bones, lock: DeploymentLock) -> Self {
        Self::new(site, config, lock)
    }

    fn new(site: &str, config: Bones, lock: DeploymentLock) -> Self {
        Self { site: site.to_string(), config, _lock: lock }
    }

    pub(crate) fn site(&self) -> &str {
        &self.site
    }

    pub(crate) fn config(&self) -> &Bones {
        &self.config
    }

    pub(crate) fn active(&self) -> Result<Option<DeploymentRecord>> {
        Ok(state::read_site_state(self.site())?.active().cloned())
    }

    pub(crate) fn state(&self) -> Result<state::store::SiteState> {
        state::read_site_state(self.site())
    }

    pub(crate) fn set_active(&self, deployment: &DeploymentRecord) -> Result<()> {
        state::write_active_deployment(self.site(), deployment)
    }

    pub(crate) fn clear_active(&self) -> Result<()> {
        state::clear_active_deployment(self.site())
    }

    pub(crate) fn staged_release(&self) -> Result<Option<String>> {
        state::staged_release(self.site())
    }

    pub(crate) fn required_staged_release(&self) -> Result<String> {
        state::read_staged_release(self.site())
    }

    pub(crate) fn release_dir(&self, release: &str) -> PathBuf {
        state::release_dir(&self.config.project_root, release)
    }

    pub(crate) fn shared_dir(&self) -> PathBuf {
        state::shared_dir(&self.config.project_root)
    }

    pub(crate) fn current_release_name(&self) -> Result<String> {
        state::current_release_name(&self.config.project_root)
    }

    pub(crate) fn clear_staged_release(&self) -> Result<()> {
        state::clear_staged_release(self.site())
    }

    pub(crate) fn set_staged_release(&self, release: &str) -> Result<()> {
        state::write_staged_release(self.site(), release)
    }
}
