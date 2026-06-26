use anyhow::Result;
use shared::paths;
use std::path::Path;

use crate::commands::push_state;
use crate::config;
use crate::infra::ssh;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = paths::bonesremote_bones_toml_path(&cfg.project_name);

    println!("Deploying {} to {}...", cfg.project_name, cfg.host);

    // Ensure local .bones/ is synced before triggering the remote deploy
    push_state::sync_bones_directory(&cfg)?;

    let session = ssh::connect_privileged(&cfg).await?;

    println!("Running remote deploy...");
    ssh::stream_cmd(&session, &format!("bonesremote deploy --config '{}'", remote_bones_toml.display())).await?;

    session.close().await?;

    println!("Deployment complete.");

    Ok(())
}
