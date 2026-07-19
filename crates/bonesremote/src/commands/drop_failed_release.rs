use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use shared::config;
use shared::paths;

use crate::privileges;
use crate::release::state as release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release drop-failed")?;

    let staged_path = release_state::staged_release_path(site);
    if !staged_path.exists() {
        println!("No staged release state found. Nothing to clean.");
        return Ok(());
    }

    let release_name = match release_state::read_staged_release(site) {
        Ok(name) => name,
        Err(error) => {
            release_state::clear_staged_release(site)
                .with_context(|| format!("Failed to clear invalid staged release state for {site}"))?;
            return Err(error).context("Staged release state was invalid and has been cleared");
        }
    };

    let bones_path = paths::bonesremote_bones_toml_path(site);
    let cfg = config::load(&bones_path)
        .with_context(|| format!("Failed to load remote site state from {}", bones_path.display()))?;

    if cfg.project_name != site {
        bail!("Remote site state belongs to '{}', expected '{}'", cfg.project_name, site);
    }

    let release_dir = release_state::release_dir(&cfg.project_root, &release_name);
    ensure_release_not_active(Path::new(&cfg.project_root), &release_name)?;
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
    use shared::paths;

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
