use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::validate_site_name;
use bonesdeploy_core::paths;

use crate::commands::{drop_failed_release, release::list};
use crate::control_plane;
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::lifecycle::context;
use crate::release::state::{self as release_state, DeploymentLock, DeploymentRecord};
use crate::runtime::docker::command::release_image_tags;

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
        if active.phase().may_have_mutated_runtime() && list::process_matches(active) {
            bail!(
                "Release {release} is preparing and cannot be cancelled because prepare scripts may change runtime state."
            );
        }
        if list::process_matches(active) {
            terminate_deployment(active)?;
        }
    } else if release_state::read_staged_release(site).ok().as_deref() != Some(release) {
        bail!("Release {release} is not building or interrupted. Run 'bonesdeploy site releases' to inspect releases.");
    }

    let lock = DeploymentLock::acquire(site)?;
    let mutation = SiteMutation::adopt(site, config.clone(), lock)?;
    let current = mutation.active()?;
    if current.as_ref().is_some_and(|deployment| deployment.release() != release) {
        bail!("Active deployment changed while cancelling {release}; no cleanup was performed.");
    }

    if let Some(context) = current.as_ref().and_then(|deployment| deployment.context()) {
        let context = validate_build_context(site, &config.project_root, Path::new(context))?;
        let tags = release_image_tags(&context)?;
        drop_failed_release::remove_unreferenced_manifest_images(&mutation, &tags, Some(release))?;
        context::cleanup(site, &context)?;
    } else {
        cleanup_stale_contexts(site, &config.project_root, Some(&mutation))?;
    }

    let staged = mutation.staged_release()?;
    if staged.as_deref() == Some(release) {
        drop_failed_release::run_locked(&mutation)?;
    }
    mutation.clear_active()?;
    println!("Cancelled release: {release}");
    Ok(())
}

fn validate_build_context(site: &str, project_root: &str, context: &Path) -> Result<PathBuf> {
    let tmp_root = fs::canonicalize(Path::new(project_root).join(paths::TMP_BUILDS_DIR))
        .with_context(|| format!("Failed to resolve build context root for release {site}"))?;
    let metadata = fs::symlink_metadata(context)
        .with_context(|| format!("Failed to inspect recorded build context {}", context.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("Refusing to remove invalid build context recorded for release {site}: {}", context.display());
    }
    let canonical = fs::canonicalize(context)
        .with_context(|| format!("Failed to resolve recorded build context {}", context.display()))?;
    if canonical != context
        || !canonical.starts_with(&tmp_root)
        || !canonical.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("build-{site}-")))
    {
        bail!("Refusing to remove invalid build context recorded for release {site}: {}", context.display());
    }
    Ok(canonical)
}

fn cleanup_stale_contexts(site: &str, project_root: &str, mutation: Option<&SiteMutation>) -> Result<()> {
    let tmp_root = Path::new(project_root).join(paths::TMP_BUILDS_DIR);
    if !tmp_root.is_dir() {
        return Ok(());
    }
    let tmp_root = fs::canonicalize(&tmp_root)
        .with_context(|| format!("Failed to resolve build context root for release {site}"))?;
    for entry in fs::read_dir(&tmp_root).with_context(|| format!("Failed to read {}", tmp_root.display()))? {
        let path = entry?.path();
        if !path.file_name().is_some_and(|name| name.to_string_lossy().starts_with(&format!("build-{site}-"))) {
            continue;
        }
        let metadata = fs::symlink_metadata(&path)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            continue;
        }
        let canonical = fs::canonicalize(&path)?;
        if canonical != path || !canonical.starts_with(&tmp_root) {
            continue;
        }
        if let Some(mutation) = mutation {
            let tags = release_image_tags(&canonical)?;
            drop_failed_release::remove_unreferenced_manifest_images(mutation, &tags, None)?;
        }
        context::cleanup(site, &canonical)?;
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

pub fn wait_for_process_exit(active: &DeploymentRecord, timeout: Duration) -> bool {
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
    use std::fs;
    use std::os::unix::fs::symlink;

    use tempfile::tempdir;

    use super::*;

    #[test]
    fn cancellation_removes_stale_artifact_contexts() -> Result<()> {
        let root = tempdir()?;
        let context = root.path().join(paths::TMP_BUILDS_DIR).join("build-demo-1");
        fs::create_dir_all(&context)?;
        fs::write(context.join("partial"), "data")?;

        cleanup_stale_contexts("demo", &root.path().display().to_string(), None)?;
        assert!(!context.exists());
        Ok(())
    }

    #[test]
    fn stale_symlinked_context_is_not_cleaned() -> Result<()> {
        let root = tempdir()?;
        let tmp_root = root.path().join(paths::TMP_BUILDS_DIR);
        fs::create_dir_all(&tmp_root)?;
        let outside = root.path().join("outside");
        fs::create_dir(&outside)?;
        fs::write(outside.join("partial"), "must remain")?;
        let candidate = tmp_root.join("build-demo-symlink");
        symlink(&outside, &candidate)?;

        cleanup_stale_contexts("demo", &root.path().display().to_string(), None)?;

        assert!(candidate.is_symlink());
        assert!(outside.join("partial").exists());
        Ok(())
    }

    #[test]
    fn recorded_context_must_be_canonical_and_bound_before_inventory_cleanup() -> Result<()> {
        let root = tempdir()?;
        let tmp_root = root.path().join(paths::TMP_BUILDS_DIR);
        fs::create_dir_all(&tmp_root)?;
        let outside = root.path().join("outside");
        fs::create_dir(&outside)?;
        fs::write(outside.join(".bonesdeploy-compose-images.json"), b"not inventory")?;

        let Err(error) = validate_build_context("demo", &root.path().display().to_string(), &outside) else {
            bail!("out-of-bound context was accepted")
        };
        assert!(error.to_string().contains("invalid build context"));
        Ok(())
    }

    #[test]
    fn symlinked_recorded_context_cannot_escape_build_root() -> Result<()> {
        let root = tempdir()?;
        let tmp_root = root.path().join(paths::TMP_BUILDS_DIR);
        fs::create_dir_all(&tmp_root)?;
        let outside = root.path().join("outside");
        fs::create_dir(&outside)?;
        symlink(&outside, tmp_root.join("build-demo-1"))?;

        let Err(error) =
            validate_build_context("demo", &root.path().display().to_string(), &tmp_root.join("build-demo-1"))
        else {
            bail!("symlinked context was accepted")
        };
        assert!(error.to_string().contains("invalid build context"));
        Ok(())
    }
}
