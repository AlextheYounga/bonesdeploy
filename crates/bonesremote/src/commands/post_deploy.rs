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
