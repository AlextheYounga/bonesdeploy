use anyhow::Result;
use bonesdeploy_core::paths;
use std::path::Path;

use crate::commands::server::doctor;
use crate::ui::output;
use crate::ui::prompts;
use crate::{config, infra};

pub async fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_server_setup()? {
        println!("Skipped.");
        return Ok(());
    }

    println!("Setting up server baseline...");
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::server_request(&cfg)?;
    bonesinfra::run_with_request(
        &["server", "apply", "--request-stdin", "--bonesremote-version", env!("CARGO_PKG_VERSION")],
        &request,
    )?;
    doctor(false).await?;
    println!("{} Server baseline is ready.", output::success_marker());
    Ok(())
}
