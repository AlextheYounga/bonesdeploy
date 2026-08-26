use anyhow::Result;
use bonesdeploy_core::paths;
use std::path::Path;

use crate::ui::output;
use crate::ui::prompts;
use crate::{config, infra};

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
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    bonesinfra::run_with_request(&["runtime", "apply", "--request-stdin"], &request)?;
    Ok(())
}
