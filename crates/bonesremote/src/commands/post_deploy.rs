use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::permissions;
use crate::privileges;
use crate::release_state;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_root("bonesremote hooks post-deploy")?;

    let cfg = config::load(Path::new(config_path))?;
    restart_site_nginx(&cfg)?;
    permissions::harden_active_release(&cfg)?;

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
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::prune_old_releases;
    use crate::config;

    fn temp_dir(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), nanos));
        fs::create_dir_all(&path).unwrap_or_else(|error| panic!("failed to create temp dir: {error}"));
        path
    }

    fn config_for(temp_root: &std::path::Path, keep: usize) -> config::BonesConfig {
        config::BonesConfig {
            data: config::Data {
                remote_name: String::from("production"),
                project_name: String::from("acme"),
                host: String::from("example.com"),
                port: String::from("22"),
                git_dir: temp_root.join("repo.git").to_string_lossy().to_string(),
                live_root: temp_root.join("live_root").to_string_lossy().to_string(),
                deploy_root: temp_root.join("deploy_root").to_string_lossy().to_string(),
                branch: String::from("main"),
                deploy_on_push: true,
            },
            permissions: config::Permissions {
                defaults: config::PermissionDefaults {
                    deploy_user: String::from("git"),
                    service_user: String::from("svc-acme"),
                    group: String::from("www-data"),
                    dir_mode: String::from("750"),
                    file_mode: String::from("640"),
                },
                paths: Vec::new(),
            },
            releases: config::Releases { keep, shared_paths: vec![String::from(".env")] },
        }
    }

    fn make_release(root: &std::path::Path, name: &str) {
        fs::create_dir_all(root.join("deploy_root/runtime").join(name))
            .unwrap_or_else(|error| panic!("failed to create release dir: {error}"));
    }

    fn set_current_release(root: &std::path::Path, name: &str) {
        let deploy_root = root.join("deploy_root");
        let runtime = deploy_root.join("runtime");
        fs::create_dir_all(&runtime).unwrap_or_else(|error| panic!("failed to create runtime dir: {error}"));
        let target = runtime.join(name);
        std::os::unix::fs::symlink(&target, deploy_root.join("current"))
            .unwrap_or_else(|error| panic!("failed to create current symlink: {error}"));
    }

    // Verifies retention policy prunes only the oldest inactive releases beyond keep count.
    #[test]
    fn prune_old_releases_removes_oldest_inactive_releases_up_to_keep_limit() {
        let root = temp_dir("bonesremote_post_deploy_prune");
        let cfg = config_for(&root, 2);

        make_release(&root, "20260101_000000");
        make_release(&root, "20260102_000000");
        make_release(&root, "20260103_000000");
        set_current_release(&root, "20260103_000000");

        let pruned = prune_old_releases(&cfg).unwrap_or_else(|error| panic!("prune_old_releases failed: {error}"));

        assert_eq!(pruned, vec!["20260101_000000"]);
        assert!(!root.join("deploy_root/runtime/20260101_000000").exists());
        assert!(root.join("deploy_root/runtime/20260102_000000").exists());
        assert!(root.join("deploy_root/runtime/20260103_000000").exists());

        fs::remove_dir_all(root).ok();
    }

    // Verifies active release is preserved when retention limit is already satisfied.
    #[test]
    fn prune_old_releases_keeps_active_release_when_within_keep_limit() {
        let root = temp_dir("bonesremote_post_deploy_prune_active");
        let cfg = config_for(&root, 2);

        make_release(&root, "20260101_000000");
        make_release(&root, "20260102_000000");
        set_current_release(&root, "20260101_000000");

        let pruned = prune_old_releases(&cfg).unwrap_or_else(|error| panic!("prune_old_releases failed: {error}"));

        assert!(pruned.is_empty());
        assert!(root.join("deploy_root/runtime/20260101_000000").exists());
        assert!(root.join("deploy_root/runtime/20260102_000000").exists());

        fs::remove_dir_all(root).ok();
    }
}
