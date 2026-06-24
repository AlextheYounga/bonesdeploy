use std::path::Path;

use anyhow::{Result, bail};

use crate::infra::bonesinfra_cli;
use crate::infra::git;
use crate::ui::output;
use crate::ui::prompts;
use shared::paths;

pub fn run(yes: bool, show_next: bool) -> Result<()> {
    git::ensure_git_repository()?;

    let runtime_toml = Path::new(paths::LOCAL_BONES_RUNTIME_TOML);
    if !runtime_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_RUNTIME_TOML);
    }

    if !yes && !prompts::confirm_remote_runtime()? {
        println!("Skipped runtime setup.");
        println!();
        println!("Next: run {} when ready.", output::command("bonesdeploy remote runtime"));
        return Ok(());
    }

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    println!("Applying runtime...");

    bonesinfra_cli::run(&[
        "runtime",
        "apply",
        "--config",
        bones_toml.to_str().unwrap_or(".bones/bones.toml"),
        "--runtime-config",
        runtime_toml.to_str().unwrap_or(".bones/runtime.toml"),
    ])?;

    println!("Runtime applied.");
    if show_next {
        println!();
        output::next("bonesdeploy push");
    }
    Ok(())
}
