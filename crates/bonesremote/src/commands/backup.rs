//! Scheduled shared-data backups: one Borg archive of the site's `shared`
//! directory followed by age-based retention pruning.

use std::fs;
use std::num::NonZeroU16;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use borgbackup::common::{CommonOptions, CreateOptions, PruneOptions, PruneWithin, PruneWithinTime};
use borgbackup::sync;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;

use crate::privileges;
use crate::release::SiteMutation;

const PASSPHRASE_MODE: u32 = 0o600;

/// Runs one scheduled backup for the site as root: archive `shared/`, then prune.
pub fn run(site: &str, keep_days: u16) -> Result<()> {
    privileges::ensure_root("bonesremote backup run")?;
    let mutation = SiteMutation::acquire(site)?;
    run_locked(&mutation, keep_days)
}

pub fn run_locked(mutation: &SiteMutation, keep_days: u16) -> Result<()> {
    let site = mutation.site();
    let passphrase = read_passphrase_file(&paths::bonesremote_site_passphrase_path(site))?;
    let repository = paths::site_backup_repository_path(site).display().to_string();
    let shared = mutation.shared_dir();
    let archive = archive_name(site, OffsetDateTime::now_utc())?;

    println!("Creating backup archive {archive}");
    let create = create_options(&repository, &archive, &shared.display().to_string(), &passphrase);
    sync::create(&create, &CommonOptions::default()).context("Failed to create the Borg archive")?;

    prune(&repository, &passphrase, keep_days)?;
    println!("Pruned archives older than {keep_days} days");
    Ok(())
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
    enforce_private_mode(path)?;
    Ok(passphrase)
}

fn enforce_private_mode(path: &Path) -> Result<()> {
    let mode = fs::metadata(path).context("Failed to read Borg passphrase file metadata")?.permissions().mode() & 0o777;
    if mode != PASSPHRASE_MODE {
        bail!(
            "Borg passphrase file {} must have mode 0600, found {:o}; re-provision the site before running backups",
            path.display(),
            mode
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;

    use anyhow::Result;
    use tempfile::tempdir;
    use time::macros::datetime;

    use super::*;

    #[test]
    fn archive_names_follow_the_release_timestamp_convention() -> Result<()> {
        let name = archive_name("atlas", datetime!(2026 - 08 - 28 12 : 34 : 56 UTC))?;

        assert_eq!(name, "atlas_20260828_123456");
        Ok(())
    }

    #[test]
    fn create_options_target_the_shared_directory_with_the_passphrase_via_env() {
        let options =
            create_options("/var/lib/bonesdeploy/backups/atlas.borg", "atlas_1", "/srv/sites/atlas/shared", "pw");

        assert_eq!(options.repository, "/var/lib/bonesdeploy/backups/atlas.borg");
        assert_eq!(options.archive, "atlas_1");
        assert_eq!(options.paths, vec!["/srv/sites/atlas/shared"]);
        assert_eq!(options.passphrase.as_deref(), Some("pw"));
    }

    #[test]
    fn prune_options_map_keep_days_to_an_age_based_window() -> Result<()> {
        let options = prune_options("/repo", "pw", 30)?;

        assert_eq!(options.repository, "/repo");
        assert_eq!(options.passphrase.as_deref(), Some("pw"));
        assert_eq!(options.keep_within.map(|window| window.to_string()).as_deref(), Some("30d"));
        assert!(options.keep_daily.is_none());
        Ok(())
    }

    #[test]
    fn prune_options_reject_zero_day_retention() {
        assert!(prune_options("/repo", "pw", 0).is_err());
    }

    #[test]
    fn passphrase_reader_requires_an_existing_private_file() -> Result<()> {
        let dir = tempdir()?;
        let path = dir.path().join(".borg_passphrase");

        assert!(read_passphrase_file(&path).is_err(), "missing file must fail");

        fs::write(&path, "")?;
        assert!(read_passphrase_file(&path).is_err(), "empty file must fail");

        fs::write(&path, "secret\n")?;
        fs::set_permissions(&path, Permissions::from_mode(0o644))?;
        assert!(
            read_passphrase_file(&path).is_err_and(|error| error.to_string().contains("0600")),
            "group-readable file must fail with a mode error"
        );

        fs::set_permissions(&path, Permissions::from_mode(0o600))?;
        assert_eq!(read_passphrase_file(&path)?, "secret");
        Ok(())
    }

    #[test]
    fn passphrase_paths_stay_inside_the_bonesremote_site_root() {
        let path: PathBuf = paths::bonesremote_site_passphrase_path("atlas");
        assert_eq!(path, PathBuf::from("/root/.config/bonesremote/sites/atlas/.borg_passphrase"));
        assert_eq!(
            paths::site_backup_repository_path("atlas"),
            PathBuf::from("/var/lib/bonesdeploy/backups/atlas.borg")
        );
    }
}
