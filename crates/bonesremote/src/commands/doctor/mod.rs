use anyhow::Result;
use bonesdeploy_core::paths;

use crate::privileges;
use crate::ui;

pub mod apparmor;
pub mod baseline;
pub mod security;
pub mod services;
pub mod site;
pub mod system;

pub fn run(site: Option<&str>, exhaustive: bool) -> Result<()> {
    privileges::ensure_root("bonesremote doctor")?;
    println!("{} doctor", console::style(paths::BONESREMOTE_BINARY).bold());

    let mut issues: Vec<String> = Vec::new();
    let mut pending: Vec<String> = Vec::new();

    system::check_supported_distribution(&mut issues);
    system::check_podman_available(&mut issues);
    apparmor::check_support(&mut issues);
    if site.is_none() {
        baseline::check(&mut issues);
    }

    if exhaustive {
        println!("  {} Exhaustively scanning the active release for permission drift.", ui::pending_marker());
    }
    let security_report = security::audit(site, exhaustive);
    security_report.render();
    issues.extend(security_report.required_failures());

    if let Some(site) = site {
        site::check(site, &mut issues, &mut pending);
    }

    if !pending.is_empty() {
        println!();
        for item in &pending {
            println!("  {} {item}", ui::pending_marker());
        }
    }

    if issues.is_empty() {
        if pending.is_empty() {
            println!("\n{} All checks passed.", ui::success_marker());
        } else {
            println!("\n{} Deployment needs one more step.", ui::pending_marker());
        }
        Ok(())
    } else {
        println!();
        for issue in &issues {
            if !issue.starts_with("security rule ") {
                println!("  {} {issue}", ui::failure_marker());
            }
        }
        anyhow::bail!("Doctor found {} issue{}", issues.len(), if issues.len() == 1 { "" } else { "s" });
    }
}
