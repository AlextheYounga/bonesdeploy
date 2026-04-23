use std::process::Command;

use anyhow::Result;
use console::style;

use crate::config;

pub fn run() -> Result<()> {
    println!("{}", style(format!("{} doctor", config::Constants::BINARY_NAME)).bold());

    let mut issues: Vec<String> = Vec::new();

    check_globally_available(&mut issues);
    check_passwordless_sudo(&mut issues);

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

fn check_globally_available(issues: &mut Vec<String>) {
    let result = Command::new(config::Constants::BINARY_NAME).arg("version").output();

    match result {
        Ok(output) if output.status.success() => {}
        _ => issues.push(format!("{} is not globally available (not in PATH)", config::Constants::BINARY_NAME)),
    }
}

fn check_passwordless_sudo(issues: &mut Vec<String>) {
    let result = Command::new("sudo").args(["-n", config::Constants::BINARY_NAME, "version"]).output();

    match result {
        Ok(output) if output.status.success() => {}
        _ => issues.push(format!(
            "{} cannot run via sudo without a password \
                 (run 'sudo {} init')",
            config::Constants::BINARY_NAME,
            config::Constants::BINARY_NAME
        )),
    }
}
