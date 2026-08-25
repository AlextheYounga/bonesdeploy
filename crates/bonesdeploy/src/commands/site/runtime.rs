use anyhow::Result;
use bonesdeploy_core::paths;

use crate::ui::output;
use crate::ui::prompts;

pub fn run(yes: bool) -> Result<()> {
    super::readiness::ensure_project_ready()?;
    if !yes && !prompts::confirm_site_runtime()? {
        println!("Skipped runtime setup.");
        println!();
        println!("{}", output::next_step_with_detail("bonesdeploy site runtime", "when ready"));
        return Ok(());
    }
    apply()?;
    println!("Runtime applied.");
    Ok(())
}

pub(super) fn apply() -> Result<()> {
    println!("Applying runtime...");
    bonesinfra::run(&["runtime", "apply", "--env-file", paths::DOT_ENV])?;
    Ok(())
}
