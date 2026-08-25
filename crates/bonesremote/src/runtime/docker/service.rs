use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use bonesdeploy_core::config::{runtime_user_for, validate_site_name};
use bonesdeploy_core::paths;

use super::command::{application_command, image_name};
use crate::privileges;

/// Starts the Docker application container for a site.
///
/// This is invoked only by the Docker-specific systemd unit provisioned from
/// local config at setup time. The unit is installed only for Docker-backed
/// sites, so no remote `.env` read is needed to determine the runtime backend.
pub(crate) fn start(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote runtime start")?;
    validate_site_name(site)?;
    let project_root = paths::default_project_root_for(site);
    let image = image_name(site)?;
    let mut command = application_command(site, Path::new(&project_root), &runtime_user_for(site), &image)?;
    let status = command.status().with_context(|| format!("Failed to start Docker runtime for {site}"))?;
    if !status.success() {
        anyhow::bail!("Docker runtime for {site} exited with status {status}");
    }
    Ok(())
}

pub(crate) fn stop(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote runtime stop")?;
    validate_site_name(site)?;
    let name = super::command::container_name(site)?;
    let status = Command::new("docker").args(["rm", "--force", &name]).status()?;
    if !status.success() && status.code() != Some(1) {
        anyhow::bail!("Failed to stop Docker runtime for {site}: {status}");
    }
    Ok(())
}
