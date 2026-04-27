use std::path::Path;

use anyhow::Result;
use console::style;

use crate::config;
use crate::ssh;

pub async fn run() -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let remote_bones_yaml = format!("{}/{}/bones.yaml", cfg.data.git_dir, config::Constants::REMOTE_BONES_DIR);

    println!("Rolling back {} on {}...", style(&cfg.data.project_name).cyan().bold(), style(&cfg.data.host).cyan());

    let session = ssh::connect(&cfg).await?;
    let command = format!("bonesremote release rollback --config '{remote_bones_yaml}'");
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("\n{} Rollback complete.", style("Done!").green().bold());

    Ok(())
}
