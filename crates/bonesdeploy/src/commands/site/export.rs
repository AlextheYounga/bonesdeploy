use std::fs::{File as StdFile, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use console::style;
use tempfile::NamedTempFile;
use time::OffsetDateTime;
use time::format_description::FormatItem;
use time::macros::format_description;
use tokio::fs::File;
use tokio::io::AsyncWriteExt;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;

pub async fn run(output_path: Option<&Path>) -> Result<()> {
    let config = config::load(Path::new(paths::DOT_ENV)).context("Failed to load the project configuration")?;
    let destination = resolve_output_path(output_path, &config.project_name, OffsetDateTime::now_utc())?;
    let export = ExportFile::create(destination)?;
    let transport = ssh::SshTransport::connect_privileged(&config).await?;
    let mut writer = export.writer()?;
    let command = archive_command(&config.project_name);
    let transfer_result = transport.download_cmd(&command, &mut writer).await;
    let close_result = transport.close().await;
    transfer_result?;
    close_result.context("Failed to close the SSH session")?;
    writer.flush().await.context("Failed to flush the shared export")?;
    writer.sync_all().await.context("Failed to sync the shared export")?;
    drop(writer);

    let destination = export.persist()?;
    println!("{} Shared export saved to {}", output::success_marker(), style(destination.display()).bold());
    Ok(())
}

fn archive_command(site: &str) -> String {
    let project_root = paths::default_project_root_for(site);
    format!("cd {} && exec zip -q -r -y - shared", ssh::shell_quote(&project_root))
}

fn resolve_output_path(output_path: Option<&Path>, site: &str, now: OffsetDateTime) -> Result<PathBuf> {
    let output_path = output_path.unwrap_or_else(|| Path::new("."));
    if output_path.is_dir() {
        return Ok(output_path.join(generated_filename(site, now)?));
    }
    if output_path.file_name().is_none() {
        bail!("Export output must be a filename or an existing directory");
    }
    Ok(output_path.to_path_buf())
}

fn generated_filename(site: &str, now: OffsetDateTime) -> Result<String> {
    static TIMESTAMP_FORMAT: &[FormatItem<'static>] = format_description!("[year][month][day]_[hour][minute][second]");
    let timestamp = now.format(&TIMESTAMP_FORMAT).context("Failed to format the export timestamp")?;
    Ok(format!("{site}-shared-{timestamp}.zip"))
}

struct ExportFile {
    staging: NamedTempFile,
    destination: PathBuf,
}

impl ExportFile {
    fn create(destination: PathBuf) -> Result<Self> {
        if destination.try_exists().context("Failed to inspect the export destination")? {
            bail!("Refusing to overwrite existing export: {}", destination.display());
        }

        let parent = destination.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
        if !parent.is_dir() {
            bail!("Export destination directory does not exist: {}", parent.display());
        }

        let staging = NamedTempFile::new_in(parent).context("Failed to create the temporary export file")?;
        staging
            .as_file()
            .set_permissions(Permissions::from_mode(0o600))
            .context("Failed to protect the temporary export file")?;
        Ok(Self { staging, destination })
    }

    fn writer(&self) -> Result<File> {
        let file = self.staging.as_file().try_clone().context("Failed to open the temporary export file")?;
        Ok(File::from_std(file))
    }

    fn persist(self) -> Result<PathBuf> {
        let Self { staging, destination } = self;
        let file = staging
            .persist_noclobber(&destination)
            .map_err(|error| error.error)
            .with_context(|| format!("Failed to save shared export to {}", destination.display()))?;
        file.sync_all().context("Failed to sync the saved shared export")?;
        sync_parent(&destination)?;
        Ok(destination)
    }
}

fn sync_parent(path: &Path) -> Result<()> {
    let parent = path.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    StdFile::open(parent)?.sync_all().context("Failed to sync the export destination directory")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::os::unix::fs::PermissionsExt;

    use tempfile::tempdir;
    use time::macros::datetime;

    use super::*;

    #[test]
    fn archive_command_runs_from_the_canonical_site_root() {
        let site = "atlas";
        let expected_root = paths::default_project_root_for(site);

        assert_eq!(
            archive_command(site),
            format!("cd {} && exec zip -q -r -y - shared", ssh::shell_quote(&expected_root))
        );
    }

    #[test]
    fn output_directory_uses_the_site_timestamp_filename() -> Result<()> {
        let directory = tempdir()?;
        let now = datetime!(2026-09-16 12:34:56 UTC);

        assert_eq!(
            resolve_output_path(Some(directory.path()), "atlas", now)?,
            directory.path().join("atlas-shared-20260916_123456.zip")
        );
        Ok(())
    }

    #[test]
    fn export_file_publishes_a_private_archive() -> Result<()> {
        let directory = tempdir()?;
        let destination = directory.path().join("atlas.zip");
        let export = ExportFile::create(destination.clone())?;
        assert_eq!(export.staging.as_file().metadata()?.permissions().mode() & 0o777, 0o600);

        export.staging.as_file().try_clone()?.write_all(b"archive")?;
        assert_eq!(export.persist()?, destination);
        assert_eq!(fs::read(&destination)?, b"archive");
        assert_eq!(fs::metadata(destination)?.permissions().mode() & 0o777, 0o600);
        Ok(())
    }

    #[test]
    fn export_file_does_not_replace_a_destination_created_during_transfer() -> Result<()> {
        let directory = tempdir()?;
        let destination = directory.path().join("atlas.zip");
        let export = ExportFile::create(destination.clone())?;
        fs::write(&destination, b"existing archive")?;

        assert!(export.persist().is_err());
        assert_eq!(fs::read(&destination)?, b"existing archive");
        assert_eq!(fs::read_dir(directory.path())?.count(), 1);
        Ok(())
    }
}
