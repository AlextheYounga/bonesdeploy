use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::paths;

use crate::ui::{output, prompts};
use crate::{config, infra};

pub async fn start(yes: bool) -> Result<()> {
    super::readiness::ensure_project_ready()?;
    if !yes && !prompts::confirm_site_tunnel_start()? {
        println!("Skipped Quick Tunnel startup.");
        return Ok(());
    }

    println!("Starting Quick Tunnel...");
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    bonesinfra::run_with_request(&["tunnel", "start", "--request-stdin"], &request)?;
    print_status(&cfg).await;
    Ok(())
}

pub fn stop(yes: bool) -> Result<()> {
    super::readiness::ensure_project_ready()?;
    if !yes && !prompts::confirm_site_tunnel_stop()? {
        println!("Kept Quick Tunnel running.");
        return Ok(());
    }

    println!("Stopping Quick Tunnel...");
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    let request = infra::provisioning_request(&cfg)?;
    bonesinfra::run_with_request(&["tunnel", "stop", "--request-stdin"], &request)?;
    println!("{} Quick Tunnel stopped and removed.", output::success_marker());
    Ok(())
}

pub async fn status() -> Result<()> {
    super::readiness::ensure_project_ready()?;
    let cfg = config::load(Path::new(paths::DOT_ENV))?;
    print_status(&cfg).await;
    Ok(())
}

async fn print_status(cfg: &config::Bones) {
    match super::status::remote_status(cfg).await {
        Ok(report) => match report.preview {
            Some(preview) if preview.active => match preview.url {
                Some(url) => println!("Quick Tunnel: {url}"),
                None => println!(
                    "{} Quick Tunnel is starting; run `bonesdeploy site tunnel status` again for its URL.",
                    output::pending_marker()
                ),
            },
            _ => println!("Quick Tunnel: stopped"),
        },
        Err(error) => println!("{} Quick Tunnel status unavailable: {error:#}", output::pending_marker()),
    }
}
