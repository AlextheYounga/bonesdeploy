use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use shared::paths;

pub(super) fn check_support(issues: &mut Vec<String>) {
    check_apparmor_kernel_enabled(issues);
    check_apparmor_service(issues);

    let Some(profile_names) = check_apparmor_profiles_installed(issues) else {
        return;
    };

    check_apparmor_unit_wiring(&profile_names, issues);
}

fn check_apparmor_kernel_enabled(issues: &mut Vec<String>) {
    let enabled_file = fs::read_to_string(paths::APPARMOR_ENABLED_PARAM);
    let Ok(enabled_value) = enabled_file else {
        issues.push(format!("AppArmor check failed: could not read {}", paths::APPARMOR_ENABLED_PARAM));
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

fn check_apparmor_profiles_installed(issues: &mut Vec<String>) -> Option<Vec<String>> {
    let profiles = fs::read_dir(paths::ETC_APPARMOR_D);
    let Ok(profiles) = profiles else {
        issues.push(format!("AppArmor check failed: could not read {}", paths::ETC_APPARMOR_D));
        return None;
    };

    let profile_files: Vec<String> = profiles
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| apparmor_profile_filename(name))
        .collect();

    if profile_files.is_empty() {
        issues.push(format!(
            "AppArmor check failed: no bonesdeploy AppArmor profile found under {}",
            paths::ETC_APPARMOR_D
        ));
        return None;
    }

    Some(profile_files)
}

fn check_apparmor_unit_wiring(profile_names: &[String], issues: &mut Vec<String>) {
    let units = fs::read_dir(paths::ETC_SYSTEMD_SYSTEM);
    let Ok(units) = units else {
        issues.push(format!("AppArmor check failed: could not read {}", paths::ETC_SYSTEMD_SYSTEM));
        return;
    };

    let installed_profiles: HashSet<&str> = profile_names.iter().map(String::as_str).collect();
    let expected_unit_names: Vec<String> =
        profile_names.iter().filter_map(|profile_name| apparmor_unit_name_for_profile(profile_name)).collect();

    if expected_unit_names.is_empty() {
        issues.push("AppArmor check failed: no bonesdeploy AppArmor profile matched an expected unit name".to_string());
        return;
    }

    let unit_entries: HashMap<String, PathBuf> = units
        .filter_map(Result::ok)
        .filter_map(|entry| entry.file_name().into_string().ok().map(|name| (name, entry.path())))
        .collect();

    for expected_unit_name in expected_unit_names {
        let Some(path) = unit_entries.get(&expected_unit_name) else {
            issues.push(format!(
                "AppArmor check failed: expected {}/{} for installed bonesdeploy profile",
                paths::ETC_SYSTEMD_SYSTEM,
                expected_unit_name
            ));
            continue;
        };

        let contents = fs::read_to_string(path);

        match contents {
            Ok(contents) => {
                if let Some(msg) = apparmor_unit_wiring_issue(&contents, &installed_profiles) {
                    issues.push(format!("AppArmor check failed: {path_display} {msg}", path_display = path.display()));
                }
            }
            Err(error) => {
                issues.push(format!("AppArmor check failed: could not read {} ({error})", path.display()));
            }
        }
    }
}

fn apparmor_kernel_enabled(contents: &str) -> bool {
    matches!(contents.trim().to_ascii_lowercase().as_str(), "y" | "yes" | "1")
}

fn apparmor_profile_filename(name: &str) -> bool {
    name.starts_with("bonesdeploy-") && name.ends_with("-nginx")
}

fn apparmor_unit_name_for_profile(profile_name: &str) -> Option<String> {
    profile_name
        .strip_prefix("bonesdeploy-")
        .and_then(|name| name.strip_suffix("-nginx"))
        .map(paths::nginx_service_name)
}

fn systemd_directive_values<'a>(contents: &'a str, directive: &str) -> Vec<&'a str> {
    contents
        .lines()
        .filter_map(|line| line.strip_prefix(directive))
        .filter_map(|value| value.strip_prefix('='))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .collect()
}

fn systemd_directive_contains_token(contents: &str, directive: &str, token: &str) -> bool {
    systemd_directive_values(contents, directive)
        .into_iter()
        .flat_map(|value| value.split_ascii_whitespace())
        .any(|value| value == token)
}

fn apparmor_profile_binding(contents: &str) -> Option<&str> {
    systemd_directive_values(contents, "AppArmorProfile").into_iter().next()
}

fn apparmor_unit_wiring_issue(contents: &str, installed_profiles: &HashSet<&str>) -> Option<String> {
    if !systemd_directive_contains_token(contents, "After", "apparmor.service")
        || !systemd_directive_contains_token(contents, "Requires", "apparmor.service")
    {
        return Some(String::from("is missing apparmor.service ordering or dependency"));
    }

    let Some(profile_name) = apparmor_profile_binding(contents) else {
        return Some(String::from("is missing AppArmorProfile="));
    };

    if installed_profiles.contains(profile_name) {
        None
    } else {
        Some(format!("references unknown AppArmor profile {profile_name}"))
    }
}

#[cfg(test)]
#[test]
fn apparmor_kernel_enabled_accepts_yes() {
    assert!(apparmor_kernel_enabled("Y\n"));
}

#[cfg(test)]
#[test]
fn apparmor_kernel_enabled_rejects_no() {
    assert!(!apparmor_kernel_enabled("N\n"));
}

#[cfg(test)]
#[test]
fn apparmor_profile_filename_accepts_bonesdeploy_profile() {
    assert!(apparmor_profile_filename("bonesdeploy-demo-nginx"));
}

#[cfg(test)]
#[test]
fn apparmor_profile_filename_rejects_unrelated_file() {
    assert!(!apparmor_profile_filename("default"));
}

#[cfg(test)]
#[test]
fn apparmor_unit_name_for_profile_maps_project_unit() {
    assert_eq!(apparmor_unit_name_for_profile("bonesdeploy-demo-nginx"), Some("demo-nginx.service".to_string()));
}

#[cfg(test)]
#[test]
fn apparmor_unit_wiring_accepts_expected_unit_with_reordered_after_tokens() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(apparmor_unit_wiring_issue(
        "[Unit]\nAfter=apparmor.service network.target\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-nginx\n",
        &installed_profiles,
    )
    .is_none());
}

#[cfg(test)]
#[test]
fn apparmor_unit_wiring_rejects_missing_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    assert!(
        apparmor_unit_wiring_issue(
            "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nType=simple\n",
            &installed_profiles,
        )
        .is_some()
    );
}

#[cfg(test)]
#[test]
fn apparmor_unit_wiring_rejects_unknown_profile_binding() {
    let installed_profiles = HashSet::from(["bonesdeploy-demo-nginx"]);

    let issue = apparmor_unit_wiring_issue(
        "[Unit]\nAfter=network.target apparmor.service\nRequires=apparmor.service\n[Service]\nAppArmorProfile=bonesdeploy-demo-ngnix\n",
        &installed_profiles,
    );
    assert!(issue.is_some_and(|msg| msg.contains("bonesdeploy-demo-ngnix")));
}
