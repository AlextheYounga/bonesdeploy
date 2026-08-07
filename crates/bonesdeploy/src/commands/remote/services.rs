use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::paths;

use crate::config;
use crate::infra::git;
use crate::ui::{output, prompts};

pub fn run(yes: bool, show_next: bool) -> Result<()> {
    git::ensure_git_repository()?;
    let bones_toml = Path::new(paths::local_bones_toml());
    if !bones_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::local_bones_toml());
    }
    let cfg = config::load(bones_toml)?;
    if cfg.services.services.is_empty() {
        return Ok(());
    }
    if !yes && !prompts::confirm_remote_services()? {
        println!("Skipped service setup.");
        return Ok(());
    }
    println!("Provisioning services...");
    bonesinfra::run(&["services", "apply", "--config", paths::local_bones_toml()])?;
    println!("Services applied.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy push"));
    }
    Ok(())
}
