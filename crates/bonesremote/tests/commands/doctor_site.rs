use std::fs;
use std::os::unix::fs::symlink;

use anyhow::Result;
use bonesdeploy_core::paths;
use bonesremote::commands::doctor::site::{classify_compose_runtime, compose_deployment_is_pending};
use bonesremote::runtime::docker::command::{ComposePublisher, ComposeServiceStatus, ComposeStackStatus};
use tempfile::tempdir;

#[test]
fn compose_doctor_treats_the_placeholder_current_release_as_pending() -> Result<()> {
    let root = tempdir()?;
    let placeholder = root.path().join(paths::RELEASES_DIR).join(paths::PLACEHOLDER_RELEASE_NAME);
    let deployed = root.path().join(paths::RELEASES_DIR).join("20260930_120000");
    fs::create_dir_all(&placeholder)?;
    fs::create_dir_all(&deployed)?;
    symlink(&placeholder, root.path().join(paths::CURRENT_LINK))?;

    assert!(compose_deployment_is_pending(root.path()));

    fs::remove_file(root.path().join(paths::CURRENT_LINK))?;
    symlink(&deployed, root.path().join(paths::CURRENT_LINK))?;
    assert!(!compose_deployment_is_pending(root.path()));
    Ok(())
}

#[test]
fn compose_runtime_findings_classify_health_failures_and_ingress() {
    let mut web = service("web", "running", None, 0);
    web.publishers.push(publisher("127.0.0.1", 8080));
    let mut worker = service("worker", "exited", None, 7);
    worker.publishers.push(publisher("0.0.0.0", 9000));
    let status = ComposeStackStatus {
        project_name: String::from("bonesdeploy-atlas"),
        files: vec![String::from("compose.yaml")],
        services: vec![
            web,
            service("healthy", "running", Some("healthy"), 0),
            service("migrate", "exited", None, 0),
            service("database", "running", Some("unhealthy"), 0),
            worker,
        ],
    };

    let findings = classify_compose_runtime(&status, Some(8080));

    assert_eq!(findings.issues.len(), 2);
    assert!(findings.issues.iter().any(|issue| issue == "Compose service database is unhealthy"));
    assert!(findings.issues.iter().any(|issue| issue.contains("Compose service worker failed")));
    assert_eq!(findings.warnings.len(), 2);
    assert!(findings.warnings.iter().any(|warning| warning.contains("web is running without a health check")));
    assert!(findings.warnings.iter().any(|warning| warning.contains("bypasses BonesDeploy-managed ingress")));
}

#[test]
fn compose_runtime_findings_report_missing_containers_and_loopback_ingress() {
    let empty = ComposeStackStatus {
        project_name: String::from("bonesdeploy-atlas"),
        files: vec![String::from("compose.yaml")],
        services: Vec::new(),
    };
    assert_eq!(classify_compose_runtime(&empty, None).issues, ["active Compose stack has no containers"]);

    let status = ComposeStackStatus {
        project_name: String::from("bonesdeploy-atlas"),
        files: vec![String::from("compose.yaml")],
        services: vec![service("web", "running", Some("healthy"), 0)],
    };
    assert_eq!(
        classify_compose_runtime(&status, Some(8080)).issues,
        ["Compose ingress port 8080 is not published on 127.0.0.1"]
    );
}

fn service(name: &str, state: &str, health: Option<&str>, exit_code: i64) -> ComposeServiceStatus {
    ComposeServiceStatus {
        name: format!("atlas-{name}-1"),
        service: name.to_owned(),
        state: state.to_owned(),
        health: health.map(str::to_owned),
        exit_code,
        publishers: Vec::new(),
    }
}

fn publisher(host_ip: &str, published_port: u16) -> ComposePublisher {
    ComposePublisher {
        host_ip: host_ip.to_owned(),
        target_port: published_port,
        published_port,
        protocol: String::from("tcp"),
    }
}
