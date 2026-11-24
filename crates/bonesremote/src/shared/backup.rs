use std::fs;
use std::num::NonZeroU16;
use std::path::Path;

use anyhow::{Context, Result, bail};
use borgbackup::common::{CommonOptions, CreateOptions, PruneOptions, PruneWithin, PruneWithinTime};
use borgbackup::sync;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::release::SiteMutation;

/// One shared-data backup and its age-based retention policy.
/// Callers must authorize the operation before running it.
pub struct BackupJob<'a> {
    pub repository: &'a Path,
    pub shared: &'a Path,
    pub passphrase_file: &'a Path,
    pub keep_days: u16,
}

impl BackupJob<'_> {
    /// Creates the archive, then prunes archives older than the retention window.
    pub fn run(&self, mutation: &SiteMutation, now: OffsetDateTime) -> Result<()> {
        let passphrase = read_passphrase_file(self.passphrase_file)?;
        let repository = self.repository.display().to_string();
        let archive = archive_name(mutation.site(), now)?;
        println!("Creating backup archive {archive}");
        let create = create_options(&repository, &archive, &self.shared.display().to_string(), &passphrase);
        sync::create(&create, &CommonOptions::default()).context("Failed to create the Borg archive")?;
        prune(&repository, &passphrase, self.keep_days)?;
        println!("Pruned archives older than {} days", self.keep_days);
        Ok(())
    }
}

/// Builds the archive name `<site>_<YYYYMMDD_HHMMSS>` in UTC, matching the
/// release naming convention.
fn archive_name(site: &str, now: OffsetDateTime) -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]_[hour][minute][second]");
    let timestamp = now.format(&TIMESTAMP_FORMAT).context("Failed to format the backup timestamp")?;
    Ok(format!("{site}_{timestamp}"))
}

fn create_options(repository: &str, archive: &str, shared: &str, passphrase: &str) -> CreateOptions {
    let mut options =
        CreateOptions::new(repository.to_string(), archive.to_string(), vec![shared.to_string()], Vec::new());
    options.passphrase = Some(passphrase.to_string());
    options
}

fn prune(repository: &str, passphrase: &str, keep_days: u16) -> Result<()> {
    sync::prune(&prune_options(repository, passphrase, keep_days)?, &CommonOptions::default())
        .context("Failed to apply backup retention")
}

fn prune_options(repository: &str, passphrase: &str, keep_days: u16) -> Result<PruneOptions> {
    let mut options = PruneOptions::new(repository.to_string());
    options.passphrase = Some(passphrase.to_string());
    options.keep_within = Some(PruneWithin {
        quantifier: NonZeroU16::new(keep_days).with_context(|| format!("Invalid retention: {keep_days} days"))?,
        time: PruneWithinTime::Day,
    });
    Ok(options)
}

fn read_passphrase_file(path: &Path) -> Result<String> {
    let passphrase = fs::read_to_string(path)
        .with_context(|| {
            format!("Missing Borg passphrase at {}; is the site provisioned for backups?", path.display())
        })?
        .trim()
        .to_string();
    if passphrase.is_empty() {
        bail!("Borg passphrase file {} is empty", path.display());
    }
    Ok(passphrase)
}
