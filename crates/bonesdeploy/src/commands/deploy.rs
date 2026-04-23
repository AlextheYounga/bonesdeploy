use std::path::Path;

use anyhow::Result;
use console::style;

use crate::config;
use crate::ssh;

const BONES_TOML: &str = ".bones/bones.toml";

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let git_dir = &cfg.data.git_dir;

    println!("Deploying {} on {}...", style(&cfg.data.project_name).cyan().bold(), style(&cfg.data.host).cyan());

    let session = ssh::connect(&cfg).await?;

    println!("Running pre-receive...");
    ssh::stream_cmd(
        &session,
        &format!("BONES_FORCE_DEPLOY=1 GIT_DIR='{git_dir}' '{git_dir}/hooks/pre-receive' </dev/null"),
    )
    .await?;

    println!("Running post-receive...");
    ssh::stream_cmd(
        &session,
        &format!("BONES_FORCE_DEPLOY=1 GIT_DIR='{git_dir}' '{git_dir}/hooks/post-receive' </dev/null"),
    )
    .await?;

    session.close().await?;

    println!("\n{} Deployment complete.", style("Done!").green().bold());

    Ok(())
}
