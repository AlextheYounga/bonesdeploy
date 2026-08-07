use std::path::Path;

use anyhow::{Context, Result};
use console::style;

use crate::commands::{push_state, secrets};
use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use bonesdeploy_core::paths;

pub fn local_bones_load_error() -> String {
    format!("Failed to load {}", paths::local_bones_toml())
}

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::local_bones_toml());
    let cfg = config::load(bones_toml).context(local_bones_load_error())?;

    println!(
        "{} {} {} {}",
        style("Deploying").cyan().bold(),
        style(&cfg.project_name).bold(),
        style("to").dim(),
        style(&cfg.host).dim(),
    );

    push_state::sync_bones_directory().context("Failed to publish .bones to bonesremote.")?;
    secrets::push().await.context("Failed to push environment secrets.")?;

    let session = ssh::connect_privileged(&cfg).await?;
    let command = format!("bonesremote deploy --site {}", ssh::shell_quote(&cfg.project_name));
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("{} Deployment complete.", output::success_marker());
    Ok(())
}
