use std::fs;
use std::process::Command;

use shared::paths;

pub(super) fn check_support(issues: &mut Vec<String>) {
    check_apparmor_kernel_enabled(issues);
    check_apparmor_service(issues);
}

fn check_apparmor_kernel_enabled(issues: &mut Vec<String>) {
    let enabled_file = fs::read_to_string(paths::APPARMOR_ENABLED_PARAM);
    let Ok(enabled_value) = enabled_file else {
        issues.push(format!("AppArmor check failed: could not read {}", paths::APPARMOR_ENABLED_PARAM));
        return;
    };

    if !matches!(enabled_value.trim().to_ascii_lowercase().as_str(), "y" | "yes" | "1") {
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
