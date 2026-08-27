use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use bonesdeploy_core::config::validate_site_name;
use bonesdeploy_core::paths;
use serde::Serialize;

use crate::inspection::systemd;
use crate::release::state as release_state;

#[derive(Debug, Serialize)]
struct Report {
    current_release: String,
    ssl: SslStatus,
    preview: Option<PreviewStatus>,
    services: Vec<ServiceStatus>,
}

#[derive(Debug, Serialize)]
pub struct SslStatus {
    pub enabled: bool,
    pub domain: String,
}

#[derive(Debug, Serialize)]
struct PreviewStatus {
    active: bool,
    url: Option<String>,
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
        preview: preview_status(site),
        services: services(site),
    }
}

fn preview_status(site: &str) -> Option<PreviewStatus> {
    let service = format!("{site}-cloudflared.service");
    if systemd::unit_state(&service, "is-active") != "active" {
        return None;
    }
    let journal = Command::new("journalctl")
        .args(["--boot", "0", "--unit", &service, "--no-pager", "--output", "cat"])
        .output()
        .ok()?;
    if !journal.status.success() {
        return None;
    }
    let output = String::from_utf8(journal.stdout).ok()?;
    Some(PreviewStatus { active: true, url: extract_preview_url(&output) })
}

pub(crate) fn extract_preview_url(journal: &str) -> Option<String> {
    journal
        .split_whitespace()
        .filter_map(|token| token.strip_prefix("https://"))
        .filter_map(|host| host.strip_suffix(".trycloudflare.com"))
        .filter(|host| {
            !host.is_empty()
                && !host.starts_with('-')
                && !host.ends_with('-')
                && host
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-')
        })
        .map(|host| format!("https://{host}.trycloudflare.com"))
        .next_back()
}

pub fn ssl_status(nginx_config_path: &str) -> SslStatus {
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
    use super::extract_preview_url;

    #[test]
    fn extracts_the_newest_valid_preview_url() {
        let journal = "old https://old.trycloudflare.com\nnoise https://new.trycloudflare.com";

        assert_eq!(extract_preview_url(journal).as_deref(), Some("https://new.trycloudflare.com"));
    }

    #[test]
    fn ignores_non_preview_urls_and_invalid_hosts() {
        let journal = "https://example.com https://bad_host.trycloudflare.com https://valid-preview.trycloudflare.com";

        assert_eq!(extract_preview_url(journal).as_deref(), Some("https://valid-preview.trycloudflare.com"));
    }

    #[test]
    fn returns_none_without_a_valid_preview_url() {
        assert_eq!(extract_preview_url("cloudflared started"), None);
    }
}
