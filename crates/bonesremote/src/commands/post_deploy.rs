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
    ensure_runtime_service(&cfg)?;
    permissions::harden_active_release(&cfg)?;

    let pruned = prune_old_releases(&cfg)?;
    if !pruned.is_empty() {
        println!("Pruned releases: {}", pruned.join(", "));
    }

    Ok(())
}

fn ensure_runtime_service(cfg: &config::BonesConfig) -> Result<()> {
    if cfg.runtime.command.is_empty() {
        return Ok(());
    }

    let service_path = format!("/etc/systemd/system/{}.service", cfg.data.project_name);
    let service_body = render_runtime_service(cfg);
    let changed = write_file_if_changed(Path::new(&service_path), &service_body)?;

    if changed {
        run_systemctl(["daemon-reload"])?;
    }

    run_systemctl(["enable", "--now", &cfg.data.project_name])?;
    Ok(())
}

fn render_runtime_service(cfg: &config::BonesConfig) -> String {
    let runtime_config_path = format!("{}/bones/bones.yaml", cfg.data.git_dir);
    format!(
        "[Unit]\nDescription=Bones runtime for {service_name}\nAfter=network.target\n\n[Service]\nType=simple\nUser={service_user}\nWorkingDirectory={working_directory}\nExecStart=/usr/local/bin/bonesremote landlock exec --config {runtime_config_path}\nRestart=always\nRestartSec=2\n\n[Install]\nWantedBy=multi-user.target\n",
        service_name = cfg.data.project_name,
        service_user = cfg.permissions.defaults.service_user,
        working_directory = cfg.data.live_root,
        runtime_config_path = runtime_config_path,
    )
}

fn write_file_if_changed(path: &Path, contents: &str) -> Result<bool> {
    if path.exists() {
        let existing = fs::read_to_string(path).with_context(|| format!("Failed to read {}", path.display()))?;
        if existing == contents {
            return Ok(false);
        }
    }

    fs::write(path, contents).with_context(|| format!("Failed to write {}", path.display()))?;
    Ok(true)
}

fn run_systemctl<'a>(args: impl IntoIterator<Item = &'a str>) -> Result<()> {
    let status = Command::new("systemctl").args(args).status().context("Failed to run systemctl")?;

    if !status.success() {
        bail!("systemctl command failed with status {status}");
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

    use super::{prune_old_releases, render_runtime_service, write_file_if_changed};
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
            runtime: config::Runtime {
                command: vec![String::from("bun"), String::from("run"), String::from("start")],
                working_dir: String::from("."),
                writable_paths: Vec::new(),
            },
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

    // Verifies generated service unit includes the runtime identity and config path contract.
    #[test]
    fn render_runtime_service_includes_expected_runtime_fields() {
        let root = temp_dir("bonesremote_post_deploy_service");
        let cfg = config_for(&root, 5);

        let service = render_runtime_service(&cfg);

        assert!(service.contains("Description=Bones runtime for acme"));
        assert!(service.contains("User=svc-acme"));
        assert!(service.contains(&format!("WorkingDirectory={}", cfg.data.live_root)));
        assert!(service.contains(&format!("--config {}/bones/bones.yaml", cfg.data.git_dir)));

        fs::remove_dir_all(root).ok();
    }

    // Prevents unnecessary daemon-reload churn by asserting no-op writes are detected.
    #[test]
    fn write_file_if_changed_reports_false_when_contents_are_unchanged() {
        let root = temp_dir("bonesremote_post_deploy_write_unchanged");
        let file_path = root.join("service.unit");
        fs::write(&file_path, "same").unwrap_or_else(|error| panic!("failed to seed file: {error}"));

        let changed = write_file_if_changed(&file_path, "same")
            .unwrap_or_else(|error| panic!("write_file_if_changed failed: {error}"));

        assert!(!changed);
        fs::remove_dir_all(root).ok();
    }

    // Ensures changed service content is persisted so runtime updates actually take effect.
    #[test]
    fn write_file_if_changed_reports_true_when_contents_change() {
        let root = temp_dir("bonesremote_post_deploy_write_changed");
        let file_path = root.join("service.unit");
        fs::write(&file_path, "before").unwrap_or_else(|error| panic!("failed to seed file: {error}"));

        let changed = write_file_if_changed(&file_path, "after")
            .unwrap_or_else(|error| panic!("write_file_if_changed failed: {error}"));

        assert!(changed);
        assert_eq!(
            fs::read_to_string(&file_path).unwrap_or_else(|error| panic!("failed to read file: {error}")),
            "after"
        );

        fs::remove_dir_all(root).ok();
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
