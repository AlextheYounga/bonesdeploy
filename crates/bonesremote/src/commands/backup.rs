//! Scheduled shared-data backups.

use anyhow::Result;
use bonesdeploy_core::paths;
use time::OffsetDateTime;

use crate::privileges;
use crate::release::SiteMutation;
use crate::shared::backup::BackupJob;

/// Runs one scheduled backup for the site as root: archive `shared/`, then prune.
pub fn run(site: &str, keep_days: u16) -> Result<()> {
    privileges::ensure_root("bonesremote backup run")?;
    let mutation = SiteMutation::acquire(site)?;
    run_locked(&mutation, keep_days)
}

pub fn run_locked(mutation: &SiteMutation, keep_days: u16) -> Result<()> {
    let site = mutation.site();
    BackupJob {
        repository: &paths::site_backup_repository_path(site),
        shared: &mutation.shared_dir(),
        passphrase_file: &paths::bonesremote_site_passphrase_path(site),
        keep_days,
    }
    .run(mutation, OffsetDateTime::now_utc())
}
