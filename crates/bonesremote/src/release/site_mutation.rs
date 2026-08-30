use std::path::PathBuf;

use anyhow::Result;
use bonesdeploy_core::config::{self, Bones};

use crate::release::state::{self, DeploymentLock, DeploymentRecord};

/// A site-scoped mutation guard: the deployment lock plus the site configuration.
///
/// Every operation that changes per-site state takes this object instead of
/// locking or deriving config on its own, so the deployment serialization lock
/// is always applied together with site validation.
///
/// Acquiring a `SiteMutation` does **not** require the site to be idle:
/// cancellation and drop-failed operate on sites that are deliberately
/// non-idle. Callers that need to run against an idle site (deploy, rollback)
/// call `ensure_site_idle` themselves after acquiring.
///
/// Site identity, paths, users, and systemd targets are always derived from
/// `--site` via `Bones::for_site(site)`. Runtime settings (branch, web root,
/// build timeouts, etc.) are supplied by the deploy descriptor for deploy
/// operations and left at defaults for other commands.
pub struct SiteMutation {
    site: String,
    config: Bones,
    _lock: Option<DeploymentLock>,
}

impl SiteMutation {
    /// Acquires the serialization lock using identity and paths derived from
    /// the site name. Runtime settings remain at defaults.
    pub(crate) fn acquire(site: &str) -> Result<Self> {
        config::validate_site_name(site)?;
        let _lock = DeploymentLock::acquire(site)?;
        let config = Bones::for_site(site);
        Ok(Self::new(site, config, Some(_lock)))
    }

    /// Acquires the serialization lock and applies a deployment descriptor
    /// supplied by the local deploy command.
    pub(crate) fn acquire_with_config(site: &str, config: Bones) -> Result<Self> {
        config::validate_site_name(site)?;
        let _lock = DeploymentLock::acquire(site)?;
        Ok(Self::new(site, config, Some(_lock)))
    }

    /// Adopts an already-held lock for cancellation, which must stop a live
    /// deployment process before the lock becomes available.
    pub fn adopt(site: &str, config: Bones, lock: DeploymentLock) -> Self {
        Self::new(site, config, Some(lock))
    }

    /// Builds the site context for a root transition invoked by the
    /// lock-holding coordinator. The transition must not acquire the lock a
    /// second time because advisory locks are process-scoped.
    pub(crate) fn for_transition(site: &str, config: Bones) -> Result<Self> {
        config::validate_site_name(site)?;
        Ok(Self::new(site, config, None))
    }

    fn new(site: &str, config: Bones, lock: Option<DeploymentLock>) -> Self {
        Self { site: site.to_string(), config, _lock: lock }
    }

    pub(crate) fn site(&self) -> &str {
        &self.site
    }

    pub fn config(&self) -> &Bones {
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
