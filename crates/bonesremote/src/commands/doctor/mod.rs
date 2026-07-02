use anyhow::Result;
use shared::paths;

const BOLD: &str = "\x1b[1m";
const GREEN_BOLD: &str = "\x1b[1;32m";
const RED_BOLD: &str = "\x1b[1;31m";
const RESET: &str = "\x1b[0m";

mod apparmor;
mod site;
mod system;

pub fn run(site: Option<&str>) -> Result<()> {
    println!("{BOLD}{} doctor{RESET}", paths::BONESREMOTE_BINARY);

    let mut issues: Vec<String> = Vec::new();

    system::check_supported_distribution(&mut issues);
    system::check_globally_available(&mut issues);
    system::check_podman_available(&mut issues);
    system::check_passwordless_sudo(&mut issues);
    apparmor::check_support(&mut issues);

    if let Some(site) = site {
        site::check(site, &mut issues);
    }

    if issues.is_empty() {
        println!("\n{GREEN_BOLD}OK{RESET} All checks passed.");
        Ok(())
    } else {
        println!();
        for issue in &issues {
            println!("  {RED_BOLD}!{RESET} {issue}");
        }
        anyhow::bail!("Doctor found {} issue{}", issues.len(), if issues.len() == 1 { "" } else { "s" });
    }
}
