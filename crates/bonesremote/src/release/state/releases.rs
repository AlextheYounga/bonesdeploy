use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use bonesdeploy_core::paths;

pub fn release_dir(project_root: &str, release: &str) -> PathBuf {
    PathBuf::from(project_root).join(paths::RELEASES_DIR).join(release)
}

pub fn releases_dir(project_root: &str) -> PathBuf {
    PathBuf::from(project_root).join(paths::RELEASES_DIR)
}

pub fn shared_dir(project_root: &str) -> PathBuf {
    PathBuf::from(project_root).join(paths::SHARED_DIR)
}

pub fn current_release_dir(project_root: &str) -> Result<PathBuf> {
    let current_link = PathBuf::from(project_root).join(paths::CURRENT_LINK);
    let active_target =
        fs::read_link(&current_link).with_context(|| format!("Failed to read {}", current_link.display()))?;

    if active_target.is_absolute() {
        return Ok(active_target);
    }

    let parent = current_link
        .parent()
        .with_context(|| format!("Current release link has no parent: {}", current_link.display()))?;
    Ok(parent.join(active_target))
}

pub fn current_release_name(project_root: &str) -> Result<String> {
    let current_release = current_release_dir(project_root)?;
    current_release
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve current release name from {}", current_release.display()))
}

pub fn list_releases_sorted(project_root: &str) -> Result<Vec<String>> {
    let releases_dir = releases_dir(project_root);
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&releases_dir)
        .with_context(|| format!("Failed to read releases dir: {}", releases_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name != paths::PLACEHOLDER_RELEASE_NAME {
                names.push(name);
            }
        }
    }

    names.sort();
    Ok(names)
}

pub fn point_symlink_atomically(link_path: &Path, target_path: &Path) -> Result<()> {
    let Some(parent) = link_path.parent() else {
        bail!("Invalid symlink path: {}", link_path.display());
    };

    fs::create_dir_all(parent).with_context(|| format!("Failed to create symlink parent: {}", parent.display()))?;

    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).context("System clock is before UNIX_EPOCH")?.as_nanos();
    let temp_name = format!(".tmp_current_{}_{}", process::id(), nanos);
    let temp_link = parent.join(temp_name);

    if fs::symlink_metadata(&temp_link).is_ok() {
        fs::remove_file(&temp_link)
            .with_context(|| format!("Failed to cleanup stale temp link: {}", temp_link.display()))?;
    }

    symlink(target_path, &temp_link).with_context(|| {
        format!("Failed to create temporary symlink {} -> {}", temp_link.display(), target_path.display())
    })?;

    fs::rename(&temp_link, link_path).with_context(|| {
        format!("Failed to atomically switch symlink {} -> {}", link_path.display(), target_path.display())
    })
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

    use super::{current_release_name, list_releases_sorted, point_symlink_atomically, release_dir};

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_releases_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    fn project_root_for(root: &Path) -> String {
        root.join("deploy").to_string_lossy().to_string()
    }

    #[test]
    fn point_symlink_atomically_creates_parent_dirs_and_points_to_target() -> Result<()> {
        let root = temp_dir_path("point_symlink_parent");
        fs::create_dir_all(&root)?;

        let target = root.join("target_dir");
        fs::create_dir_all(&target)?;

        let link_path = root.join("nested/path/current");
        point_symlink_atomically(&link_path, &target)?;

        assert!(link_path.exists());
        let linked = fs::read_link(&link_path)?;
        assert_eq!(linked, target);

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn point_symlink_atomically_repoints_existing_link() -> Result<()> {
        let root = temp_dir_path("point_symlink_repoint");
        fs::create_dir_all(&root)?;

        let target_a = root.join("target_a");
        let target_b = root.join("target_b");
        fs::create_dir_all(&target_a)?;
        fs::create_dir_all(&target_b)?;

        let link_path = root.join(paths::CURRENT_LINK);
        point_symlink_atomically(&link_path, &target_a)?;
        point_symlink_atomically(&link_path, &target_b)?;

        let linked = fs::read_link(&link_path)?;
        assert_eq!(linked, target_b);

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn current_release_name_resolves_symlink_target_name() -> Result<()> {
        let root = temp_dir_path("current_release_name");
        fs::create_dir_all(&root)?;

        let project_root = project_root_for(&root);
        let release_path = release_dir(&project_root, "20260507_151502");
        fs::create_dir_all(&release_path)?;
        let current = Path::new(&project_root).join(paths::CURRENT_LINK);
        if let Some(parent) = current.parent() {
            fs::create_dir_all(parent)?;
        }
        symlink(&release_path, &current)?;

        assert_eq!(current_release_name(&project_root)?, "20260507_151502");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn list_releases_sorted_skips_placeholder() -> Result<()> {
        let root = temp_dir_path("list_releases");
        fs::create_dir_all(&root)?;

        let project_root = project_root_for(&root);
        fs::create_dir_all(release_dir(&project_root, "20260507_151500"))?;
        fs::create_dir_all(release_dir(&project_root, paths::PLACEHOLDER_RELEASE_NAME))?;
        fs::create_dir_all(release_dir(&project_root, "20260507_151501"))?;

        assert_eq!(list_releases_sorted(&project_root)?, vec!["20260507_151500", "20260507_151501"]);

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
