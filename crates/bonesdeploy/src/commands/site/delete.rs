use std::path::Path;

use anyhow::{Context, Result};
use bonesdeploy_core::paths;

use crate::config;
use crate::infra::{self, ssh};
use crate::ui::{output, prompts};

pub async fn run(yes: bool) -> Result<()> {
    super::readiness::ensure_project_ready()?;
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    let preflight = bonesinfra::run_with_request_output(&["site", "preflight", "--request-stdin"], &request)
        .context("Site deletion preflight failed; no remote resources were changed")?;

    println!("Remote site to delete: {}", cfg.project_name);
    println!("Validated deletion inventory:");
    println!("{}", preflight.trim());
    if !yes && !prompts::confirm_site_delete(&cfg.project_name)? {
        println!("Skipped.");
        return Ok(());
    }

    let session = ssh::SshTransport::connect_privileged(&cfg).await?;
    let begin = infra::decommission_command("begin", &cfg.project_name);
    let plan = session.run_cmd_with_stdin_output(&begin, preflight.as_bytes()).await?;
    let plan = plan.trim();
    if plan.is_empty() {
        anyhow::bail!("BonesRemote returned an empty deletion plan");
    }

    let delete_args = ["site", "delete", "--request-stdin", "--plan-json", plan];
    bonesinfra::run_with_request(&delete_args, &request)
        .context("Site deletion stopped before completion; rerun the command to resume")?;

    let complete = infra::decommission_command("complete", &cfg.project_name);
    session.run_cmd(&complete).await?;
    let verify = infra::decommission_command("verify", &cfg.project_name);
    session.run_cmd(&verify).await?;
    session.close().await?;

    println!("{} Site deletion complete.", output::success_marker());
    Ok(())
}
