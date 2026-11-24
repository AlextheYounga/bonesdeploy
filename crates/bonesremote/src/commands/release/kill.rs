use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::validate_site_name;

use crate::control_plane;
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::cancellation;
use crate::release::process;
use crate::release::state::{self as release_state, DeploymentLock, DeploymentRecord};

const PROCESS_STOP_TIMEOUT: Duration = Duration::from_secs(5);

pub fn run(site: &str, release: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release kill")?;

    // A live deployment holds the site's lock, so cancellation must verify the
    // site identity and stop the process *before* the lock becomes available.
    // Only then is the guard assembled and used for all file mutations.
    validate_site_name(site)?;
    let config = control_plane::load(site)?.into_site_config(site);
    let active = release_state::read_active_deployment(site)?;
    if let Some(active) = &active {
        if active.release() != release {
            bail!(
                "Release {release} is not the active deployment. Run 'bonesdeploy site releases' to inspect releases."
            );
        }
        if active.phase().may_have_mutated_runtime() && process::matches(active) {
            bail!(
                "Release {release} is preparing and cannot be cancelled because prepare scripts may change runtime state."
            );
        }
        if process::matches(active) {
            terminate_deployment(active)?;
        }
    } else if release_state::read_staged_release(site).ok().as_deref() != Some(release) {
        bail!("Release {release} is not building or interrupted. Run 'bonesdeploy site releases' to inspect releases.");
    }

    let lock = DeploymentLock::acquire(site)?;
    let mutation = SiteMutation::adopt(site, config, lock)?;
    cancellation::finish_locked(&mutation, release)?;
    println!("Cancelled release: {release}");
    Ok(())
}

fn terminate_deployment(active: &DeploymentRecord) -> Result<()> {
    signal(active.pid(), "TERM")?;
    if wait_for_process_exit(active, PROCESS_STOP_TIMEOUT) {
        return Ok(());
    }

    signal(active.pid(), "KILL")?;
    if wait_for_process_exit(active, PROCESS_STOP_TIMEOUT) {
        return Ok(());
    }

    bail!("Deployment process {} did not stop", active.pid());
}

fn signal(pid: u32, signal: &str) -> Result<()> {
    let status = Command::new("kill")
        .args([format!("-{signal}"), pid.to_string()])
        .status()
        .with_context(|| format!("Failed to send SIG{signal} to deployment process {pid}"))?;
    if !status.success() {
        bail!("Failed to send SIG{signal} to deployment process {pid}: {status}");
    }
    Ok(())
}

pub fn wait_for_process_exit(active: &DeploymentRecord, timeout: Duration) -> bool {
    let attempts = timeout.as_millis() / 100;
    for _ in 0..attempts {
        if !process::matches(active) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    !process::matches(active)
}
