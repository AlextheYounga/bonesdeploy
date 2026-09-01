use std::cell::RefCell;
use std::fs;
use std::fs::{File, OpenOptions, TryLockError};
use std::path::PathBuf;
use std::thread_local;

use anyhow::{Context, Result, bail};

use bonesdeploy_core::paths;

pub mod atomic;
pub mod record;
pub mod releases;
pub mod store;

pub use atomic::atomic_write;
pub use record::{DeploymentPhase, DeploymentRecord, ProcessIdentity};
pub use releases::{
    current_release_dir, current_release_name, list_releases_sorted, point_symlink_atomically, release_dir, shared_dir,
};
pub use store::quarantine_candidates;

thread_local! {
    static SITES_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Overrides the state root for the current thread until the returned scope is dropped.
pub fn override_sites_root(root: PathBuf) -> ScopedSitesRoot {
    let prev = SITES_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root)));
    ScopedSitesRoot(prev)
}

#[must_use = "the scope must be retained for the root override to remain active"]
pub struct ScopedSitesRoot(Option<PathBuf>);

impl Drop for ScopedSitesRoot {
    fn drop(&mut self) {
        let previous = self.0.take();
        SITES_ROOT_OVERRIDE.with(|slot| {
            slot.replace(previous);
        });
    }
}

pub fn resolved_sites_root() -> PathBuf {
    SITES_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_sites_root)
}

pub fn resolved_site_root(site: &str) -> PathBuf {
    resolved_sites_root().join(site)
}

pub fn recovery_dir(site: &str) -> PathBuf {
    resolved_site_root(site).join(paths::RECOVERY_DIR)
}

fn deployment_lock_path(site: &str) -> PathBuf {
    resolved_sites_root().join(format!(".{site}.{}", paths::DEPLOYMENT_LOCK_FILE))
}

pub struct DeploymentLock(File);

impl DeploymentLock {
    pub fn acquire(site: &str) -> Result<Self> {
        let path = deployment_lock_path(site);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create deployment state directory {}", parent.display()))?;
        }

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("Failed to open deployment lock {}", path.display()))?;
        match file.try_lock() {
            Ok(()) => Ok(Self(file)),
            Err(TryLockError::WouldBlock) => {
                bail!("A deployment is already running for {site}. Run 'bonesdeploy site releases' to inspect it.")
            }
            Err(TryLockError::Error(error)) => {
                Err(error).with_context(|| format!("Failed to lock deployment state for {site}"))
            }
        }
    }
}

impl Drop for DeploymentLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

/// Reads the centralized per-site state (migrating previous files on first read).
pub fn read_site_state(site: &str) -> Result<store::SiteState> {
    store::read_state(site)
}

pub fn read_active_deployment(site: &str) -> Result<Option<DeploymentRecord>> {
    Ok(store::read_state(site)?.active().cloned())
}

pub fn write_active_deployment(site: &str, deployment: &DeploymentRecord) -> Result<()> {
    let state = store::read_state(site)?;
    let state = state.with_active(Some(deployment.clone()));
    store::write_state(site, &state).with_context(|| format!("Failed to write active deployment state for {site}"))
}

pub fn clear_active_deployment(site: &str) -> Result<()> {
    let state = store::read_state(site)?;
    let state = state.with_active(None);
    store::write_state(site, &state).with_context(|| format!("Failed to clear active deployment state for {site}"))
}

/// Returns `Ok(name)` with the staged release present in the store, or an error
/// when no staging is recorded (mirrors the historical staged-release behavior).
pub fn read_staged_release(site: &str) -> Result<String> {
    match store::read_state(site)?.staged_release() {
        Some(release) if !release.is_empty() => Ok(release.to_string()),
        _ => bail!("Staged release state is empty: {}", store::state_path(site).display()),
    }
}

/// Returns the staged release name if one is recorded.
pub fn staged_release(site: &str) -> Result<Option<String>> {
    Ok(store::read_state(site)?.staged_release().map(str::to_string))
}

pub fn write_staged_release(site: &str, release: &str) -> Result<()> {
    let state = store::read_state(site)?;
    let state = state.with_staged_release(Some(release.to_string()));
    store::write_state(site, &state).with_context(|| format!("Failed to write staged release state for {site}"))
}

pub fn clear_staged_release(site: &str) -> Result<()> {
    let state = store::read_state(site)?;
    let state = state.with_staged_release(None);
    store::write_state(site, &state).with_context(|| format!("Failed to clear staged release state for {site}"))
}
