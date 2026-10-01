use std::iter::once;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::inspection::systemd;
use crate::privileges;
use crate::release::SiteMutation;

pub fn run(mutation: &SiteMutation) -> Result<()> {
    run_with_options(mutation, true)
}

pub fn run_for_release(mutation: &SiteMutation) -> Result<()> {
    run_with_options(mutation, false)
}

pub fn stop(mutation: &SiteMutation) -> Result<()> {
    privileges::ensure_root("bonesremote shared import")?;

    let target_name = paths::site_target_name(mutation.site());
    let services = systemd::required_services(&target_name)?;
    if services.is_empty() {
        bail!("Site target {target_name} has no registered services");
    }

    let status = Command::new("systemctl")
        .args(["stop", "--", &target_name])
        .status()
        .with_context(|| format!("Failed to stop {target_name}"))?;
    if !status.success() {
        bail!("Failed to stop {target_name}");
    }
    let active: Vec<&str> = services
        .iter()
        .map(String::as_str)
        .chain(once(target_name.as_str()))
        .filter(|unit| systemd::is_active(unit))
        .collect();
    if !active.is_empty() {
        bail!("Site services are still active after stopping {target_name}: {}", active.join(", "));
    }
    Ok(())
}

fn run_with_options(mutation: &SiteMutation, start_target: bool) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;

    let site = mutation.site();
    let target_name = paths::site_target_name(site);
    let registered = systemd::required_services(&target_name)?;
    let services = registered;
    if services.is_empty() {
        bail!("Site target {target_name} has no registered services");
    }

    if start_target {
        let status = Command::new("systemctl")
            .args(["start", "--", &target_name])
            .status()
            .with_context(|| format!("Failed to restart {target_name}"))?;

        if !status.success() {
            bail!("Failed to restart {target_name}");
        }
    }

    for service in &services {
        restart_service(service)?;
    }

    verify_units_active(&target_name, &services)?;

    println!("Restarted {target_name}: {}", services.join(", "));
    Ok(())
}

fn restart_service(service: &str) -> Result<()> {
    let status = Command::new("systemctl")
        .args(["restart", "--", service])
        .status()
        .with_context(|| format!("Failed to restart {service}"))?;
    if !status.success() {
        bail!("Failed to restart {service}");
    }
    Ok(())
}

// `systemctl restart` exits 0 as soon as the start job is queued; it does not
// confirm the unit stays up. Re-check each required service so a unit that
// starts then immediately exits is reported as a failure here instead of
// silently leaving the site down.
fn verify_units_active(target: &str, services: &[String]) -> Result<()> {
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
