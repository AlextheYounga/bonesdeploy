use std::path::Path;

use anyhow::{Context, Result};
use console::style;

use crate::commands::push_state;
use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use shared::paths;

pub fn local_bones_load_error() -> String {
    format!("Failed to load {}", paths::LOCAL_BONES_TOML)
}

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml).context(local_bones_load_error())?;

    println!(
        "{} {} {} {}",
        style("Deploying").cyan().bold(),
        style(&cfg.project_name).bold(),
        style("to").dim(),
        style(&cfg.host).dim(),
    );

    // Ensure local .bones/ is published into the remote control plane before triggering deploy.
    push_state::sync_bones_directory(&cfg).context("Failed to publish .bones to bonesremote.")?;

    let session = ssh::connect_privileged(&cfg).await?;

    let command = format!("bonesremote deploy --site {}", ssh::shell_quote(&cfg.project_name));
    ssh::stream_cmd(&session, &command).await?;

    session.close().await?;

    println!("{} Deployment complete.", output::success_marker());

    Ok(())
}
