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
            issues.push(format!("AppArmor check failed: could not run systemctl is-active apparmor ({error})"));
        }
    }
}

fn check_apparmor_profiles_enforcing(issues: &mut Vec<String>) {
    let profiles = fs::read_to_string("/sys/kernel/security/apparmor/profiles");
    match profiles {
        Ok(contents) => {
            if !kernel_profiles_have_enforce_mode(&contents) {
                issues.push("AppArmor check failed: no profiles appear to be in enforce mode".to_string());
            }
        }
        Err(error) => {
            issues.push(format!(
                "AppArmor check failed: could not read /sys/kernel/security/apparmor/profiles ({error})"
            ));
        }
    }
}

fn apparmor_kernel_enabled(contents: &str) -> bool {
    matches!(contents.trim().to_ascii_lowercase().as_str(), "y" | "yes" | "1")
}

fn kernel_profiles_have_enforce_mode(contents: &str) -> bool {
    contents.lines().any(|line| line.trim_end().ends_with(" (enforce)"))
}

#[cfg(test)]
mod tests {
    use super::{apparmor_kernel_enabled, kernel_profiles_have_enforce_mode};

    #[test]
    fn apparmor_kernel_enabled_accepts_yes() {
        assert!(apparmor_kernel_enabled("Y\n"));
    }

    #[test]
    fn apparmor_kernel_enabled_rejects_no() {
        assert!(!apparmor_kernel_enabled("N\n"));
    }

    #[test]
    fn kernel_profiles_detects_enforce_mode() {
        assert!(kernel_profiles_have_enforce_mode("bonesdeploy-demo-nginx (enforce)\n"));
    }

    #[test]
    fn kernel_profiles_rejects_non_enforce_mode() {
        assert!(!kernel_profiles_have_enforce_mode("bonesdeploy-demo-nginx (complain)\n"));
    }

    #[test]
    fn doctor_source_uses_kernel_profile_list_for_enforce_check() {
        let source = include_str!("doctor.rs");
        let production_source = source.split("#[cfg(test)]").next().unwrap_or(source);
        let apparmor_check =
            production_source.split("fn check_apparmor_profiles_enforcing").nth(1).unwrap_or(production_source);

        assert!(
            apparmor_check.contains("/sys/kernel/security/apparmor/profiles") && !apparmor_check.contains("aa-status"),
            "doctor should use only kernel apparmor profile list for enforce check\n{apparmor_check}"
        );
    }
}
