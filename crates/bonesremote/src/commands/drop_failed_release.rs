use std::fs;

use anyhow::{Context, Result, bail};

use crate::privileges;
use crate::release::SiteMutation;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;
    let mutation = SiteMutation::acquire(site)?;
    run_locked(&mutation)
}

pub(crate) fn run_locked(mutation: &SiteMutation) -> Result<()> {
    let staged = mutation.staged_release()?;
    let Some(release_name) = staged.filter(|name| !name.is_empty()) else {
        println!("No staged release state found. Nothing to clean.");
        return Ok(());
    };

    let release_dir = mutation.release_dir(&release_name);
    ensure_release_not_active(mutation, &release_name)?;
    if release_dir.exists() {
        fs::remove_dir_all(&release_dir)
            .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
        println!("Removed failed release: {release_name}");
    }

    mutation.clear_staged_release()?;
    println!("Cleared staged release state.");
    Ok(())
}

fn ensure_release_not_active(mutation: &SiteMutation, release: &str) -> Result<()> {
    let current = mutation.current_release_name().context("Failed to determine the active release before cleanup")?;
    if current == release {
        bail!("Refusing to remove active release {release}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use bonesdeploy_core::config::Bones;
    use bonesdeploy_core::paths;

    use super::ensure_release_not_active;
    use crate::release::SiteMutation;
    use crate::release::state::{DeploymentLock, set_sites_root_for_tests};

    #[test]
    fn active_release_cannot_be_dropped() -> Result<()> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = env::temp_dir().join(format!("bonesremote_drop_{}_{}", process::id(), nonce));
        let _root_guard = set_sites_root_for_tests(root.clone());
        let release = root.join(paths::RELEASES_DIR).join("active-release");
        fs::create_dir_all(&release)?;
        symlink(&release, root.join(paths::CURRENT_LINK))?;

        let mut config = Bones::for_site("demo");
        config.project_root = root.to_string_lossy().into_owned();
        let mutation = SiteMutation::adopt("demo", config, DeploymentLock::acquire("demo")?);
        assert!(ensure_release_not_active(&mutation, "active-release").is_err());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn cleanup_requires_a_readable_active_release() -> Result<()> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = env::temp_dir().join(format!("bonesremote_drop_missing_current_{}_{}", process::id(), nonce));
        let _root_guard = set_sites_root_for_tests(root.clone());
        fs::create_dir_all(&root)?;

        let mut config = Bones::for_site("demo");
        config.project_root = root.to_string_lossy().into_owned();
        let mutation = SiteMutation::adopt("demo", config, DeploymentLock::acquire("demo")?);
        assert!(ensure_release_not_active(&mutation, "candidate").is_err());

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
