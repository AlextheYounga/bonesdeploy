use anyhow::Result;
use bonesdeploy_core::paths;

use crate::commands::server::doctor;
use crate::ui::output;
use crate::ui::prompts;

pub async fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_server_setup()? {
        println!("Skipped.");
        return Ok(());
    }

    println!("Setting up server baseline...");
    bonesinfra::run(&[
        "server",
        "apply",
        "--env-file",
        paths::DOT_ENV,
        "--bonesremote-version",
        env!("CARGO_PKG_VERSION"),
    ])?;
    doctor(false).await?;
    println!("{} Server baseline is ready.", output::success_marker());
    Ok(())
}
