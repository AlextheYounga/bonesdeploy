use std::path::Path;

use anyhow::{Context, Result};
use console::style;
use shared::paths;

use crate::commands::{doctor, push_state, remote_bootstrap, remote_runtime};
use crate::config;
use crate::ui::output;

pub async fn run(skip_confirm: bool) -> Result<()> {
    let bones_toml = Path::new(paths::LOCAL_BONES_TOML);
    let cfg = config::load(bones_toml)?;

    println!("{} {}", style("Setting up").cyan().bold(), style(&cfg.host).bold(),);

    remote_bootstrap::run(skip_confirm, false).with_context(|| setup_error("bootstrapping remote server"))?;
    remote_runtime::run(true, false).with_context(|| setup_error("applying runtime"))?;
    push_state::run(false).with_context(|| setup_error("syncing .bones"))?;
    let pending_first_push = doctor::run(false).await.with_context(|| setup_error("checking deployment"))?;

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
