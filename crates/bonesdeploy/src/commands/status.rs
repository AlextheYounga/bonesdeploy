use std::path::Path;

use anyhow::Result;
use shared::config as shared_config;
use shared::paths;

use crate::commands::guide;
use crate::config;
use crate::infra::ssh;

pub async fn run() -> Result<()> {
    let report = guide::build_report().await?;
    let cfg = report.cfg.as_ref();

    println!("Project: {}", report.project);

    if let Some(cfg) = cfg {
        println!("Host: {}", cfg.host);
        println!("Branch: {}", cfg.branch);
    }

    println!("State: {}", report.state_label);

    let (current_release, service_state, ssl_state) = if let Some(cfg) = cfg {
        remote_status(cfg).await.unwrap_or((String::from("unknown"), String::from("unknown"), ssl_state(cfg)))
    } else {
        (String::from("unknown"), String::from("unknown"), String::from("unknown"))
    };

    println!("Current release: {current_release}");
    println!("Service: {service_state}");
    println!("SSL: {ssl_state}");
    println!();
    println!("Next: {}", report.commands[0]);

    Ok(())
}

async fn remote_status(cfg: &config::Bones) -> Result<(String, String, String)> {
    let session = ssh::connect(cfg).await?;
    let runtime = shared_config::load_runtime(Path::new(paths::LOCAL_BONES_DIR))?;
    let deployment = cfg.deployment_paths(&runtime.web_root);

    let release = ssh::run_cmd(&session, &format!("readlink -f {}", shell_quote(&deployment.current))).await.ok();
    let current_release = release.map_or_else(|| String::from("unknown"), |value| release_name(value.trim()));

    let service =
        ssh::run_cmd(&session, &format!("systemctl is-active {}", shell_quote(&deployment.systemd_site_nginx_service)))
            .await
            .map_or_else(|_| String::from("unknown"), |value| value.trim().to_string());

    session.close().await?;

    Ok((current_release, service, ssl_state(cfg)))
}

fn ssl_state(cfg: &config::Bones) -> String {
    if cfg.ssl_enabled {
        if cfg.domain.is_empty() { String::from("enabled") } else { format!("enabled ({})", cfg.domain) }
    } else {
        String::from("disabled")
    }
}

fn release_name(value: &str) -> String {
    Path::new(value).file_name().map_or_else(|| value.to_string(), |name| name.to_string_lossy().to_string())
}

fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
