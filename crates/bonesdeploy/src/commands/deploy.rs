use std::path::Path;

use anyhow::{Context, Result};
use console::style;

use crate::config;
use crate::infra::{self, ssh};
use crate::ui::output;
use bonesdeploy_core::config::RemoteDeploymentConfig;
use bonesdeploy_core::paths;

pub fn local_bones_load_error() -> String {
    format!("Failed to load root {}", paths::DOT_ENV)
}

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::DOT_ENV);
    let cfg = config::load(bones_toml).context(local_bones_load_error())?;

    println!(
        "{} {} {} {}",
        style("Deploying").cyan().bold(),
        style(&cfg.project_name).bold(),
        style("to").dim(),
        style(&cfg.host).dim(),
    );

    let descriptor = RemoteDeploymentConfig::from_bones(&cfg);
    let descriptor_json =
        format!("{}\n", serde_json::to_string_pretty(&descriptor).context("Failed to serialize deployment config")?);

    let session = ssh::connect_privileged(&cfg).await?;
    infra::sync_control_plane(&session, &cfg).await?;
    let command = format!("bonesremote deploy --site {} --config-stdin", ssh::shell_quote(&cfg.project_name));
    ssh::stream_cmd_with_stdin(&session, &command, descriptor_json.as_bytes()).await?;
    session.close().await?;

    println!("{} Deployment complete.", output::success_marker());
    Ok(())
}
