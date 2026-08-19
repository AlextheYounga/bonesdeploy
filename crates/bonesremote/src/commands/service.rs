use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::commands::inspection::systemd;
use crate::privileges;
use crate::release::SiteMutation;

pub fn run(mutation: &SiteMutation) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;

    let cfg = mutation.config();
    let target_name = target_name_for_registered_site(mutation.site(), &cfg.project_name)?;
    let services = systemd::required_services(&target_name)?;
    if services.is_empty() {
        bail!("Site target {target_name} has no registered services");
    }

    let status = Command::new("systemctl")
        .args(restart_args(&target_name))
        .status()
        .with_context(|| format!("Failed to restart {target_name}"))?;

    if !status.success() {
        bail!("Failed to restart {target_name}");
    }

    verify_units_active(&target_name, &services)?;

    println!("Restarted {target_name}: {}", services.join(", "));
    Ok(())
}

// `systemctl restart` exits 0 as soon as the start job is queued; it does not
// confirm the unit stays up. Re-check each required service so a unit that
// starts then immediately exits is reported as a failure here instead of
// silently leaving the site down.
fn verify_units_active(target: &str, services: &[String]) -> Result<()> {
    // ponytail: 1s window catches immediate post-start crashes. Slower failures
    // still surface via journald/watchdog later; upgrade to a configurable wait
    // or `systemctl --wait` if a longer settle window becomes necessary.
    thread::sleep(Duration::from_secs(1));

    let failed: Vec<&str> = services.iter().map(String::as_str).filter(|unit| !systemd::is_active(unit)).collect();

    if failed.is_empty() {
        return Ok(());
    }

    let names = failed.join(", ");
    bail!("Restart of {target} reported success, but these units are not active: {names}\n{}", journal_output(&failed));
}

fn journal_output(units: &[&str]) -> String {
    let mut cmd = Command::new("journalctl");
    cmd.arg("--no-pager").arg("-n").arg("20");
    for unit in units {
        cmd.arg("-u").arg(unit);
    }
    match cmd.output() {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout).into_owned(),
        _ => String::new(),
    }
}

fn restart_args(target: &str) -> [&str; 3] {
    ["restart", "--", target]
}

fn target_name_for_registered_site(site: &str, registered_site: &str) -> Result<String> {
    if registered_site != site {
        bail!("Registered site state belongs to '{registered_site}', expected '{site}'");
    }
    Ok(paths::site_target_name(site))
}

#[cfg(test)]
mod tests {
    use super::target_name_for_registered_site;

    #[test]
    fn site_cannot_restart_another_projects_target() {
        assert!(target_name_for_registered_site("shop", "shop-admin").is_err());
    }
}
