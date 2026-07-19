use std::path::Path;

use anyhow::{Context, Result};
use console::style;
use serde::Deserialize;
use shared::paths;

use crate::config;
use crate::infra::ssh;
use crate::ui::output;

#[derive(Debug, Deserialize)]
struct Report {
    releases: Vec<Release>,
}

#[derive(Debug, Deserialize)]
struct Release {
    name: String,
    status: String,
    phase: Option<String>,
    started_at: Option<String>,
}

pub async fn list() -> Result<()> {
    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let report = remote_report(&cfg).await?;

    if report.releases.is_empty() {
        println!("No releases found.");
        return Ok(());
    }

    println!("{}  {}  {}", style("RELEASE").dim(), style("STATUS").dim(), style("STARTED").dim(),);
    for release in &report.releases {
        let (marker, status_str) = render_status(release);
        let name = if release.status == "active" {
            style(&release.name).bold().to_string()
        } else {
            style(&release.name).dim().to_string()
        };
        println!("{marker} {name:<24} {status_str:<14} {}", release.started_at.as_deref().unwrap_or("-"));
    }
    Ok(())
}

fn render_status(release: &Release) -> (String, String) {
    match release.status.as_str() {
        "active" => (output::success_marker(), style("active").green().to_string()),
        "building" | "preparing" => (output::pending_marker(), style(format_status(release)).yellow().to_string()),
        "interrupted" => (output::failure_marker(), style("interrupted").red().to_string()),
        "previous" => (style("·").dim().to_string(), style("previous").dim().to_string()),
        other => (output::pending_marker(), style(other).to_string()),
    }
}

fn format_status(release: &Release) -> String {
    match release.phase.as_deref() {
        Some(phase) if phase != release.status => format!("{}/{}", release.status, phase),
        _ => release.status.clone(),
    }
}

pub async fn kill(release: &str) -> Result<()> {
    let cfg = config::load(Path::new(paths::LOCAL_BONES_TOML))?;
    let session = ssh::connect_privileged(&cfg).await?;
    let command = format!(
        "bonesremote release kill --site {} --release {}",
        ssh::shell_quote(&cfg.project_name),
        ssh::shell_quote(release)
    );
    let result = ssh::stream_cmd(&session, &command).await;
    session.close().await?;
    result
}

async fn remote_report(cfg: &config::Bones) -> Result<Report> {
    let session = ssh::connect_privileged(cfg).await?;
    let command = format!("bonesremote release list --site {}", ssh::shell_quote(&cfg.project_name));
    let output = ssh::run_cmd(&session, &command).await;
    session.close().await?;
    serde_json::from_str(&output?).context("Failed to parse remote release report")
}

#[cfg(test)]
mod tests {
    use super::{Release, format_status};

    #[test]
    fn release_status_includes_phase_when_present() {
        let release = Release {
            name: String::from("20260715_225306"),
            status: String::from("building"),
            phase: Some(String::from("building")),
            started_at: None,
        };
        assert_eq!(format_status(&release), "building");
    }
}
