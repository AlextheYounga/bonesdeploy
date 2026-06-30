use std::path::Path;

use anyhow::{Result, bail};

use crate::infra::bonesinfra;
use crate::infra::git;
use crate::ui::prompts;
use shared::paths;

pub fn run(yes: bool) -> Result<()> {
    git::ensure_git_repository()?;

    let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
    if !runtime_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_RUNTIME_TOML);
    }

    if !yes && !prompts::confirm_remote_runtime()? {
        println!("Skipped runtime setup.");
        println!();
        println!("Next: run bonesdeploy remote runtime when ready.");
        return Ok(());
    }

    println!("Applying runtime...");

    bonesinfra::run(&[
        "runtime",
        "apply",
        "--config",
        paths::LOCAL_BONES_TOML,
        "--runtime-config",
        paths::LOCAL_BONES_RUNTIME_TOML,
    ])?;

    println!("Runtime applied.");
    println!();
    println!("Next: run bonesdeploy push.");
    Ok(())
}
