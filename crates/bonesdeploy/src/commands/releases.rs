use std::path::Path;

use anyhow::{Context, Result};
use serde::Deserialize;
use shared::paths;

use crate::config;
use crate::infra::ssh;

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

    println!("RELEASE           STATUS       STARTED");
    for release in report.releases {
        let status = format_status(&release);
        println!("{:<17} {:<12} {}", release.name, status, release.started_at.unwrap_or_else(|| String::from("-")));
    }
    Ok(())
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
