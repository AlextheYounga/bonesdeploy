use std::path::Path;

use anyhow::Result;
use console::style;

use crate::config;
use crate::ssh;

const BONES_TOML: &str = ".bones/bones.toml";

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = format!("{}/bones/bones.toml", cfg.data.git_dir);

    println!(
        "Rolling back {} on {}...",
        style(&cfg.data.project_name).cyan().bold(),
        style(&cfg.data.host).cyan()
    );

    let session = ssh::connect(&cfg).await?;
    let command = format!("sudo gitbones-remote rollback --config '{remote_bones_toml}'");
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("\n{} Rollback complete.", style("Done!").green().bold());

    Ok(())
}
