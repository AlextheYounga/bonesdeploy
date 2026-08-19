use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::{build_user_for, validate_site_name};
use bonesdeploy_core::paths;

use crate::commands::{drop_failed_release, release::list};
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle::build::ensure_build_user_ready;
use crate::release::lifecycle::{self, build::remove_build_container, checkout};
use crate::release::state::{self as release_state, DeploymentLock, DeploymentRecord};

const PROCESS_STOP_TIMEOUT: Duration = Duration::from_secs(5);

pub fn run(site: &str, release: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release kill")?;

    // A live deployment holds the site's lock, so cancellation must verify the
    // site identity and stop the process *before* the lock becomes available.
    // Only then is the guard assembled and used for all file mutations.
    validate_site_name(site)?;
    let config = lifecycle::load_site_config(site)?;
    let active = release_state::read_active_deployment(site)?;
    if let Some(active) = &active {
        if active.release() != release {
            bail!("Release {release} is not the active deployment. Run 'bonesdeploy releases' to inspect releases.");
        }
        if active.phase().may_have_mutated_runtime() && list::process_matches(active) {
            bail!(
                "Release {release} is preparing and cannot be cancelled because prepare scripts may change runtime state."
            );
        }
        if list::process_matches(active) {
            terminate_deployment(active)?;
        }
    } else if release_state::read_staged_release(site).ok().as_deref() != Some(release) {
        bail!("Release {release} is not building or interrupted. Run 'bonesdeploy releases' to inspect releases.");
    }

    let lock = DeploymentLock::acquire(site)?;
    let mutation = SiteMutation::adopt(site, config, lock);
    let current = mutation.active()?;
    if current.as_ref().is_some_and(|deployment| deployment.release() != release) {
        bail!("Active deployment changed while cancelling {release}; no cleanup was performed.");
    }

    let build_user = build_user_for(&mutation.config().project_name);
    let working_dir = Path::new(&mutation.config().project_root);
    ensure_build_user_ready(&build_user, working_dir)?;
    remove_build_container(&build_user, &mutation.config().project_name, working_dir)?;

    if let Some(context) = current.as_ref().and_then(|deployment| deployment.context()) {
        let context = Path::new(context);
        let tmp_root = Path::new(&mutation.config().project_root).join(paths::TMP_BUILDS_DIR);
        if !context.starts_with(&tmp_root)
            || !context.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("build-{site}-")))
        {
            bail!("Refusing to remove invalid build context recorded for release {release}: {}", context.display());
        }
        checkout::cleanup_build_context(site, context)?;
    } else {
        cleanup_stale_contexts(site, &mutation.config().project_root)?;
    }

    let staged = mutation.staged_release()?;
    if staged.as_deref() == Some(release) {
        drop_failed_release::run_locked(&mutation)?;
    }
    mutation.clear_active()?;
    println!("Cancelled release: {release}");
    Ok(())
}

fn cleanup_stale_contexts(site: &str, project_root: &str) -> Result<()> {
    let tmp_root = Path::new(project_root).join(paths::TMP_BUILDS_DIR);
    if !tmp_root.is_dir() {
        return Ok(());
    }
    for entry in fs::read_dir(&tmp_root).with_context(|| format!("Failed to read {}", tmp_root.display()))? {
        let path = entry?.path();
        if path.is_dir()
            && path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("build-{site}-")))
        {
            checkout::cleanup_build_context(site, &path)?;
        }
    }
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

fn wait_for_process_exit(active: &DeploymentRecord, timeout: Duration) -> bool {
    let attempts = timeout.as_millis() / 100;
    for _ in 0..attempts {
        if !list::process_matches(active) {
            return true;
        }
        thread::sleep(Duration::from_millis(100));
    }
    !list::process_matches(active)
}

#[cfg(test)]
mod tests {
    use super::wait_for_process_exit;
    use crate::release::state::{DeploymentPhase, DeploymentRecord};
    use std::time::Duration;

    #[test]
    fn wait_returns_when_process_is_already_gone() {
        let deployment = DeploymentRecord::new(
            String::from("20260715_225306"),
            String::from("46a0b75c"),
            DeploymentPhase::Created,
            u32::MAX,
            0,
            String::new(),
        );
        assert!(wait_for_process_exit(&deployment, Duration::from_millis(1)));
    }
}
