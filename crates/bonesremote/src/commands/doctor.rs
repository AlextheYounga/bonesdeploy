use std::fs;
use std::process::Command;

use anyhow::Result;
use console::style;

use crate::config;
use crate::landlock;

pub fn run() -> Result<()> {
    println!("{}", style(format!("{} doctor", config::Constants::BINARY_NAME)).bold());

    let mut issues: Vec<String> = Vec::new();

    check_supported_distribution(&mut issues);
    check_globally_available(&mut issues);
    check_passwordless_sudo(&mut issues);
    check_apparmor_support(&mut issues);
    check_landlock_support(&mut issues);

    if issues.is_empty() {
        println!("\n{} All checks passed.", style("OK").green().bold());
        Ok(())
    } else {
        println!();
        for issue in &issues {
            println!("  {} {issue}", style("!").red().bold());
        }
        anyhow::bail!("Doctor found {} issue{}", issues.len(), if issues.len() == 1 { "" } else { "s" });
    }
}

fn check_supported_distribution(issues: &mut Vec<String>) {
    let os_release = fs::read_to_string("/etc/os-release");
    let Ok(os_release) = os_release else {
        issues.push("Failed to read /etc/os-release; expected Debian or Ubuntu host".to_string());
        return;
    };

    let normalized = os_release.to_lowercase();
    if normalized.contains("id=debian") || normalized.contains("id=ubuntu") {
        return;
    }

    issues.push("Unsupported host OS; bonesremote currently supports Debian/Ubuntu only".to_string());
}

fn check_globally_available(issues: &mut Vec<String>) {
    let result = Command::new(config::Constants::BINARY_NAME).arg("version").output();

    match result {
        Ok(output) if output.status.success() => {}
        _ => issues.push(format!("{} is not globally available (not in PATH)", config::Constants::BINARY_NAME)),
    }
}

fn check_passwordless_sudo(issues: &mut Vec<String>) {
    let privileged_commands = [
        [config::Constants::BINARY_NAME, "release", "stage", "--config", "/nonexistent"],
        [config::Constants::BINARY_NAME, "release", "wire", "--config", "/nonexistent"],
        [config::Constants::BINARY_NAME, "hooks", "post-deploy", "--config", "/nonexistent"],
    ];

    for command in privileged_commands {
        let result = Command::new("sudo").arg("-n").arg("-l").args(command).output();

        match result {
            Ok(output) if output.status.success() => {}
            _ => issues.push(format!(
                "{} is not allowed via passwordless sudo: {} (run 'sudo {} init')",
                config::Constants::BINARY_NAME,
                command.join(" "),
                config::Constants::BINARY_NAME
            )),
        }
    }
}

fn check_landlock_support(issues: &mut Vec<String>) {
    match landlock::verify_support() {
        Ok(()) => {}
        Err(error) => issues.push(format!("Landlock support check failed: {error}")),
    }
}

fn check_apparmor_support(issues: &mut Vec<String>) {
    check_apparmor_kernel_enabled(issues);
    check_apparmor_service(issues);
    check_apparmor_profiles_enforcing(issues);
}

fn check_apparmor_kernel_enabled(issues: &mut Vec<String>) {
    let enabled_file = fs::read_to_string("/sys/module/apparmor/parameters/enabled");
    let Ok(enabled_value) = enabled_file else {
        issues.push("AppArmor check failed: could not read /sys/module/apparmor/parameters/enabled".to_string());
        return;
    };

    if !apparmor_kernel_enabled(&enabled_value) {
        issues.push("AppArmor check failed: kernel module is not enabled".to_string());
    }
}

fn check_apparmor_service(issues: &mut Vec<String>) {
    let status = Command::new("systemctl").args(["is-active", "apparmor"]).output();
    match status {
        Ok(output) if output.status.success() && String::from_utf8_lossy(&output.stdout).trim() == "active" => {}
        Ok(output) => {
            let service_status = String::from_utf8_lossy(&output.stdout).trim().to_string();
            issues.push(format!(
                "AppArmor check failed: apparmor service is not active (status: {})",
                if service_status.is_empty() { "unknown" } else { service_status.as_str() }
            ));
        }
        Err(error) => {
            issues.push(format!("AppArmor check failed: could not run systemctl is-active apparmor ({error})"))
        }
    }
}

fn check_apparmor_profiles_enforcing(issues: &mut Vec<String>) {
    let result = Command::new("aa-status").output();
    match result {
        Ok(output) if output.status.success() => {
            let status_output = String::from_utf8_lossy(&output.stdout);
            if !aa_status_has_enforcing_profiles(&status_output) {
                issues.push("AppArmor check failed: no profiles appear to be in enforce mode".to_string());
            }
        }
        Ok(_) => issues.push("AppArmor check failed: aa-status returned non-zero status".to_string()),
        Err(error) => issues.push(format!("AppArmor check failed: could not run aa-status ({error})")),
    }
}

fn apparmor_kernel_enabled(contents: &str) -> bool {
    matches!(contents.trim().to_ascii_lowercase().as_str(), "y" | "yes" | "1")
}

fn aa_status_has_enforcing_profiles(contents: &str) -> bool {
    contents.lines().any(|line| {
        let trimmed = line.trim().to_ascii_lowercase();
        if let Some(count) = trimmed.split_whitespace().next().and_then(|token| token.parse::<u32>().ok()) {
            return count > 0
                && (trimmed.contains("profiles are in enforce mode")
                    || trimmed.contains("profile is in enforce mode"));
        }

        false
    })
}

#[cfg(test)]
mod tests {
    use super::{aa_status_has_enforcing_profiles, apparmor_kernel_enabled};

    #[test]
    fn apparmor_kernel_enabled_accepts_yes() {
        assert!(apparmor_kernel_enabled("Y\n"));
    }

    #[test]
    fn apparmor_kernel_enabled_rejects_no() {
        assert!(!apparmor_kernel_enabled("N\n"));
    }

    #[test]
    fn aa_status_detects_enforce_plural_line() {
        assert!(aa_status_has_enforcing_profiles("42 profiles are in enforce mode."));
    }

    #[test]
    fn aa_status_detects_enforce_singular_line() {
        assert!(aa_status_has_enforcing_profiles("1 profile is in enforce mode."));
    }

    #[test]
    fn aa_status_rejects_when_no_enforced_profiles_present() {
        assert!(!aa_status_has_enforcing_profiles("0 profiles are in enforce mode."));
    }
}
