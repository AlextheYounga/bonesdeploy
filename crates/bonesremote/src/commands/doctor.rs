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
