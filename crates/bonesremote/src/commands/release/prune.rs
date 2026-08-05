use std::fs;

use anyhow::{Context, Result};

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::state as release_state;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote release prune")?;
    let mutation = SiteMutation::acquire(site)?;
    run_locked(&mutation)
}

pub(crate) fn run_locked(mutation: &SiteMutation) -> Result<()> {
    let project_root = &mutation.config().project_root;

    let pruned = prune_old_releases(project_root, mutation.config().releases_keep)?;
    if !pruned.is_empty() {
        println!("Pruned releases: {}", pruned.join(", "));
    }

    Ok(())
}

fn prune_old_releases(project_root: &str, keep: usize) -> Result<Vec<String>> {
    let active_release = release_state::current_release_name(project_root)?;
    let releases = release_state::list_releases_sorted(project_root)?;
    let keep = keep.max(1);

    // Compute the plan before touching the filesystem. The active release is
    // never a candidate, so an active release that happens to be the oldest
    // must not stall pruning (the old remove/push-back loop selected it forever).
    let excess = releases.len().saturating_sub(keep);
    let candidates = releases.into_iter().filter(|release| release != &active_release).take(excess);

    let mut pruned = Vec::new();
    for release in candidates {
        let path = release_state::release_dir(project_root, &release);
        if path.exists() {
            fs::remove_dir_all(&path).with_context(|| format!("Failed to prune old release {}", path.display()))?;
            pruned.push(release);
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
    use bonesdeploy_core::paths;

    use super::prune_old_releases;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn project_root_for(temp_root: &Path) -> String {
        temp_root.join("project_root").to_string_lossy().to_string()
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
        let project_root = project_root_for(&root);

        make_release(&root, "20260101_000000")?;
        make_release(&root, "20260102_000000")?;
        make_release(&root, "20260103_000000")?;
        set_current_release(&root, "20260103_000000")?;

        let pruned = prune_old_releases(&project_root, 2)?;

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
        let project_root = project_root_for(&root);

        make_release(&root, "20260101_000000")?;
        make_release(&root, "20260102_000000")?;
        set_current_release(&root, "20260101_000000")?;

        let pruned = prune_old_releases(&project_root, 2)?;

        assert!(pruned.is_empty());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260101_000000").exists());
        assert!(root.join("project_root").join(paths::RELEASES_DIR).join("20260102_000000").exists());

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn prune_old_releases_terminates_when_active_is_the_oldest_release() -> Result<()> {
        let root = temp_dir("bonesremote_post_deploy_prune_active_oldest")?;
        let project_root = project_root_for(&root);

        make_release(&root, "20260101_000000")?;
        make_release(&root, "20260102_000000")?;
        make_release(&root, "20260103_000000")?;
        set_current_release(&root, "20260101_000000")?;

        let pruned = prune_old_releases(&project_root, 2)?;
        let releases = root.join("project_root").join(paths::RELEASES_DIR);

        assert_eq!(pruned, vec!["20260102_000000"]);
        assert!(releases.join("20260101_000000").exists());
        assert!(!releases.join("20260102_000000").exists());
        assert!(releases.join("20260103_000000").exists());

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
