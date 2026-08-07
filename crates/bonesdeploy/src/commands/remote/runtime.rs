use std::path::Path;

use anyhow::{Result, bail};

use crate::infra::git;
use crate::ui::output;
use crate::ui::prompts;
use bonesdeploy_core::paths;

pub fn run(yes: bool, show_next: bool) -> Result<()> {
    git::ensure_git_repository()?;

    let bones_toml = Path::new(paths::local_bones_toml());
    if !bones_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::local_bones_toml());
    }

    if !yes && !prompts::confirm_remote_runtime()? {
        println!("Skipped runtime setup.");
        if show_next {
            println!();
            println!("{}", output::next_step_with_detail("bonesdeploy remote runtime", "when ready"));
        }
        return Ok(());
    }

    println!("Applying runtime...");

    bonesinfra::run(&["runtime", "apply", "--config", paths::local_bones_toml()])?;

    println!("Runtime applied.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy push"));
    }
    Ok(())
}
