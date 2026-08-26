use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

use crate::commands::server;
use crate::ui::output;
use crate::ui::prompts;
use crate::{config, infra};

use console::style;

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
    super::services::apply()?;
    super::runtime::apply()?;

    let pending_first_push =
        super::doctor::run_with_pending(false, false).await.context("Site setup failed while checking site")?;
    println!();
    println!("{} Site setup complete.", output::success_marker());
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    print_next_step(&cfg, pending_first_push).await;
    Ok(())
}

async fn print_next_step(cfg: &config::Bones, pending_first_push: bool) {
    if pending_first_push {
        println!(
            "{}",
            output::next_step_with_detail(
                &format!("git push {} {}", cfg.remote_name, cfg.branch),
                "to publish the first deploy branch",
            )
        );
    } else if cfg.domain.is_empty() {
        match super::status::remote_status(cfg).await {
            Ok(remote) => match remote.preview.and_then(|preview| preview.active.then_some(preview.url).flatten()) {
                Some(url) => println!("{} {url}", style("Preview").dim()),
                None => println!(
                    "{} Quick Tunnel is starting; run `bonesdeploy status` for its URL.",
                    output::pending_marker()
                ),
            },
            Err(error) => println!("{} Preview status unavailable: {error:#}", output::pending_marker()),
        }
        println!("{}", output::next_step("bonesdeploy deploy"));
    } else if cfg.ssl_enabled {
        println!("{}", output::next_step("bonesdeploy deploy"));
    } else {
        println!("{}", output::next_step_with_detail("bonesdeploy site ssl", "to configure HTTPS"));
    }
}
