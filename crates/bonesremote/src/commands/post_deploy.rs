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
