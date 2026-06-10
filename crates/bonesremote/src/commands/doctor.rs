use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use console::style;
use shared::paths;

use crate::config;

pub fn run() -> Result<()> {
    println!("{}", style(format!("{} doctor", config::Constants::BINARY_NAME)).bold());

    let mut issues: Vec<String> = Vec::new();

    check_supported_distribution(&mut issues);
    check_globally_available(&mut issues);
    check_passwordless_sudo(&mut issues);
    check_apparmor_support(&mut issues);
    check_algif_aead_disabled(&mut issues);
    check_per_site_nginx_config(&mut issues);

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

fn check_apparmor_support(issues: &mut Vec<String>) {
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
            Ok(contents) => match apparmor_unit_wiring_status(&contents, &installed_profiles) {
                AppArmorUnitWiringStatus::Ok => {}
                AppArmorUnitWiringStatus::MissingOrdering => {
                    issues.push(format!(
                        "AppArmor check failed: {} is missing apparmor.service ordering or dependency",
                        path.display()
                    ));
                }
                AppArmorUnitWiringStatus::MissingProfile => {
                    issues.push(format!("AppArmor check failed: {} is missing AppArmorProfile=", path.display()));
                }
                AppArmorUnitWiringStatus::UnknownProfile(profile_name) => {
                    issues.push(format!(
                        "AppArmor check failed: {} references unknown AppArmor profile {}",
                        path.display(),
                        profile_name
                    ));
                }
            },
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
        .map(|project_name| format!("{project_name}-nginx.service"))
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

enum AppArmorUnitWiringStatus {
    Ok,
    MissingOrdering,
    MissingProfile,
    UnknownProfile(String),
}

fn apparmor_unit_wiring_status(contents: &str, installed_profiles: &HashSet<&str>) -> AppArmorUnitWiringStatus {
    let has_apparmor_after = systemd_directive_contains_token(contents, "After", "apparmor.service");
    let has_apparmor_requires = systemd_directive_contains_token(contents, "Requires", "apparmor.service");

    if !has_apparmor_after || !has_apparmor_requires {
        return AppArmorUnitWiringStatus::MissingOrdering;
    }

    let Some(profile_name) = apparmor_profile_binding(contents) else {
        return AppArmorUnitWiringStatus::MissingProfile;
    };

    if installed_profiles.contains(profile_name) {
        AppArmorUnitWiringStatus::Ok
    } else {
        AppArmorUnitWiringStatus::UnknownProfile(profile_name.to_string())
    }
}

fn check_algif_aead_disabled(issues: &mut Vec<String>) {
    let modules = fs::read_to_string(paths::PROC_MODULES);
    let Ok(modules) = modules else {
        issues.push(format!("Kernel module check failed: could not read {}", paths::PROC_MODULES));
        return;
    };

    if algif_aead_is_loaded(&modules) {
        issues.push(
            "Kernel module check failed: algif_aead is loaded; disable it to mitigate CVE-2026-31431 \
             (run: echo 'install algif_aead /bin/false' > /etc/modprobe.d/disable-algif.conf && rmmod algif_aead)"
                .to_string(),
        );
        return;
    }

    if !algif_aead_is_blocked() {
        issues.push(
            "Kernel module check failed: algif_aead is not blocked from loading; block it to mitigate CVE-2026-31431 \
             (run: echo 'install algif_aead /bin/false' > /etc/modprobe.d/disable-algif.conf)"
                .to_string(),
        );
    }
}

fn algif_aead_is_loaded(proc_modules: &str) -> bool {
    proc_modules.lines().any(|line| line.split_whitespace().next() == Some("algif_aead"))
}

fn algif_aead_is_blocked() -> bool {
    let Ok(entries) = fs::read_dir(paths::ETC_MODPROBE_D) else {
        return false;
    };

    entries.filter_map(Result::ok).any(|entry| {
        let Ok(contents) = fs::read_to_string(entry.path()) else { return false };
        contents.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with('#') && trimmed.starts_with("install algif_aead")
        })
    })
}

fn check_per_site_nginx_config(issues: &mut Vec<String>) {
    let Ok(units) = fs::read_dir(paths::ETC_SYSTEMD_SYSTEM) else {
        return;
    };

    let site_nginx_units: Vec<(String, PathBuf)> = units
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().into_string().ok()?;
            if name.ends_with("-nginx.service") { Some((name, entry.path())) } else { None }
        })
        .collect();

    for (unit_name, _unit_path) in site_nginx_units {
        let Some(project_name) = unit_name.strip_suffix("-nginx.service") else {
            continue;
        };

        let nginx_conf_path = PathBuf::from(paths::DEFAULT_CONF_ROOT_PARENT).join(project_name).join("nginx.conf");

        if !nginx_conf_path.exists() {
            issues.push(format!(
                "Per-site nginx config check failed: {} does not exist (re-run remote setup with --tags nginx)",
                nginx_conf_path.display()
            ));
        }
    }
}

#[cfg(test)]
#[path = "doctor_tests.rs"]
mod tests;
