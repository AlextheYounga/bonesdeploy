use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use anyhow::Result;
use bonesdeploy_core::config::validate_site_name;
use bonesdeploy_core::paths;
use serde::Serialize;

use crate::commands::inspection::systemd;
use crate::release::state as release_state;

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
    validate_site_name(site)?;
    let report = build_report(site);
    println!("{}", serde_json::to_string(&report)?);
    Ok(())
}

fn build_report(site: &str) -> Report {
    let project_root = paths::default_project_root_for(site);
    let nginx_site_available =
        Path::new(paths::ETC_NGINX_SITES_AVAILABLE).join(format!("{site}.conf")).display().to_string();

    Report {
        current_release: release_state::current_release_name(&project_root).unwrap_or_else(|_| String::from("unknown")),
        ssl: ssl_status(&nginx_site_available),
        services: services(site),
    }
}

fn ssl_status(nginx_config_path: &str) -> SslStatus {
    let content = fs::read_to_string(nginx_config_path).unwrap_or_default();
    let domain = content
        .lines()
        .find_map(|line| line.trim().strip_prefix("server_name "))
        .and_then(|value| value.strip_suffix(';'))
        .unwrap_or_default()
        .to_string();
    let enabled = !domain.is_empty() && content.contains("listen 443 ssl;");

    SslStatus { enabled, domain }
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
        service.state = systemd::unit_state(service.name.as_str(), "is-active");
        service.enabled = systemd::unit_state(service.name.as_str(), "is-enabled");
    }

    services.into_values().collect()
}

fn target_service_names(target: &str) -> Vec<String> {
    systemd::required_services(target).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::{env, fs, process};

    use anyhow::Result;

    use super::ssl_status;

    #[test]
    fn reads_ssl_domain_from_conventionally_named_nginx_config() -> Result<()> {
        let path = env::temp_dir().join(format!("bonesremote-status-{}.conf", process::id()));
        fs::write(&path, "server_name example.test;\nlisten 443 ssl;\n")?;
        let path = path.display().to_string();

        assert_eq!(ssl_status(&path).domain, "example.test");
        assert!(ssl_status(&path).enabled);
        fs::remove_file(path)?;
        Ok(())
    }
}
