use std::fs::File as StdFile;
use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;
use tokio::fs::File;

use crate::config;
use crate::infra::{self, ssh};
use crate::platform;
use crate::ui::{output, progress::ProgressReader, prompts};

pub async fn run(archive: &Path, yes: bool) -> Result<()> {
    let file = open_archive(archive)?;
    let archive_size = file.metadata()?.len();
    println!("Archive size: {archive_size} bytes.");

    let config = config::load(Path::new(paths::DOT_ENV)).context("Failed to load the project configuration")?;
    println!("Shared data for {} will be replaced.", config.project_name);
    println!("The archive's .env is ignored and the current environment is preserved.");
    println!("Services will briefly stop during replacement and restart afterward.");
    if !yes && !prompts::confirm_site_import(&config.project_name)? {
        println!("Skipped.");
        return Ok(());
    }

    let file = File::from_std(file);
    let reader = ProgressReader::new(file, archive_size, |percentage| {
        output::upload_progress("shared data archive", percentage);
    });
    let transport = ssh::SshTransport::connect_privileged(&config).await?;
    let transfer =
        transport.stream_cmd_with_reader(&infra::shared_import_command(&config.project_name), &[], reader).await;
    if transfer.is_err() {
        println!();
    }
    transfer?;
    println!("{} Shared data imported.", output::success_marker());
    println!("The existing .env was preserved and services were restarted.");
    Ok(())
}

fn open_archive(archive: &Path) -> Result<StdFile> {
    platform::open_regular_file(archive)
}
