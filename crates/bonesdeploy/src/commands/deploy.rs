use anyhow::Result;
use console::style;
use std::path::Path;

use crate::config;
use crate::ssh;

pub async fn run() -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let repo_path = &cfg.data.repo_path;

    println!("Deploying {} on {}...", style(&cfg.data.project_name).cyan().bold(), style(&cfg.data.host).cyan());

    let session = ssh::connect(&cfg).await?;

    println!("Running pre-receive...");
    ssh::stream_cmd(
        &session,
        &format!(
            "BONES_FORCE_DEPLOY=1 GIT_DIR='{repo_path}' '{repo_path}/{}/{}' </dev/null",
            config::Constants::REMOTE_HOOKS_DIR,
            config::Constants::PRE_RECEIVE_HOOK
        ),
    )
    .await?;

    println!("Running post-receive...");
    ssh::stream_cmd(
        &session,
        &format!(
            "BONES_FORCE_DEPLOY=1 GIT_DIR='{repo_path}' '{repo_path}/{}/{}' </dev/null",
            config::Constants::REMOTE_HOOKS_DIR,
            config::Constants::POST_RECEIVE_HOOK
        ),
    )
    .await?;

    session.close().await?;

    println!("\n{} Deployment complete.", style("Done!").green().bold());

    Ok(())
}
