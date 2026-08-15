use std::path::Path;

use anyhow::{Result, bail};
use bonesdeploy_core::config::PROJECT_SETUP_ERROR;

use crate::infra::git;
use crate::ui::output;
use crate::ui::prompts;
use bonesdeploy_core::paths;

pub fn run(yes: bool, show_next: bool) -> Result<()> {
    git::ensure_git_repository()?;

    let env_file = Path::new(paths::DOT_ENV);
    if !env_file.exists() || !Path::new(paths::LOCAL_INFRA_DIR).is_dir() {
        bail!(PROJECT_SETUP_ERROR);
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

    bonesinfra::run(&["runtime", "apply", "--env-file", paths::DOT_ENV])?;

    println!("Runtime applied.");

    Ok(())
}
