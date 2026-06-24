use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use serde::Serialize;
use shared::config;

#[derive(Debug, Serialize)]
struct Report {
    current_release: String,
    ssl: SslStatus,
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Serialize)]
struct SslStatus {
    enabled: bool,
    domain: String,
}

#[derive(Clone, Debug, Serialize)]
struct ServiceStatus {
    name: String,
    kind: String,
    state: String,
    enabled: String,
}

pub fn run(config_path: &str) -> Result<()> {
    let report = build_report(config_path)?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn build_report(config_path: &str) -> Result<Report> {
    let config_path = Path::new(config_path);
    let cfg = config::load(config_path)?;
    let runtime = config::load_runtime(config_path.parent().unwrap_or_else(|| Path::new(".")))?;
    let deployment = cfg.deployment_paths(&runtime.web_root);

    Ok(Report {
        current_release: current_release(&deployment.current),
        ssl: ssl_status(&cfg, &deployment.nginx_site_available),
        services: services(&cfg.project_name),
    })
}

fn current_release(current_path: &str) -> String {
    fs::read_link(current_path).map_or_else(
        |_| String::from("unknown"),
        |path| path.file_name().map_or_else(|| String::from("unknown"), |name| name.to_string_lossy().to_string()),
    )
}

fn ssl_status(cfg: &config::Bones, nginx_config_path: &str) -> SslStatus {
    let enabled = !cfg.domain.is_empty()
        && fs::read_to_string(nginx_config_path).is_ok_and(|content| {
            content.contains(&format!("server_name {};", cfg.domain)) && content.contains("listen 443 ssl;")
        });

    SslStatus { enabled, domain: cfg.domain.clone() }
}

fn services(project_name: &str) -> Vec<ServiceStatus> {
    let expected = format!("{project_name}-nginx.service");
    let mut services = BTreeMap::from([(
        expected.clone(),
        ServiceStatus {
            name: expected,
            kind: String::from("site_nginx"),
            state: String::from("unknown"),
            enabled: String::from("unknown"),
        },
    )]);

    for name in discovered_service_names(project_name) {
        services.entry(name.clone()).or_insert_with(|| ServiceStatus {
            name,
            kind: String::from("discovered"),
            state: String::from("unknown"),
            enabled: String::from("unknown"),
        });
    }

    for service in services.values_mut() {
        service.state = systemctl_output(["is-active", service.name.as_str()]);
        service.enabled = systemctl_output(["is-enabled", service.name.as_str()]);
    }

    services.into_values().collect()
}

fn discovered_service_names(project_name: &str) -> Vec<String> {
    let pattern = format!("*{project_name}*.service");
    let mut names = BTreeSet::new();

    for args in [
        vec!["list-units", "--all", "--type=service", "--no-legend", "--no-pager", pattern.as_str()],
        vec!["list-unit-files", "--type=service", "--no-legend", "--no-pager", pattern.as_str()],
    ] {
        let output = Command::new("systemctl").args(args).output();
        let Ok(output) = output else { continue };
        if output.status.success() {
            names.extend(parse_systemctl_units(&String::from_utf8_lossy(&output.stdout)));
        }
    }

    names.into_iter().collect()
}

fn systemctl_output<const N: usize>(args: [&str; N]) -> String {
    Command::new("systemctl").args(args).output().map_or_else(
        |_| String::from("unknown"),
        |output| {
            let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if value.is_empty() { String::from("unknown") } else { value }
        },
    )
}

fn parse_systemctl_units(output: &str) -> Vec<String> {
    output
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| name.ends_with(".service"))
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_systemctl_units;

    #[test]
    fn parses_unit_names_from_systemctl_output() {
        let output = "atlas-nginx.service loaded active running Per-site nginx\natlas-worker.service loaded failed failed Worker\n";

        assert_eq!(parse_systemctl_units(output), vec!["atlas-nginx.service", "atlas-worker.service"]);
    }
}
