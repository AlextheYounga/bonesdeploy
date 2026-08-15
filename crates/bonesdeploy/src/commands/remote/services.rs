use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::config::PROJECT_SETUP_ERROR;
use bonesdeploy_core::paths;

use crate::config;
use crate::infra::git;
use crate::ui::prompts;

pub fn run(yes: bool, show_next: bool) -> Result<()> {
    git::ensure_git_repository()?;
    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() || !Path::new(paths::LOCAL_INFRA_DIR).is_dir() {
        bail!(PROJECT_SETUP_ERROR);
    }
    let cfg = config::load(env_file)?;
    if cfg.services.services.is_empty() {
        return Ok(());
    }
    if !yes && !prompts::confirm_remote_services()? {
        println!("Skipped service setup.");
        return Ok(());
    }
    println!("Provisioning services...");
    bonesinfra::run(&["services", "apply", "--env-file", paths::DOT_ENV])?;
    println!("Services applied.");
    if show_next {
        println!();
    }
    Ok(())
}
