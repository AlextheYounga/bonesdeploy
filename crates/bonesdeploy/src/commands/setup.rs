use std::path::Path;

use anyhow::{Context, Result};
use shared::paths;

use crate::commands::{doctor, push_state, remote_runtime, remote_setup};
use crate::config;
use crate::ui::output;

pub async fn run(_yes: bool) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    println!("Setting up deployment...");

    remote_setup::run(false).with_context(|| setup_error("bootstrapping remote server"))?;
    remote_runtime::run(true, false).with_context(|| setup_error("applying runtime"))?;
    push_state::run(false).await.with_context(|| setup_error("syncing .bones"))?;
    doctor::run(false).await.with_context(|| setup_error("checking deployment"))?;

    println!();
    println!("Setup complete.");
    println!();
    if cfg.ssl_enabled {
        output::next("bonesdeploy deploy");
    } else {
        println!("Next: run {} to configure HTTPS.", output::command("bonesdeploy remote ssl"));
    }

    Ok(())
}

fn setup_error(step: &str) -> String {
    format!("Setup failed while {step}.\n\nNext: fix the error above, then run bonesdeploy setup again.")
}
