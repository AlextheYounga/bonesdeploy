use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use bonesdeploy_core::config::{RuntimeBackend, load_runtime, runtime_user_for};
use bonesdeploy_core::paths;

use super::command::{application_command, image_name};
use crate::privileges;
use crate::release::lifecycle::load_site_config;

pub(crate) fn start(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote runtime start")?;
    let config = load_site_config(site)?;
    let runtime = load_runtime(&paths::bonesremote_site_root(site))?;
    if runtime.backend != RuntimeBackend::Docker {
        anyhow::bail!("Site {site} does not use the Docker runtime backend");
    }
    let image = image_name(&config.project_name)?;
    let mut command = application_command(
        &config.project_name,
        Path::new(&config.project_root),
        &runtime_user_for(&config.project_name),
        &image,
    )?;
    let status = command.status().with_context(|| format!("Failed to start Docker runtime for {site}"))?;
    if !status.success() {
        anyhow::bail!("Docker runtime for {site} exited with status {status}");
    }
    Ok(())
}

pub(crate) fn stop(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote runtime stop")?;
    let config = load_site_config(site)?;
    let name = super::command::container_name(&config.project_name)?;
    let status = Command::new("docker").args(["rm", "--force", &name]).status()?;
    if !status.success() && status.code() != Some(1) {
        anyhow::bail!("Failed to stop Docker runtime for {site}: {status}");
    }
    Ok(())
}
