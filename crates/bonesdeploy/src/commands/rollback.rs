use std::path::Path;

use anyhow::Result;
use console::style;

use crate::config;
use crate::ssh;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = format!("{}/{}/bones.toml", cfg.repo_path, config::Constants::REMOTE_BONES_DIR);

    println!("Rolling back {} on {}...", style(&cfg.project_name).cyan().bold(), style(&cfg.host).cyan());

    let session = ssh::connect(&cfg).await?;
    let command = format!("bonesremote release rollback --config '{remote_bones_toml}'");
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("\n{} Rollback complete.", style("Done!").green().bold());

    Ok(())
}
