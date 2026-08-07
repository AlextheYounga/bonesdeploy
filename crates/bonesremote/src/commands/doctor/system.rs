use std::fs;
use std::process::Command;

use bonesdeploy_core::paths;

pub(super) fn check_supported_distribution(issues: &mut Vec<String>) {
    let os_release = fs::read_to_string(paths::etc_os_release());
    let Ok(os_release) = os_release else {
        issues.push(format!("Failed to read {}; expected Debian or Ubuntu host", paths::etc_os_release()));
        return;
    };

    let normalized = os_release.to_lowercase();
    if normalized.contains("id=debian") || normalized.contains("id=ubuntu") {
        return;
    }

    issues.push("Unsupported host OS; bonesremote currently supports Debian/Ubuntu only".to_string());
}

pub(super) fn check_podman_available(issues: &mut Vec<String>) {
    let result = Command::new("podman").arg("--version").output();

    match result {
        Ok(output) if output.status.success() => {}
        _ => issues.push("podman is not available; install Podman for disposable builds".to_string()),
    }
}
