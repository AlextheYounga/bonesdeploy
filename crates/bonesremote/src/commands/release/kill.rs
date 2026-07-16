use std::fs;
use std::path::Path;
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use shared::config::{self, build_user_for};
use shared::paths;

use crate::commands::{drop_failed_release, release::list};
use crate::privileges;
use crate::release::lifecycle::checkout;
use crate::release::script_runner::{ensure_build_user_ready, remove_build_container};
use crate::release::state::{self as release_state, DeploymentPhase};

const PROCESS_STOP_TIMEOUT: Duration = Duration::from_secs(5);

pub fn run(site: &str, release: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release kill")?;
    let cfg = config::load(&paths::bonesremote_bones_toml_path(site))
        .context("Failed to load release cancellation site configuration")?;
    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{site}'", cfg.project_name);
    }
    let active = release_state::read_active_deployment(site)?;
    if let Some(active) = &active {
        if active.release != release {
            bail!("Release {release} is not the active deployment. Run 'bonesdeploy releases' to inspect releases.");
        }
        if active.phase == DeploymentPhase::Preparing && list::process_matches(active) {
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

    let _lock = release_state::DeploymentLock::acquire(site)?;
    let current = release_state::read_active_deployment(site)?;
    if current.as_ref().is_some_and(|deployment| deployment.release != release) {
        bail!("Active deployment changed while cancelling {release}; no cleanup was performed.");
    }

    let build_user = build_user_for(&cfg.project_name);
    let working_dir = Path::new(&cfg.project_root);
    ensure_build_user_ready(&build_user, working_dir)?;
    remove_build_container(&build_user, &cfg.project_name, working_dir)?;

    if let Some(context) = current.as_ref().and_then(|deployment| deployment.context.as_deref()) {
        let context = Path::new(context);
        let tmp_root = Path::new(&cfg.project_root).join(paths::TMP_BUILDS_DIR);
        if !context.starts_with(&tmp_root)
            || !context.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("build-{site}-")))
        {
            bail!("Refusing to remove invalid build context recorded for release {release}: {}", context.display());
        }
        checkout::cleanup_build_context(site, context)?;
    } else {
        cleanup_stale_contexts(site, &cfg.project_root)?;
    }

    let staged = release_state::read_staged_release(site).ok();
    if staged.as_deref() == Some(release) {
        drop_failed_release::run(site)?;
    }
    release_state::clear_active_deployment(site)?;
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

fn terminate_deployment(active: &release_state::ActiveDeployment) -> Result<()> {
    signal(active.pid, "TERM")?;
    if wait_for_process_exit(active, PROCESS_STOP_TIMEOUT) {
        return Ok(());
    }

    signal(active.pid, "KILL")?;
    if wait_for_process_exit(active, PROCESS_STOP_TIMEOUT) {
        return Ok(());
    }

    bail!("Deployment process {} did not stop", active.pid);
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

fn wait_for_process_exit(active: &release_state::ActiveDeployment, timeout: Duration) -> bool {
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
    use crate::release::state::{ActiveDeployment, DeploymentPhase};
    use std::time::Duration;

    #[test]
    fn wait_returns_when_process_is_already_gone() {
        let deployment = ActiveDeployment {
            release: String::from("20260715_225306"),
            pid: u32::MAX,
            process_start_ticks: 0,
            phase: DeploymentPhase::Building,
            started_at: String::new(),
            context: None,
        };
        assert!(wait_for_process_exit(&deployment, Duration::from_millis(1)));
    }
}
