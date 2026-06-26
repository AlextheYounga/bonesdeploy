use std::path::Path;

use anyhow::Result;

use crate::config;
use crate::infra::ssh;
use shared::paths;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let remote_bones_toml = paths::bonesremote_bones_toml_path(&cfg.project_name);

    println!("Rolling back {} on {}...", cfg.project_name, cfg.host);

    let session = ssh::connect_privileged(&cfg).await?;
    let command = format!("bonesremote release rollback --config '{}'", remote_bones_toml.display());
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("Rollback complete.");
    println!();
    println!("Next: run bonesdeploy status.");

    Ok(())
}
