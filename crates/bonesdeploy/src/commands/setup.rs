use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;
use console::style;

use crate::commands::{doctor, remote, status};
use crate::config;
use crate::ui::output;

pub async fn run(skip_confirm: bool) -> Result<()> {
    let bones_toml = Path::new(paths::DOT_ENV);
    let cfg = config::load(bones_toml)?;

    println!("{} {}", style("Setting up").cyan().bold(), style(&cfg.host).bold());

    remote::bootstrap::run(skip_confirm, false).with_context(|| setup_error("bootstrapping remote server"))?;
    remote::services::run(true, false).with_context(|| setup_error("provisioning services"))?;
    remote::runtime::run(true, false).with_context(|| setup_error("applying runtime"))?;
    let pending_first_push = doctor::run(false, false).await.with_context(|| setup_error("checking deployment"))?;

    println!();
    println!("{} Setup complete.", output::success_marker());
    println!();
    if pending_first_push {
        println!(
            "{}",
            output::next_step_with_detail(
                &format!("git push {} {}", cfg.remote_name, cfg.branch),
                "to publish the first deploy branch",
            )
        );
    } else if cfg.domain.is_empty() {
        match status::remote_status(&cfg).await {
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
        println!("{}", output::next_step_with_detail("bonesdeploy remote ssl", "to configure HTTPS"));
    }

    Ok(())
}

fn setup_error(step: &str) -> String {
    format!(
        "Setup failed while {step}.\n\nNext: fix the error above, then {} again.",
        output::run_command("bonesdeploy setup")
    )
}
