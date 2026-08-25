use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

use crate::commands::server;
use crate::config;
use crate::ui::output;
use crate::ui::prompts;

pub async fn run(yes: bool) -> Result<()> {
    if !yes && !prompts::confirm_site_setup()? {
        println!("Skipped.");
        return Ok(());
    }

    server::doctor(false).await.context("Server baseline is not ready.\n\nNext: bonesdeploy server setup --yes")?;
    println!("Provisioning site base...");
    bonesinfra::run(&["site", "apply", "--env-file", paths::DOT_ENV])?;
    super::services::apply()?;
    super::runtime::apply()?;

    let pending_first_push =
        super::doctor::run_with_pending(false, false).await.context("Site setup failed while checking site")?;
    println!();
    println!("{} Site setup complete.", output::success_marker());
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    print_next_step(&cfg, pending_first_push);
    Ok(())
}

fn print_next_step(cfg: &config::Bones, pending_first_push: bool) {
    if pending_first_push {
        println!(
            "{}",
            output::next_step_with_detail(
                &format!("git push {} {}", cfg.remote_name, cfg.branch),
                "to publish the first deploy branch",
            )
        );
    } else if cfg.ssl_enabled {
        println!("{}", output::next_step("bonesdeploy deploy"));
    } else {
        println!("{}", output::next_step_with_detail("bonesdeploy site ssl", "to configure HTTPS"));
    }
}
