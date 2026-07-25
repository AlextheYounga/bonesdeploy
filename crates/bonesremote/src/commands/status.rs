use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use serde::Serialize;
use shared::config;
use shared::paths;

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

pub fn run(site: &str) -> Result<()> {
    let report = build_report(site)?;
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn build_report(site: &str) -> Result<Report> {
    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path).context("Failed to load remote bones.toml")?;
    let current = Path::new(&cfg.project_root).join(paths::CURRENT_LINK);
    let nginx_site_available =
        Path::new(paths::ETC_NGINX_SITES_AVAILABLE).join(format!("{}.conf", cfg.project_name)).display().to_string();

    Ok(Report {
        current_release: current_release(&current),
        ssl: ssl_status(&cfg, &nginx_site_available),
        services: services(&cfg.project_name),
    })
}

fn current_release(current_path: &Path) -> String {
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
    let target = paths::site_target_name(project_name);
    let mut services = BTreeMap::from([(
        target.clone(),
        ServiceStatus {
            name: target.clone(),
            kind: String::from("site_target"),
            state: String::from("unknown"),
            enabled: String::from("unknown"),
        },
    )]);

    for name in target_service_names(&target) {
        services.entry(name.clone()).or_insert_with(|| ServiceStatus {
            name,
            kind: String::from("registered"),
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

fn target_service_names(target: &str) -> Vec<String> {
    let output =
        Command::new("systemctl").args(["show", "--property=Requires", "--value", "--no-pager", "--", target]).output();
    match output {
        Ok(output) if output.status.success() => parse_target_units(&String::from_utf8_lossy(&output.stdout)),
        _ => Vec::new(),
    }
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

fn parse_target_units(output: &str) -> Vec<String> {
    output
        .split_whitespace()
        .filter(|name| name.ends_with(paths::SYSTEMD_SERVICE_SUFFIX))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::parse_target_units;

    #[test]
    fn parses_registered_units_from_target_properties() {
        let output = "atlas-nginx.service atlas-worker.service\natlas-worker.service";

        assert_eq!(parse_target_units(output), vec!["atlas-nginx.service", "atlas-worker.service"]);
    }
}
