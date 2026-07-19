use std::collections::BTreeSet;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use shared::{config, paths};

use crate::privileges;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;
    config::validate_project_name(site)?;

    let config_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&config_path)
        .with_context(|| format!("Failed to load registered site state from {}", config_path.display()))?;
    let target_name = target_name_for_registered_site(site, &cfg.project_name)?;
    let services = target_services(&target_name)?;
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

    let failed: Vec<&str> = services.iter().map(String::as_str).filter(|unit| !is_active(unit)).collect();

    if failed.is_empty() {
        return Ok(());
    }

    let names = failed.join(", ");
    bail!("Restart of {target} reported success, but these units are not active: {names}\n{}", journal_output(&failed));
}

fn is_active(unit: &str) -> bool {
    Command::new("systemctl").args(["is-active", "--quiet", "--", unit]).status().is_ok_and(|status| status.success())
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

fn target_services(target: &str) -> Result<Vec<String>> {
    let output = Command::new("systemctl")
        .args(["show", "--property=Requires", "--value", "--no-pager", "--", target])
        .output()
        .with_context(|| format!("Failed to inspect {target}"))?;
    if !output.status.success() {
        bail!("Failed to inspect {target}");
    }

    Ok(parse_target_services(&String::from_utf8_lossy(&output.stdout)))
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

fn parse_target_services(output: &str) -> Vec<String> {
    output
        .split_whitespace()
        .filter(|name| name.ends_with(paths::SYSTEMD_SERVICE_SUFFIX))
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{parse_target_services, restart_args, target_name_for_registered_site};
    use shared::paths;

    #[test]
    fn target_dependencies_include_only_services() {
        let names = "nexttest-nginx.service nexttest-next.service nexttest.target";
        assert_eq!(parse_target_services(names), ["nexttest-next.service", "nexttest-nginx.service"]);
    }

    #[test]
    fn restart_uses_the_site_target_not_a_runtime_service() {
        let target = paths::site_target_name("nexttest");
        assert_eq!(restart_args(&target), ["restart", "--", "nexttest.target"]);
    }

    #[test]
    fn site_cannot_restart_another_projects_target() {
        assert!(target_name_for_registered_site("shop", "shop-admin").is_err());
    }
}
