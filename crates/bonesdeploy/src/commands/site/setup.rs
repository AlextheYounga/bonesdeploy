use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

use crate::commands::server;
use crate::infra::ssh;
use crate::ui::output;
use crate::ui::prompts;
use crate::{config, infra};

pub async fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_site_setup()? {
        println!("Skipped.");
        return Ok(());
    }

    server::doctor(false).await.context("Server baseline is not ready.\n\nNext: bonesdeploy server setup --yes")?;
    println!("Provisioning site base...");
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    bonesinfra::run_with_request(&["site", "apply", "--request-stdin"], &request)?;
    super::runtime::apply()?;

    let session = ssh::connect_privileged(&cfg).await?;
    let reactivate = infra::decommission_command("reactivate", &cfg.project_name);
    ssh::run_cmd(&session, &reactivate).await?;
    session.close().await?;

    super::doctor::run_with_pending(false, false).await.context("Site setup failed while checking site")?;
    println!();
    println!("{} Site setup complete.", output::success_marker());
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    print_next_step(&cfg).await;
    Ok(())
}

async fn print_next_step(cfg: &config::Bones) {
    if cfg.domain.is_empty() {
        println!("{}", output::next_step("bonesdeploy deploy"));
        println!("Optional public preview: run `bonesdeploy site tunnel start` to create an ephemeral Cloudflare URL.");
    } else if cfg.ssl_enabled {
        println!("{}", output::next_step("bonesdeploy deploy"));
    } else {
        println!("{}", output::next_step_with_detail("bonesdeploy site ssl", "to configure HTTPS"));
    }
}
