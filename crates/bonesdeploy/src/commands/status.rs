use anyhow::Result;
use console::style;
use serde::Deserialize;

use crate::commands::skill;
use crate::config;
use crate::infra::ssh;
use crate::ui::output;

#[derive(Debug, Deserialize)]
struct RemoteReport {
    current_release: String,
    ssl: RemoteSslStatus,
    services: Vec<RemoteServiceStatus>,
}

#[derive(Debug, Deserialize)]
struct RemoteSslStatus {
    enabled: bool,
    domain: String,
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

async fn remote_status(cfg: &config::Bones) -> Result<RemoteReport> {
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

fn service_marker(state: &str) -> String {
    match state {
        "active" => output::success_marker(),
        "unknown" => output::pending_marker(),
        _ => output::failure_marker(),
    }
}
