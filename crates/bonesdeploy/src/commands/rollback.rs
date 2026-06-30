use std::path::Path;

use anyhow::{Context, Result};

use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use shared::paths;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml).context(super::deploy_project::local_bones_load_error())?;

    println!("Rolling back {} on {}...", cfg.project_name, cfg.host);

    let session = ssh::connect_privileged(&cfg).await?;
    let command = format!("bonesremote release rollback --site '{}'", single_quote(&cfg.project_name));
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("Rollback complete.");
    println!();
    println!("{}", output::next_step("bonesdeploy status"));

    Ok(())
}

fn single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
