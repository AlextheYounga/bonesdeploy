use std::fs;
use std::path::Path;
use std::process::Command;

use bonesdeploy_core::config::RemoteDeploymentConfig;
use bonesdeploy_core::{config, paths};

use crate::control_plane;
use crate::inspection::{accounts, systemd};
use crate::runtime::docker;
use crate::runtime::docker::command::ComposeStackStatus;

use super::services;

struct ComposeFindings<'a> {
    issues: &'a mut Vec<String>,
    pending: &'a mut Vec<String>,
    warnings: &'a mut Vec<String>,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct ComposeRuntimeFindings {
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
}

pub fn check(site: &str, issues: &mut Vec<String>, pending: &mut Vec<String>, warnings: &mut Vec<String>) {
    if let Err(error) = config::validate_site_name(site) {
        issues.push(format!("Invalid site name for doctor: {error}"));
        return;
    }

    // Without the synchronized descriptor, runtime checks would be fabricated from the site name.
    let descriptor = match control_plane::load(site) {
        Ok(descriptor) => descriptor,
        Err(error) => {
            pending.push(error.to_string());
            return;
        }
    };

    if docker_checks_required(&descriptor) {
        warnings.push(String::from(
            "Compose uses trusted privileged project input and does not provide native runtime guarantees for images, users, mounts, capabilities, namespaces, networks, or published ports; BonesDeploy does not add a Docker socket mount.",
        ));
    }

    if !paths::bonesremote_site_root(site).is_dir() {
        pending.push(format!("first deployment is pending for {site}"));
        return;
    }

    let project_root = paths::default_project_root_for(site);
    let shared_root = Path::new(&project_root).join(paths::SHARED_DIR);
    let releases_root = Path::new(&project_root).join(paths::RELEASES_DIR);
    let runtime_user = config::runtime_user_for(site);
    let runtime_group = config::runtime_group_for(site);

    let shared_env = shared_root.join(paths::DOT_ENV);
    if !shared_env.is_file() {
        pending.push(format!(
            "shared environment is missing: {}. Run 'bonesdeploy secrets push' first.",
            shared_env.display()
        ));
    }

    match fs::read_to_string(paths::ETC_PASSWD) {
        Ok(passwd) => {
            check_runtime_identity(&runtime_user, &runtime_group, &passwd, issues);
        }
        Err(error) => {
            issues.push(format!("could not read {} to validate user accounts ({error})", paths::ETC_PASSWD));
        }
    }

    check_site_layout(&shared_root, &releases_root, issues);

    services::check_target(site, issues);

    if docker_checks_required(&descriptor) {
        let mut findings = ComposeFindings { issues, pending, warnings };
        check_docker_runtime(site, Path::new(&project_root), descriptor.runtime.compose_port(), &mut findings);
    }
}

#[must_use]
pub fn docker_checks_required(descriptor: &RemoteDeploymentConfig) -> bool {
    descriptor.runtime.backend() == config::RuntimeBackend::Docker
}

