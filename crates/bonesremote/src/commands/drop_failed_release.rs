use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state as release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;
    let mutation = SiteMutation::acquire(site)?;
    run_locked(&mutation)
}

pub(crate) fn run_locked(mutation: &SiteMutation) -> Result<()> {
    let site = mutation.site();
    let staged = release_state::staged_release(site)?;
    let Some(release_name) = staged.filter(|name| !name.is_empty()) else {
        println!("No staged release state found. Nothing to clean.");
        return Ok(());
    };

    let release_dir = release_state::release_dir(&mutation.config().project_root, &release_name);
    ensure_release_not_active(Path::new(&mutation.config().project_root), &release_name)?;
    if release_dir.exists() {
        fs::remove_dir_all(&release_dir)
            .with_context(|| format!("Failed to remove failed release {}", release_dir.display()))?;
        println!("Removed failed release: {release_name}");
    }

    release_state::clear_staged_release(site)?;
    println!("Cleared staged release state.");
    Ok(())
}

fn ensure_release_not_active(project_root: &Path, release: &str) -> Result<()> {
    let current = release_state::current_release_name(&project_root.to_string_lossy())
        .context("Failed to determine the active release before cleanup")?;
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
    use bonesdeploy_core::paths;

    use super::ensure_release_not_active;

    #[test]
    fn active_release_cannot_be_dropped() -> Result<()> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = env::temp_dir().join(format!("bonesremote_drop_{}_{}", process::id(), nonce));
        let release = root.join(paths::RELEASES_DIR).join("active-release");
        fs::create_dir_all(&release)?;
        symlink(&release, root.join(paths::CURRENT_LINK))?;

        assert!(ensure_release_not_active(&root, "active-release").is_err());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn cleanup_requires_a_readable_active_release() -> Result<()> {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let root = env::temp_dir().join(format!("bonesremote_drop_missing_current_{}_{}", process::id(), nonce));
        fs::create_dir_all(&root)?;

        assert!(ensure_release_not_active(&root, "candidate").is_err());

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
