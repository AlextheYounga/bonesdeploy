use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result, bail};

use crate::config::{BonesConfig, Constants};
use shared::paths::DeploymentPaths;

fn deployment_paths(cfg: &BonesConfig) -> DeploymentPaths {
    DeploymentPaths::new(&cfg.data.project_name, &cfg.data.repo_path, &cfg.data.project_root, &cfg.data.web_root)
}

pub fn staged_release_path(cfg: &BonesConfig) -> PathBuf {
    Path::new(&deployment_paths(cfg).repo_bones).join(Constants::STAGED_RELEASE_FILE)
}

pub fn read_staged_release(cfg: &BonesConfig) -> Result<String> {
    let path = staged_release_path(cfg);
    let value = fs::read_to_string(&path)
        .with_context(|| format!("Failed to read staged release state at {}", path.display()))?;
    let release = value.trim().to_string();

    if release.is_empty() {
        bail!("Staged release state file is empty: {}", path.display());
    }

    Ok(release)
}

pub fn write_staged_release(cfg: &BonesConfig, release: &str) -> Result<()> {
    let path = staged_release_path(cfg);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create staged release state dir: {}", parent.display()))?;
    }

    fs::write(&path, format!("{release}\n"))
        .with_context(|| format!("Failed to write staged release state: {}", path.display()))
}

pub fn clear_staged_release(cfg: &BonesConfig) -> Result<()> {
    let path = staged_release_path(cfg);
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("Failed to remove staged release state: {}", path.display()))?;
    }
    Ok(())
}

pub fn release_dir(cfg: &BonesConfig, release: &str) -> PathBuf {
    releases_dir(cfg).join(release)
}

pub fn releases_dir(cfg: &BonesConfig) -> PathBuf {
    PathBuf::from(deployment_paths(cfg).releases)
}

pub fn build_root(cfg: &BonesConfig) -> PathBuf {
    PathBuf::from(deployment_paths(cfg).build_root)
}

pub fn shared_dir(cfg: &BonesConfig) -> PathBuf {
    PathBuf::from(deployment_paths(cfg).shared)
}

pub fn current_link(cfg: &BonesConfig) -> PathBuf {
    PathBuf::from(deployment_paths(cfg).current)
}

pub fn current_release_dir(cfg: &BonesConfig) -> Result<PathBuf> {
    let current_link = current_link(cfg);
    let active_target =
        fs::read_link(&current_link).with_context(|| format!("Failed to read {}", current_link.display()))?;

    Ok(if active_target.is_absolute() {
        active_target
    } else {
        current_link.parent().unwrap_or_else(|| Path::new("/")).join(active_target)
    })
}

pub fn current_release_name(cfg: &BonesConfig) -> Result<String> {
    let current_release = current_release_dir(cfg)?;
    current_release
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .ok_or_else(|| anyhow::anyhow!("Failed to resolve current release name from {}", current_release.display()))
}

pub fn list_releases_sorted(cfg: &BonesConfig) -> Result<Vec<String>> {
    let releases_dir = releases_dir(cfg);
    if !releases_dir.exists() {
        return Ok(Vec::new());
    }

    let mut names = Vec::new();
    for entry in fs::read_dir(&releases_dir)
        .with_context(|| format!("Failed to read releases dir: {}", releases_dir.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            names.push(entry.file_name().to_string_lossy().to_string());
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

    // Generate unique temp symlink name to avoid collisions in concurrent deployments
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
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use shared::paths;

    use crate::config::{BonesConfig, Data, Releases};

    use super::{
        clear_staged_release, current_link, current_release_name, list_releases_sorted, point_symlink_atomically,
        read_staged_release, releases_dir, staged_release_path, write_staged_release,
    };

    fn temp_dir_path(test_name: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
        env::temp_dir().join(format!("bonesremote_release_state_test_{}_{}_{}", process::id(), nanos, test_name))
    }

    fn sample_config(root: &Path) -> BonesConfig {
        let project = "unitapp";
        BonesConfig {
            data: Data {
                remote_name: String::from("production"),
                project_name: String::from(project),
                host: String::from("deploy.example.com"),
                port: String::from("22"),
                repo_path: root.join("repo.git").to_string_lossy().to_string(),
                project_root: root.join("deploy").to_string_lossy().to_string(),
                web_root: String::from("public"),
                branch: String::from("master"),
                deploy_on_push: true,
            },
            releases: Releases { keep: 5 },
        }
    }

    /// Round-trips a staged release name through write and read.
    #[test]
    fn write_then_read_staged_release_round_trips() -> Result<()> {
        let root = temp_dir_path("round_trip");
        fs::create_dir_all(&root)?;
        let cfg = sample_config(&root);

        write_staged_release(&cfg, "20260507_151500")?;
        let release_name = read_staged_release(&cfg)?;
        assert_eq!(release_name, "20260507_151500");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Returns an error when the staged release file is empty.
    #[test]
    fn read_staged_release_rejects_empty_file() -> Result<()> {
        let root = temp_dir_path("empty_state");
        fs::create_dir_all(&root)?;
        let cfg = sample_config(&root);

        let state_path = staged_release_path(&cfg);
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&state_path, " \n")?;

        let result = read_staged_release(&cfg);
        assert!(result.is_err());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Removes the staged release state file from disk.
    #[test]
    fn clear_staged_release_removes_state_file() -> Result<()> {
        let root = temp_dir_path("clear_state");
        fs::create_dir_all(&root)?;
        let cfg = sample_config(&root);

        write_staged_release(&cfg, "20260507_151501")?;
        clear_staged_release(&cfg)?;
        assert!(!staged_release_path(&cfg).exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Creates parent directories and atomically points a symlink to its target.
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

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Atomically repoints an existing symlink to a new target.
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

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Returns only directories sorted chronologically, excluding files.
    #[test]
    fn list_releases_sorted_returns_only_directories_in_order() -> Result<()> {
        let root = temp_dir_path("list_releases");
        fs::create_dir_all(&root)?;
        let cfg = sample_config(&root);

        let releases = releases_dir(&cfg);
        fs::create_dir_all(&releases)?;
        fs::create_dir_all(releases.join("20260507_120000"))?;
        fs::create_dir_all(releases.join("20260507_110000"))?;
        fs::write(releases.join("notes.txt"), "not a release")?;

        let items = list_releases_sorted(&cfg)?;
        assert_eq!(items, vec![String::from("20260507_110000"), String::from("20260507_120000")]);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Resolves the current release name from the `current` symlink target.
    #[test]
    fn current_release_name_resolves_from_current_symlink() -> Result<()> {
        let root = temp_dir_path("current_release_name");
        fs::create_dir_all(&root)?;
        let cfg = sample_config(&root);

        let releases_dir = releases_dir(&cfg);
        let release = releases_dir.join("20260507_170000");
        fs::create_dir_all(&release)?;

        let current = current_link(&cfg);
        point_symlink_atomically(&current, &release)?;

        let name = current_release_name(&cfg)?;
        assert_eq!(name, "20260507_170000");

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
