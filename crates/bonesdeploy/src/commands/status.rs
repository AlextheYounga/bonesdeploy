use std::path::Path;

use anyhow::Result;
use serde::Deserialize;
use shared::config as shared_config;
use shared::paths;

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

    let remote = if let Some(cfg) = cfg {
        remote_status(cfg).await.unwrap_or_else(|_| fallback_remote_status(cfg))
    } else {
        empty_remote_status()
    };

    println!("Current release: {}", remote.current_release);
    println!("SSL: {}", ssl_state(&remote.ssl));
    println!();
    println!("Services:");
    for service in &remote.services {
        println!("{} {} {}/{}", service_marker(&service.state), service.name, service.state, service.enabled);
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

fn fallback_remote_status(cfg: &config::Bones) -> RemoteReport {
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR)).ok();
    let current_release = runtime.as_ref().map_or_else(
        || String::from("unknown"),
        |runtime| {
            let deployment = cfg.deployment_paths(&runtime.web_root);
            release_name(&deployment.current)
        },
    );

    RemoteReport {
        current_release,
        ssl: RemoteSslStatus { enabled: cfg.ssl_enabled, domain: cfg.domain.clone() },
        services: vec![RemoteServiceStatus {
            name: paths::nginx_service_name(&cfg.project_name),
            state: String::from("unknown"),
            enabled: String::from("unknown"),
        }],
    }
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
    if state == "active" { "✓" } else { "✗" }
}

fn release_name(value: &str) -> String {
    Path::new(value).file_name().map_or_else(|| value.to_string(), |name| name.to_string_lossy().to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::{RemoteSslStatus, service_marker, ssl_state};

    #[test]
    fn ssl_state_uses_remote_state_when_local_flag_is_stale() {
        let ssl = RemoteSslStatus { domain: String::from("app.example.com"), enabled: true };

        assert_eq!(ssl_state(&ssl), "enabled (app.example.com)");
    }

    #[test]
    fn service_marker_marks_only_active_as_ok() {
        assert_eq!(service_marker("active"), "✓");
        assert_eq!(service_marker("failed"), "✗");
        assert_eq!(service_marker("unknown"), "✗");
    }
}
