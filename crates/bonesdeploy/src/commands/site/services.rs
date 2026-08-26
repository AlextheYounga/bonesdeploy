use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::config::ProvisioningRequest;
use bonesdeploy_core::paths;

use crate::commands::secrets;
use crate::config;
use crate::ui::prompts;

pub fn run(yes: bool) -> Result<()> {
    super::readiness::ensure_project_ready()?;
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    if cfg.services.services.is_empty() {
        return Ok(());
    }
    if !yes && !prompts::confirm_site_services()? {
        println!("Skipped service setup.");
        return Ok(());
    }
    apply()?;
    println!("Services applied.");
    println!();
    Ok(())
}

pub(super) fn apply() -> Result<()> {
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    if cfg.services.services.is_empty() {
        return Ok(());
    }
    println!("Provisioning services...");
    let request = service_request(&cfg)?;
    bonesinfra::run_with_request(&["services", "apply", "--request-stdin"], &request)?;
    Ok(())
}

fn service_request(cfg: &config::Bones) -> Result<String> {
    let mut request = ProvisioningRequest::from_bones(cfg)?;
    request.services = Some(secrets::read_service_credentials(cfg)?);
    serde_json::to_string(&request).context("Failed to serialize services request")
}
