use anyhow::Result;
use bonesdeploy_core::paths;
use bonesinfra::provisioning::apply_server_setup;
use std::path::Path;

use crate::commands::server::doctor;
use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub async fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_server_setup()? {
        println!("Skipped.");
        return Ok(());
    }

    println!("Setting up server baseline...");
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    apply_server_setup(&cfg, Path::new("."), env!("CARGO_PKG_VERSION")).await?;
    doctor(false).await?;
    println!("{} Server baseline is ready.", output::success_marker());
    Ok(())
}
