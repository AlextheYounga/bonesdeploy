use std::fs;
use std::process::Command;

use shared::paths;

pub(super) fn check_supported_distribution(issues: &mut Vec<String>) {
    let os_release = fs::read_to_string(paths::ETC_OS_RELEASE);
    let Ok(os_release) = os_release else {
        issues.push(format!("Failed to read {}; expected Debian or Ubuntu host", paths::ETC_OS_RELEASE));
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

pub(super) fn check_passwordless_sudo(issues: &mut Vec<String>) {
    let privileged_commands = [
        [paths::BONESREMOTE_BINARY, "hook", "post-receive", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "service", "restart", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "release", "rollback", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "release", "drop-failed", "--site", "nonexistent"],
        [paths::BONESREMOTE_BINARY, "release", "prune", "--site", "nonexistent"],
    ];

    let missing: Vec<String> = privileged_commands
        .into_iter()
        .filter(|command| !deploy_user_can_run(*command))
        .map(|command| command.join(" "))
        .collect();

    if !missing.is_empty() {
        issues.push(format!(
            "deploy user is missing passwordless sudo permissions for: {} (ensure bonesinfra has provisioned the sudoers policy on this host)",
            missing.join(", "),
        ));
    }
}

fn deploy_user_can_run(command: [&str; 5]) -> bool {
    match deploy_user_sudo_check_command(command).output() {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

fn deploy_user_sudo_check_command(command: [&str; 5]) -> Command {
    let mut sudo = Command::new("sudo");
    sudo.args(["-n", "-u", paths::DEPLOY_USER, "sudo", "-n", "-l"]).args(command);
    sudo
}
