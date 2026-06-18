use anyhow::Result;
use console::style;
use shared::paths;
use std::path::Path;

use crate::config;
use crate::ssh;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = cfg.data.deployment_paths(paths::DEFAULT_WEB_ROOT).repo_bones_toml;

    println!("Deploying {} on {}...", style(&cfg.data.project_name).cyan().bold(), style(&cfg.data.host).cyan());

    let session = ssh::connect(&cfg).await?;

    println!("Running remote deploy...");
    ssh::stream_cmd(&session, &format!("bonesremote deploy --config '{remote_bones_toml}'")).await?;

    session.close().await?;

    println!("\n{} Deployment complete.", style("Done!").green().bold());

    Ok(())
}