fn check_docker_runtime(
    site: &str,
    project_root: &Path,
    compose_port: Option<u16>,
    findings: &mut ComposeFindings<'_>,
) {
    match Command::new("docker").arg("info").output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            findings
                .issues
                .push(format!("Docker daemon is unavailable: {}", String::from_utf8_lossy(&output.stderr).trim()));
        }
        Err(error) => findings.issues.push(format!("Docker is unavailable: {error}")),
    }

    match Command::new("docker").args(["compose", "version"]).output() {
        Ok(output) if output.status.success() => {}
        Ok(output) => findings
            .issues
            .push(format!("Docker Compose plugin is unavailable: {}", String::from_utf8_lossy(&output.stderr).trim())),
        Err(error) => findings.issues.push(format!("Docker Compose plugin is unavailable: {error}")),
    }

    if compose_deployment_is_pending(project_root) {
        findings.pending.push(format!("first Compose deployment is pending for {site}"));
        return;
    }
    if let Err(error) = docker::command::validate_active_configuration(site, project_root) {
        findings.issues.push(format!("active Compose configuration is invalid: {error:#}"));
        return;
    }

    let compose_unit = format!("{site}-compose.service");
    let target = paths::site_target_name(site);
    match systemd::required_services(&target) {
        Ok(services) if services.iter().any(|service| service == &compose_unit) => {}
        Ok(_) => findings.issues.push(format!("Compose service is not registered in {target}: {compose_unit}")),
        Err(error) => {
            findings.issues.push(format!("could not inspect Compose service registration in {target} ({error})"));
        }
    }

    let status = match docker::command::active_status(site, project_root) {
        Ok(status) => status,
        Err(error) => {
            findings.issues.push(format!("could not inspect active Compose stack: {error:#}"));
            return;
        }
    };
    let runtime_findings = classify_compose_runtime(&status, compose_port);
    findings.issues.extend(runtime_findings.issues);
    findings.warnings.extend(runtime_findings.warnings);
}

#[must_use]
pub fn compose_deployment_is_pending(project_root: &Path) -> bool {
    let current = project_root.join(paths::CURRENT_LINK);
    !current.exists() || services::current_is_placeholder(&project_root.to_string_lossy())
}

#[must_use]
pub fn classify_compose_runtime(status: &ComposeStackStatus, compose_port: Option<u16>) -> ComposeRuntimeFindings {
    let mut findings = ComposeRuntimeFindings::default();
    if status.services.is_empty() {
        findings.issues.push(String::from("active Compose stack has no containers"));
        return findings;
    }

    for service in &status.services {
        let name = if service.service.is_empty() { &service.name } else { &service.service };
        match service.condition() {
            "healthy" | "completed" => {}
            "running" => findings.warnings.push(format!(
                "Compose service {name} is running without a health check; application readiness is not proven"
            )),
            "starting" => findings.issues.push(format!("Compose service {name} has not become healthy")),
            "unhealthy" => findings.issues.push(format!("Compose service {name} is unhealthy")),
            _ => findings.issues.push(format!(
                "Compose service {name} failed (state {}, exit code {})",
                service.state, service.exit_code
            )),
        }
    }

    if let Some(port) = compose_port
        && !status.services.iter().any(|service| service.publishes_loopback_port(port))
    {
        findings.issues.push(format!("Compose ingress port {port} is not published on 127.0.0.1"));
    }
    if status.services.iter().any(docker::command::ComposeServiceStatus::publishes_public_port) {
        findings.warnings.push(String::from(
            "Compose publishes one or more ports outside loopback; that traffic bypasses BonesDeploy-managed ingress",
        ));
    }
    findings
}

fn check_runtime_identity(runtime_user: &str, runtime_group: &str, passwd: &str, issues: &mut Vec<String>) {
    if !accounts::account_exists(passwd, runtime_user) {
        issues.push(format!("runtime user does not exist: {runtime_user}"));
    }

    let groupfile = match fs::read_to_string(paths::ETC_GROUP) {
        Ok(groupfile) => groupfile,
        Err(error) => {
            issues.push(format!("could not read {} to validate runtime group ({error})", paths::ETC_GROUP));
            return;
        }
    };
    let Some(members) = accounts::group_members(&groupfile, runtime_group) else {
        issues.push(format!("runtime group does not exist: {runtime_group}"));
        return;
    };
    if members.iter().any(|member| member == paths::DEPLOY_USER) {
        issues.push(format!("{} must not be a member of runtime group {}", paths::DEPLOY_USER, runtime_group));
    }
}

fn check_site_layout(shared_root: &Path, releases_root: &Path, issues: &mut Vec<String>) {
    if !shared_root.is_dir() {
        issues.push(format!("shared root is missing: {}", shared_root.display()));
    }

    if !releases_root.is_dir() {
        issues.push(format!("releases root is missing: {}", releases_root.display()));
    }
}
