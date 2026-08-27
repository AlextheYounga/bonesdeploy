use anyhow::Result;
use console::style;
use serde::Deserialize;

use crate::commands::skill;
use crate::config;
use crate::infra::ssh;
use crate::ui::output;

#[derive(Debug, Deserialize)]
pub(crate) struct RemoteReport {
    current_release: String,
    ssl: RemoteSslStatus,
    pub(crate) preview: Option<RemotePreviewStatus>,
    services: Vec<RemoteServiceStatus>,
}

#[derive(Debug, Deserialize)]
struct RemoteSslStatus {
    enabled: bool,
    domain: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct RemotePreviewStatus {
    pub(crate) active: bool,
    pub(crate) url: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RemoteServiceStatus {
    name: String,
    state: String,
    enabled: String,
}

pub async fn run() -> Result<()> {
    let report = skill::build_report().await?;
    let cfg = report.cfg.as_ref();

    println!("{} {}", style("Project").dim(), style(&report.project).bold());

    if let Some(cfg) = cfg {
        println!("{} {}", style("Host").dim(), cfg.host);
        println!("{} {}", style("Branch").dim(), cfg.branch);
    }

    println!("{} {}", style("State").dim(), report.state_label);

    if let Some(cfg) = cfg {
        match remote_status(cfg).await {
            Ok(remote) => {
                println!("{} {}", style("Release").dim(), style(&remote.current_release).bold());
                println!("{} {}", style("SSL").dim(), ssl_state(&remote.ssl));
                if let Some(preview) = render_preview_status(remote.preview.as_ref()) {
                    println!("{preview}");
                }
                if !remote.services.is_empty() {
                    println!();
                    println!("{}", style("Services").dim());
                    for service in &remote.services {
                        println!(
                            "  {} {}  {}/{}",
                            service_marker(&service.state),
                            service.name,
                            style(&service.state).dim(),
                            style(&service.enabled).dim(),
                        );
                    }
                }
            }
            Err(error) => {
                println!("{} Remote unavailable: {error:#}", output::failure_marker());
            }
        }
    }

    println!();
    println!("{}", output::next_step(&report.next.command));

    Ok(())
}

pub(crate) async fn remote_status(cfg: &config::Bones) -> Result<RemoteReport> {
    let session = ssh::connect_privileged(cfg).await?;
    let command = format!("bonesremote status --site {}", ssh::shell_quote(&cfg.project_name));
    let output = ssh::run_cmd(&session, &command).await;
    session.close().await?;

    Ok(serde_json::from_str(&output?)?)
}

fn ssl_state(ssl: &RemoteSslStatus) -> String {
    if ssl.enabled {
        if ssl.domain.is_empty() { String::from("enabled") } else { format!("enabled ({})", ssl.domain) }
    } else {
        String::from("disabled")
    }
}

pub(crate) fn render_preview_status(preview: Option<&RemotePreviewStatus>) -> Option<String> {
    let preview = preview.filter(|preview| preview.active)?;
    match preview.url.as_deref() {
        Some(url) => Some(format!("Preview: {url}")),
        None => Some(format!(
            "{} Quick Tunnel is starting; run `bonesdeploy status` for its URL.",
            output::pending_marker()
        )),
    }
}

fn service_marker(state: &str) -> String {
    match state {
        "active" => output::success_marker(),
        "unknown" => output::pending_marker(),
        _ => output::failure_marker(),
    }
}

#[cfg(test)]
mod tests {
    use super::{RemotePreviewStatus, render_preview_status};

    #[test]
    fn renders_active_preview_url() {
        let preview =
            RemotePreviewStatus { active: true, url: Some(String::from("https://preview.trycloudflare.com")) };

        assert_eq!(
            render_preview_status(Some(&preview)).as_deref(),
            Some("Preview: https://preview.trycloudflare.com")
        );
    }

    #[test]
    fn renders_starting_preview_without_url() {
        let preview = RemotePreviewStatus { active: true, url: None };

        assert!(render_preview_status(Some(&preview)).is_some_and(|line| line.contains("Quick Tunnel is starting")));
    }

    #[test]
    fn does_not_render_inactive_preview() {
        let preview = RemotePreviewStatus { active: false, url: None };

        assert!(render_preview_status(Some(&preview)).is_none());
    }
}
