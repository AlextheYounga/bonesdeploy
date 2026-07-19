use std::path::Path;

use anyhow::{Context, Result};
use console::style;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;
use shared::paths;

pub async fn run() -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml).context(super::deploy_project::local_bones_load_error())?;

    println!(
        "{} {} {} {}",
        style("Rolling back").yellow().bold(),
        style(&cfg.project_name).bold(),
        style("on").dim(),
        style(&cfg.host).dim(),
    );

    let session = ssh::connect_privileged(&cfg).await?;
    let command = format!("bonesremote release rollback --site {}", ssh::shell_quote(&cfg.project_name));
    ssh::stream_cmd(&session, &command).await?;
    session.close().await?;

    println!("{} Rollback complete.", output::success_marker());
    println!();
    println!("{}", output::next_step("bonesdeploy status"));

    Ok(())
}
