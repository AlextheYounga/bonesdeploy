use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::permissions;
use crate::privileges;
use crate::release_state;

use shared::config::PathOverride;

#[derive(serde::Deserialize)]
struct RuntimeConfig {
    #[serde(default)]
    permissions: RuntimePermissions,
}

#[derive(serde::Deserialize, Default)]
struct RuntimePermissions {
    #[serde(default)]
    defaults: RuntimePermissionDefaults,
    #[serde(default)]
    paths: Vec<PathOverride>,
}

#[derive(serde::Deserialize)]
struct RuntimePermissionDefaults {
    #[serde(default = "default_dir_mode")]
    dir_mode: String,
    #[serde(default = "default_file_mode")]
    file_mode: String,
}

impl Default for RuntimePermissionDefaults {
    fn default() -> Self {
        Self { dir_mode: default_dir_mode(), file_mode: default_file_mode() }
    }
}

fn default_dir_mode() -> String {
    "750".into()
}

fn default_file_mode() -> String {
    "640".into()
}

fn load_runtime_permissions(config_path: &Path) -> Result<(String, String, Vec<PathOverride>)> {
    let runtime_path = config_path.parent().unwrap_or(Path::new(".")).join("runtime.yaml");
    if !runtime_path.exists() {
        return Ok((default_dir_mode(), default_file_mode(), Vec::new()));
    }
    let content =
        fs::read_to_string(&runtime_path).with_context(|| format!("Failed to read {}", runtime_path.display()))?;
    let rt: RuntimeConfig =
        serde_yml::from_str(&content).with_context(|| format!("Failed to parse {}", runtime_path.display()))?;
    Ok((rt.permissions.defaults.dir_mode, rt.permissions.defaults.file_mode, rt.permissions.paths))
}

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_root("bonesremote hooks post-deploy")?;

    let config_path = Path::new(config_path);
    let cfg = config::load(config_path)?;
    restart_site_nginx(&cfg)?;

    let (dir_mode, file_mode, path_overrides) = load_runtime_permissions(config_path)?;
    permissions::harden_active_release(&cfg, &dir_mode, &file_mode, &path_overrides)?;

    let pruned = prune_old_releases(&cfg)?;
    if !pruned.is_empty() {
        println!("Pruned releases: {}", pruned.join(", "));
    }

    Ok(())
}

fn restart_site_nginx(cfg: &config::BonesConfig) -> Result<()> {
    let service_name = format!("{}-nginx", cfg.data.project_name);
    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", &service_name])
        .status()
        .context("Failed to check nginx service status")?;

    if status.success() {
        let restart_status = Command::new("systemctl")
            .args(["restart", &service_name])
            .status()
            .context("Failed to restart nginx service")?;

        if !restart_status.success() {
            bail!("Failed to restart {service_name} service");
        }
        println!("Restarted {service_name} service");
    }

    Ok(())
}

fn prune_old_releases(cfg: &config::BonesConfig) -> Result<Vec<String>> {
    let active_release = release_state::current_release_name(cfg)?;
    let mut releases = release_state::list_releases_sorted(cfg)?;
    let keep = cfg.releases.keep.max(1);

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

    use super::prune_old_releases;
    use crate::config;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn config_for(temp_root: &Path, keep: usize) -> config::BonesConfig {
        config::BonesConfig {
            data: config::Data {
                remote_name: String::from("production"),
                project_name: String::from("acme"),
                host: String::from("example.com"),
                port: String::from("22"),
                repo_path: temp_root.join("repo.git").to_string_lossy().to_string(),
                project_root: temp_root.join("project_root").to_string_lossy().to_string(),
                web_root: String::from("public"),
                branch: String::from("main"),
                deploy_on_push: true,
            },
            releases: config::Releases { keep },
            shared: config::Shared {
                shared_files: vec![String::from(".env")],
                shared_dirs: vec![String::from("storage")],
            },
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

    /// Prunes the oldest inactive releases when the active release count exceeds the keep limit.
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

    /// Keeps all releases when the active release count is within the keep limit.
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
