use anyhow::Result;
use console::style;
use shared::paths;
use std::path::Path;

use crate::config;
use crate::infra::ssh;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = cfg.deployment_paths(paths::DEFAULT_WEB_ROOT).repo_bones_toml;

    println!("Deploying {} on {}...", style(&cfg.project_name).cyan().bold(), style(&cfg.host).cyan());

    let session = ssh::connect(&cfg).await?;

    println!("Running remote deploy...");
    ssh::stream_cmd(&session, &format!("bonesremote deploy --config '{remote_bones_toml}'")).await?;

    session.close().await?;

    println!("\n{} Deployment complete.", style("Done!").green().bold());

    Ok(())
}
