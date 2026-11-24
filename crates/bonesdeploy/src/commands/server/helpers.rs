use anyhow::Result;
use bonesdeploy_core::paths;
use bonesinfra::provisioning::provision_helpers;
use console::style;
use std::path::Path;

use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub async fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_server_helpers()? {
        println!("Skipped.");
        return Ok(());
    }

    println!("{}", style("Installing server helper tools").cyan().bold());
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    provision_helpers(&cfg, Path::new(".")).await?;
    println!("{} Helper tools installed.", output::success_marker());
    Ok(())
}
