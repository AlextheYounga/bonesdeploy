use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::Result;
use console::style;

use crate::config;
use crate::landlock;

pub fn run(config_path: Option<&str>) -> Result<()> {
    println!("{}", style(format!("{} doctor", config::Constants::BINARY_NAME)).bold());

    let mut issues: Vec<String> = Vec::new();

    check_supported_distribution(&mut issues);
    check_globally_available(&mut issues);
    check_passwordless_sudo(&mut issues);
    check_landlock_support(&mut issues);

    if let Some(path) = config_path {
        check_runtime_readiness(path, &mut issues);
    }

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

fn check_runtime_readiness(config_path: &str, issues: &mut Vec<String>) {
    let path = Path::new(config_path);
    let cfg = match config::load(path) {
        Ok(cfg) => cfg,
        Err(error) => {
            issues.push(format!("Failed to load config {config_path}: {error}"));
            return;
        }
    };

    if cfg.runtime.command.is_empty() {
        issues.push("runtime.command is empty in bones.toml".to_string());
    }

    let service_user = &cfg.permissions.defaults.service_user;
    let user_lookup = Command::new("id").arg("-u").arg(service_user).output();
    match user_lookup {
        Ok(output) if output.status.success() => {}
        _ => issues.push(format!("service user does not exist: {service_user}")),
    }

    match fs::canonicalize(&cfg.data.live_root) {
        Ok(runtime_tree) => {
            if !runtime_tree.exists() {
                issues.push(format!("Resolved runtime tree does not exist: {}", runtime_tree.display()));
            }
        }
        Err(error) => {
            issues.push(format!("Failed to resolve runtime tree from live_root {}: {error}", cfg.data.live_root));
        }
    }

    let service_unit = format!("/etc/systemd/system/{}.service", cfg.data.project_name);
    if !Path::new(&service_unit).exists() {
        issues.push(format!("Systemd service unit is missing: {service_unit}"));
    }
}
