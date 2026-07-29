use std::path::Path;

use anyhow::{Result, bail};

use crate::infra::git;
use crate::ui::output;
use crate::ui::prompts;
use shared::paths;

pub fn run(yes: bool, show_next: bool) -> Result<()> {
    git::ensure_git_repository()?;

    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    if !bones_toml.exists() {
        bail!("{} does not exist. Run `bonesdeploy init` first.", paths::LOCAL_BONES_TOML);
    }

    if !yes && !prompts::confirm_remote_framework()? {
        println!("Skipped framework setup.");
        if show_next {
            println!();
            println!("{}", output::next_step_with_detail("bonesdeploy remote framework", "when ready"));
        }
        return Ok(());
    }

    println!("Applying framework...");

    bonesinfra::run(&["framework", "apply", "--config", paths::LOCAL_BONES_TOML])?;

    println!("Framework applied.");
    if show_next {
        println!();
        println!("{}", output::next_step("bonesdeploy push"));
    }
    Ok(())
}
