use std::path::PathBuf;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::commands::{ensure_site_idle, service};
use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state as release_state;

pub fn rollback(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release rollback")?;
    // Rollback mutates live site state, so it must be serialized with deploy,
    // cancellation, pruning, and config receive like every other mutation. It
    // also runs against an idle site so it never repoints `current` under an
    // in-flight or interrupted deployment.
    let mutation = SiteMutation::acquire(site)?;
    ensure_site_idle(site)?;

    let cfg = mutation.config();

    let releases = release_state::list_releases_sorted(&cfg.project_root)?;
    if releases.len() < 2 {
        bail!("Need at least two releases to perform rollback");
    }

    let current_name = release_state::current_release_name(&cfg.project_root)?;
    let current_idx = releases
        .iter()
        .position(|name| name == &current_name)
        .with_context(|| format!("Current release '{current_name}' was not found in releases/"))?;

    if current_idx == 0 {
        bail!("Current release is already the oldest release; cannot roll back");
    }

    let previous_name = releases[current_idx - 1].clone();
    switch_and_verify(&cfg.project_root, &current_name, &previous_name, || service::run(&mutation))?;

    println!("Rollback complete: {current_name} -> {previous_name}");
    Ok(())
}

/// Repoints `current` to `previous_name`, restarts (via `restart`) and verifies
/// the result, then restores `current_name` and restarts it again if
/// verification fails. This keeps rollback reversible when service restart
/// reports failure.
fn switch_and_verify(
    project_root: &str,
    current_name: &str,
    previous_name: &str,
    restart: impl Fn() -> Result<()>,
) -> Result<()> {
    let current_link = PathBuf::from(project_root).join(paths::CURRENT_LINK);
    let previous_dir = release_state::release_dir(project_root, previous_name);
    let current_dir = release_state::release_dir(project_root, current_name);

    release_state::point_symlink_atomically(&current_link, &previous_dir)?;

    if let Err(error) = restart() {
        let mut error = error;
        if let Err(restore_error) = release_state::point_symlink_atomically(&current_link, &current_dir) {
            error =
                error.context(format!("Failed to restore the previous release '{current_name}': {restore_error:#}"));
        } else if let Err(restart_error) = restart() {
            error = error.context(format!("Failed to restart the restored release: {restart_error:#}"));
        }
        return Err(error.context(format!(
            "Service restart failed after rolling back to '{previous_name}'; restored '{current_name}'"
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::PathBuf;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;

    use super::switch_and_verify;

    fn temp_root(prefix: &str) -> Result<PathBuf> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nonce));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    #[test]
    fn rollback_points_current_to_previous_release_when_restart_succeeds() -> Result<()> {
        let root = temp_root("bonesremote_rollback_ok")?;
        let current_link = root.join("current");
        let previous_dir = root.join("releases/20260101_000000");
        let _current_dir = root.join("releases/20260102_000000");
        fs::create_dir_all(&previous_dir)?;
        symlink(&previous_dir, &current_link)?;

        let project_root = root.to_string_lossy().into_owned();
        switch_and_verify(&project_root, "20260102_000000", "20260101_000000", || Ok(()))?;

        assert_eq!(fs::read_link(&current_link)?, previous_dir);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn rollback_restores_original_release_when_restart_fails() -> Result<()> {
        let root = temp_root("bonesremote_rollback_restore")?;
        let current_link = root.join("current");
        let current_dir = root.join("releases/20260102_000000");
        let previous_dir = root.join("releases/20260101_000000");
        fs::create_dir_all(&previous_dir)?;
        fs::create_dir_all(&current_dir)?;
        symlink(&current_dir, &current_link)?;

        let project_root = root.to_string_lossy().into_owned();
        let result = switch_and_verify(&project_root, "20260102_000000", "20260101_000000", || {
            anyhow::bail!("simulated restart failure")
        });

        assert!(result.is_err());
        assert_eq!(fs::read_link(&current_link)?, current_dir, "current must be restored after failed rollback");
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
