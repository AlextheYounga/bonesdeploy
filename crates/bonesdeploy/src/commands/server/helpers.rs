use anyhow::Result;
use bonesdeploy_core::paths;
use console::style;
use std::path::Path;

use crate::ui::output;
use crate::ui::prompts;
use crate::{config, infra};

pub fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_server_helpers()? {
        println!("Skipped.");
        return Ok(());
    }

    println!("{}", style("Installing server helper tools").cyan().bold());
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    bonesinfra::run_with_request(&["helpers", "apply", "--request-stdin"], &request)?;
    println!("{} Helper tools installed.", output::success_marker());
    Ok(())
}
