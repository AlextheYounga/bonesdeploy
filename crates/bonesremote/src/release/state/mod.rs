use std::cell::RefCell;
use std::fs::{File, OpenOptions, TryLockError};
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
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
    static STATE_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
    static LOCK_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Overrides both state and lock roots for the current thread until dropped.
///
/// Tests use one temporary root for both domains; production always uses their
/// separate root-owned directories.
pub fn override_sites_root(root: PathBuf) -> ScopedSitesRoot {
    let previous_state = STATE_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root.clone())));
    let previous_lock = LOCK_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root)));
    ScopedSitesRoot { previous_state, previous_lock }
}

#[must_use = "the scope must be retained for the root override to remain active"]
pub struct ScopedSitesRoot {
    previous_state: Option<PathBuf>,
    previous_lock: Option<PathBuf>,
}

impl Drop for ScopedSitesRoot {
    fn drop(&mut self) {
        STATE_ROOT_OVERRIDE.with(|slot| {
            slot.replace(self.previous_state.take());
        });
        LOCK_ROOT_OVERRIDE.with(|slot| {
            slot.replace(self.previous_lock.take());
        });
    }
}

pub fn resolved_sites_root() -> PathBuf {
    STATE_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_state_root)
}

pub fn resolved_lock_root() -> PathBuf {
    LOCK_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_lock_root)
}

pub fn resolved_site_root(site: &str) -> PathBuf {
    resolved_sites_root().join(site)
}

pub fn recovery_dir(site: &str) -> PathBuf {
    resolved_site_root(site).join(paths::RECOVERY_DIR)
}

pub fn deployment_lock_path(site: &str) -> PathBuf {
    resolved_lock_root().join(format!(".{site}.{}", paths::DEPLOYMENT_LOCK_FILE))
}

pub struct DeploymentLock(File);

impl DeploymentLock {
    pub fn acquire(site: &str) -> Result<Self> {
        let path = deployment_lock_path(site);
        let file = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW).open(&path).with_context(|| {
            format!("Failed to open deployment lock {}. Run 'bonesdeploy site setup' to provision it.", path.display())
        })?;
        let metadata =
            file.metadata().with_context(|| format!("Failed to inspect deployment lock {}", path.display()))?;
        if !metadata.file_type().is_file() {
            bail!("Deployment lock {} is not a regular file", path.display());
        }
        if metadata.permissions().mode() & 0o022 != 0 {
            bail!("Deployment lock {} must not be group or world writable", path.display());
        }
        if unsafe { libc::geteuid() } == 0 && metadata.uid() != 0 {
            bail!("Deployment lock {} must be owned by root", path.display());
        }
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
