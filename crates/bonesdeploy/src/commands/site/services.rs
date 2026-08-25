use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::paths;

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
    bonesinfra::run(&["services", "apply", "--env-file", paths::DOT_ENV])?;
    Ok(())
}
