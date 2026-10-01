use std::fs::{File as StdFile, OpenOptions};
use std::os::unix::fs::OpenOptionsExt;
use std::path::Path;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use tokio::fs::File;

use crate::config;
use crate::infra::{self, ssh};
use crate::ui::{output, prompts};

pub async fn run(archive: &Path, yes: bool) -> Result<()> {
    let file = open_archive(archive)?;

    let config = config::load(Path::new(paths::DOT_ENV)).context("Failed to load the project configuration")?;
    println!("Shared data for {} will be replaced.", config.project_name);
    println!("The archive's .env is ignored and the current environment is preserved.");
    println!("Services will briefly stop during replacement and restart afterward.");
    if !yes && !prompts::confirm_site_import(&config.project_name)? {
        println!("Skipped.");
        return Ok(());
    }

    let file = File::from_std(file);
    let transport = ssh::SshTransport::connect_privileged(&config).await?;
    let transfer_result =
        transport.stream_cmd_with_reader(&infra::shared_import_command(&config.project_name), &[], file).await;
    let close_result = transport.close().await;

    transfer_result?;
    close_result.context("Failed to close the SSH session")?;
    println!("{} Shared data imported.", output::success_marker());
    println!("The existing .env was preserved and services were restarted.");
    Ok(())
}

fn open_archive(archive: &Path) -> Result<StdFile> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(archive)
        .with_context(|| format!("Failed to open archive {}", archive.display()))?;
    if !file.metadata()?.is_file() {
        bail!("Archive is not a regular file: {}", archive.display());
    }
    Ok(file)
}
