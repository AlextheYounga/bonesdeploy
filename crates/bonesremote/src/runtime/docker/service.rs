use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::validate_site_name;
use bonesdeploy_core::paths;

use super::command;
use crate::control_plane;
use crate::privileges;

/// Starts the Docker Compose stack for a site.
///
/// This is invoked only by the Docker-specific systemd unit provisioned from
/// local config at setup time. The unit is installed only for Docker-backed
/// sites, so no remote `.env` read is needed to determine the runtime backend.
pub(crate) fn start(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote runtime start")?;
    validate_site_name(site)?;
    let project_root = paths::default_project_root_for(site);
    let descriptor = control_plane::load(site)?;
    let Some(wait_timeout) = descriptor.runtime.compose_wait_timeout() else {
        bail!("Docker runtime service cannot start for native site {site}")
    };
    command::active_start(site, project_root.as_ref(), u64::from(wait_timeout))
        .with_context(|| format!("Failed to start Docker Compose runtime for {site}"))
}

pub(crate) fn stop(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote runtime stop")?;
    validate_site_name(site)?;
    let project_root = paths::default_project_root_for(site);
    command::active_stop(site, project_root.as_ref())
        .with_context(|| format!("Failed to stop Docker Compose runtime for {site}"))
}
