use std::{env, fs, process, process::Command};

use bonesdeploy_core::config::RuntimeBackend;
use bonesremote::commands::doctor::podman_check_required;
use bonesremote::commands::doctor::site::{check_branch_ref, classify_compose_runtime};
use bonesremote::runtime::docker::command::{ComposePublisher, ComposeServiceStatus, ComposeStackStatus};

#[test]
fn empty_bare_repo_is_pending_before_first_push() {
    let root = env::temp_dir().join(format!("bonesremote-doctor-empty-repo-{}", process::id()));
    let _ = fs::remove_dir_all(&root);
    let output = Command::new("git").args(["init", "--bare", root.to_str().unwrap_or_default()]).output();
    assert!(output.is_ok_and(|output| output.status.success()));

    let mut issues = Vec::new();
    let mut pending = Vec::new();
    check_branch_ref(root.to_str().unwrap_or_default(), &mut issues, &mut pending);

    let _ = fs::remove_dir_all(root);
    assert!(issues.is_empty());
    assert_eq!(pending.len(), 1);
}

#[test]
fn compose_sites_skip_the_native_podman_prerequisite() {
    assert!(!podman_check_required(Some(RuntimeBackend::Docker)));
    assert!(podman_check_required(Some(RuntimeBackend::Native)));
    assert!(podman_check_required(None));
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
