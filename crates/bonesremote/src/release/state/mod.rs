use std::cell::RefCell;
use std::fs;
use std::fs::{File, OpenOptions, TryLockError};
use std::path::PathBuf;
use std::thread_local;

use anyhow::{Context, Result, bail};

use bonesdeploy_core::paths;

mod atomic;
pub(crate) mod record;
pub(crate) mod releases;
pub(crate) mod store;

pub(crate) use atomic::atomic_write;
pub(crate) use record::{DeploymentPhase, DeploymentRecord};
pub(crate) use releases::{
    current_release_dir, current_release_name, list_releases_sorted, point_symlink_atomically, release_dir, shared_dir,
};
pub(crate) use store::quarantine_candidates;

thread_local! {
    static SITES_ROOT_OVERRIDE: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn set_sites_root_for_tests(root: PathBuf) -> ScopedRoot {
    let prev = SITES_ROOT_OVERRIDE.with(|slot| slot.replace(Some(root)));
    ScopedRoot(prev)
}

#[cfg(test)]
pub(crate) struct ScopedRoot(Option<PathBuf>);

#[cfg(test)]
impl Drop for ScopedRoot {
    fn drop(&mut self) {
        let previous = self.0.take();
        SITES_ROOT_OVERRIDE.with(|slot| {
            slot.replace(previous);
        });
    }
}

pub(crate) fn resolved_sites_root() -> PathBuf {
    SITES_ROOT_OVERRIDE.with(|slot| slot.borrow().clone()).unwrap_or_else(paths::bonesremote_sites_root)
}

pub(crate) fn resolved_site_root(site: &str) -> PathBuf {
    resolved_sites_root().join(site)
}

pub(crate) fn recovery_dir(site: &str) -> PathBuf {
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
                bail!("A deployment is already running for {site}. Run 'bonesdeploy releases' to inspect it.")
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::{ScopedRoot, set_sites_root_for_tests, staged_release, store};

    fn temp_root(test_name: &str) -> Result<(ScopedRoot, PathBuf)> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("bonesremote_state_test_{}_{}_{}", process::id(), nanos, test_name));
        fs::create_dir_all(&path)?;
        Ok((set_sites_root_for_tests(path.clone()), path))
    }

    #[test]
    fn write_then_read_staged_release_round_trips() -> Result<()> {
        let (_guard, _root) = temp_root("round_trip")?;

        super::write_staged_release("unitapp", "20260507_151500")?;
        assert_eq!(super::read_staged_release("unitapp")?, "20260507_151500");

        Ok(())
    }

    #[test]
    fn read_staged_release_rejects_missing_state() -> Result<()> {
        let (_guard, root) = temp_root("empty_state")?;

        assert!(super::read_staged_release("emptyapp").is_err());
        let state = store::read_state("emptyapp")?;
        assert!(state.staged_release().is_none());
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn clear_staged_release_removes_the_pointer() -> Result<()> {
        let (_guard, _root) = temp_root("clear_state")?;

        super::write_staged_release("clearapp", "20260507_151501")?;
        assert_eq!(staged_release("clearapp")?.as_deref(), Some("20260507_151501"));
        super::clear_staged_release("clearapp")?;
        assert!(staged_release("clearapp")?.is_none());

        Ok(())
    }
}
