use anyhow::Result;
use serde::Deserialize;

use crate::commands::guide;
use crate::config;
use crate::infra::ssh;

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
    let report = guide::build_report().await?;
    let cfg = report.cfg.as_ref();

    println!("Project: {}", report.project);

    if let Some(cfg) = cfg {
        println!("Host: {}", cfg.host);
        println!("Branch: {}", cfg.branch);
    }

    println!("State: {}", report.state_label);

    let remote = match cfg {
        Some(cfg) => match remote_status(cfg).await {
            Ok(remote) => Some(remote),
            Err(error) => {
                println!("Remote: unavailable ({error:#})");
                None
            }
        },
        None => Some(empty_remote_status()),
    };

    if let Some(remote) = remote {
        println!("Current release: {}", remote.current_release);
        println!("SSL: {}", ssl_state(&remote.ssl));
        println!();
        println!("Services:");
        for service in &remote.services {
            println!("{} {} {}/{}", service_marker(&service.state), service.name, service.state, service.enabled);
        }
    }
    println!();
    println!("Next: {}", report.commands[0]);

    Ok(())
}

async fn remote_status(cfg: &config::Bones) -> Result<RemoteReport> {
    let session = ssh::connect_privileged(cfg).await?;
    let command = format!("bonesremote status --site '{}'", shell_quote(&cfg.project_name));
    let output = ssh::run_cmd(&session, &command).await;
    session.close().await?;

    Ok(serde_json::from_str(&output?)?)
}

fn empty_remote_status() -> RemoteReport {
    RemoteReport {
        current_release: String::from("unknown"),
        ssl: RemoteSslStatus { enabled: false, domain: String::new() },
        services: Vec::new(),
    }
}

fn ssl_state(ssl: &RemoteSslStatus) -> String {
    if ssl.enabled {
        if ssl.domain.is_empty() { String::from("enabled") } else { format!("enabled ({})", ssl.domain) }
    } else {
        String::from("disabled")
    }
}

fn service_marker(state: &str) -> &'static str {
    match state {
        "active" => "✓",
        "unknown" => "?",
        _ => "✗",
    }
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
