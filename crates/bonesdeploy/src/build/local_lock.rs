use std::fs::{self, File, OpenOptions, TryLockError};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::validate_site_name;
use bonesdeploy_core::paths;

const LOCK_FILE: &str = "local-build.lock";

/// Serializes local builds that share one site's Docker and cache state.
pub struct LocalBuildLock(File);

impl LocalBuildLock {
    /// Acquires the site's local-build lock without waiting.
    ///
    /// # Errors
    /// Returns an error when the site name is invalid, the lock file cannot be
    /// opened, or another local build already owns the lock.
    pub fn acquire(site: &str) -> Result<Self> {
        validate_site_name(site)?;

        let path = lock_path(site);
        let parent = path.parent().context("Local build lock path has no parent directory")?;
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create local build cache directory {}", parent.display()))?;

        let file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .truncate(false)
            .open(&path)
            .with_context(|| format!("Failed to open local build lock for {site} at {}", path.display()))?;
        match file.try_lock() {
            Ok(()) => Ok(Self(file)),
            Err(TryLockError::WouldBlock) => bail!("A local build is already running for {site}"),
            Err(TryLockError::Error(error)) => {
                Err(error).with_context(|| format!("Failed to lock local build state for {site} at {}", path.display()))
            }
        }
    }
}

impl Drop for LocalBuildLock {
    fn drop(&mut self) {
        let _ = self.0.unlock();
    }
}

fn lock_path(site: &str) -> PathBuf {
    paths::bones_cache_root().join("build").join(site).join(LOCK_FILE)
}
