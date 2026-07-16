use anyhow::Result;
use shared::paths;
use std::io::{self, IsTerminal};

fn style(code: &str, value: &str) -> String {
    if io::stdout().is_terminal() { format!("\x1b[{code}m{value}\x1b[0m") } else { value.to_string() }
}

mod apparmor;
mod site;
mod system;

pub fn run(site: Option<&str>) -> Result<()> {
    println!("{}", style("1", &format!("{} doctor", paths::BONESREMOTE_BINARY)));

    let mut issues: Vec<String> = Vec::new();
    let mut pending: Vec<String> = Vec::new();

    system::check_supported_distribution(&mut issues);
    system::check_podman_available(&mut issues);
    system::check_passwordless_sudo(&mut issues);
    apparmor::check_support(&mut issues);

    if let Some(site) = site {
        site::check(site, &mut issues, &mut pending);
    }

    if !pending.is_empty() {
        println!();
        for item in &pending {
            println!("  {} {item}", style("1;33", "•"));
        }
    }

    if issues.is_empty() {
        if pending.is_empty() {
            println!("\n{} All checks passed.", style("1;32", "OK"));
        } else {
            println!("\n{} Deployment needs one more step.", style("1;33", "PENDING"));
        }
        Ok(())
    } else {
        println!();
        for issue in &issues {
            println!("  {} {issue}", style("1;31", "!"));
        }
        anyhow::bail!("Doctor found {} issue{}", issues.len(), if issues.len() == 1 { "" } else { "s" });
    }
}
