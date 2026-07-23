use std::path::Path;

use anyhow::{Result, bail};
use shared::paths;

use crate::config;
use crate::infra::bonesinfra;
use crate::infra::git;
use crate::ui::{output, prompts};

pub fn run(yes: bool, show_next: bool) -> Result<()> {
    git::ensure_git_repository()?;
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if !bones_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_TOML);
    }
    let cfg = config::load(bones_toml)?;
    if cfg.dbs.services.is_empty() {
        return Ok(());
    }
    if !yes && !prompts::confirm_remote_dbs()? {
        println!("Skipped database setup.");
        return Ok(());
    }
    println!("Provisioning database services...");
    bonesinfra::run(&["dbs", "apply", "--config", paths::LOCAL_BONES_TOML])?;
    println!("Database services applied.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy push"));
    }
    Ok(())
}
