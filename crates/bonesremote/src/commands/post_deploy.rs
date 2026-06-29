use std::fs;

use anyhow::{Context, Result};
use shared::paths;
use shared::registry;

use crate::privileges;
use crate::release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release prune")?;
    registry::validate_site_name(site)?;

    let registry_path = paths::bonesremote_registry_path(site);
    let cfg = registry::load(&registry_path)
        .with_context(|| format!("Failed to load remote site state from {}", registry_path.display()))?;

    let pruned = prune_old_releases(&cfg)?;
    if !pruned.is_empty() {
        println!("Pruned releases: {}", pruned.join(", "));
    }

    Ok(())
}

fn prune_old_releases(cfg: &registry::Registry) -> Result<Vec<String>> {
    let active_release = release_state::current_release_name(cfg)?;
    let mut releases = release_state::list_releases_sorted(cfg)?;
    let keep = cfg.releases_keep.max(1);

    let mut pruned = Vec::new();
    while releases.len() > keep {
        let oldest = releases.remove(0);
        if oldest == active_release {
            releases.push(oldest);
            releases.sort();
            continue;
        }

        let path = release_state::release_dir(cfg, &oldest);
        if path.exists() {
            fs::remove_dir_all(&path).with_context(|| format!("Failed to prune old release {}", path.display()))?;
            pruned.push(oldest);
        }
    }

    Ok(pruned)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;
    use shared::registry::Registry;

    use super::prune_old_releases;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn config_for(temp_root: &Path, keep: usize) -> Registry {
        let site_root = temp_root.join("project_root");
        Registry {
            site: String::from("acme"),
            repo_path: temp_root.join("repo.git").to_string_lossy().to_string(),
            site_root: site_root.to_string_lossy().to_string(),
            shared_root: site_root.join(paths::SHARED_DIR).to_string_lossy().to_string(),
            releases_root: site_root.join(paths::RELEASES_DIR).to_string_lossy().to_string(),
            current_path: site_root.join(paths::CURRENT_LINK).to_string_lossy().to_string(),
            runtime_user: String::from("acme"),
            runtime_group: String::from("acme"),
            branch: String::from("main"),
            deploy_on_push: true,
            releases_keep: keep,
        }
    }

    fn make_release(root: &Path, name: &str) -> Result<()> {
        fs::create_dir_all(root.join("project_root").join(paths::RELEASES_DIR).join(name))?;
        Ok(())
    }

    fn set_current_release(root: &Path, name: &str) -> Result<()> {
        let project_root = root.join("project_root");
        let releases = project_root.join(paths::RELEASES_DIR);
        fs::create_dir_all(&releases)?;
        let target = releases.join(name);
        symlink(&target, project_root.join(paths::CURRENT_LINK))?;
        Ok(())
    }

    #[test]
    fn prune_old_releases_removes_oldest_inactive_releases_up_to_keep_limit() -> Result<()> {
        let root = temp_dir("bonesremote_post_deploy_prune")?;
        let cfg = config_for(&root, 2);

        make_release(&root, "20260101_000000")?;
        make_release(&root, "20260102_000000")?;
        make_release(&root, "20260103_000000")?;
        set_current_release(&root, "20260103_000000")?;

        let pruned = prune_old_releases(&cfg)?;

        assert_eq!(pruned, vec!["20260101_000000"]);
        assert!(!root.join("project_root").join(paths::RELEASES_DIR).join("20260101_000000").exists());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260102_000000").exists());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260103_000000").exists());

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn prune_old_releases_keeps_active_release_when_within_keep_limit() -> Result<()> {
        let root = temp_dir("bonesremote_post_deploy_prune_active")?;
        let cfg = config_for(&root, 2);

        make_release(&root, "20260101_000000")?;
        make_release(&root, "20260102_000000")?;
        set_current_release(&root, "20260101_000000")?;

        let pruned = prune_old_releases(&cfg)?;

        assert!(pruned.is_empty());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260101_000000").exists());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260102_000000").exists());

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
