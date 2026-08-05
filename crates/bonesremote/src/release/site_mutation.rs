use anyhow::Result;
use bonesdeploy_core::config::Bones;

use crate::release::lifecycle;
use crate::release::state::DeploymentLock;

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
        Ok(Self { site: site.to_string(), config, _lock })
    }

    /// Builds the guard from an already-loaded config and a held lock.
    ///
    /// Used by cancellation, which must stop a live deployment process before
    /// its serialization lock becomes available: the config is loaded and the
    /// site identity verified *before* terminating, then the lock is taken and
    /// the guard assembled for all subsequent file mutations.
    pub(crate) fn adopt(site: &str, config: Bones, lock: DeploymentLock) -> Self {
        Self { site: site.to_string(), config, _lock: lock }
    }

    pub(crate) fn site(&self) -> &str {
        &self.site
    }

    pub(crate) fn config(&self) -> &Bones {
        &self.config
    }
}
