use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;

use crate::config;

pub fn run() -> Result<()> {
    let bones_toml = Path::new(config::Constants::BONES_TOML);
    let cfg = config::load(bones_toml)?;

    let playbook = Path::new(config::Constants::BONES_SERVER_SETUP_PLAYBOOK);
    if !playbook.is_file() {
        bail!("Missing server setup playbook: {}", playbook.display());
    }

    ensure_ansible_playbook_installed()?;

    let live_root_parent = resolve_live_root_parent(&cfg.data.live_root);

    println!(
        "Running {} against {} as {}...",
        style("server setup").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.permissions.defaults.deploy_user).cyan(),
    );

    let inventory = format!("{},", cfg.data.host);

    let status = Command::new("ansible-playbook")
        .arg("-i")
        .arg(&inventory)
        .arg("-u")
        .arg(&cfg.permissions.defaults.deploy_user)
        .arg("-e")
        .arg(format!("ansible_port={}", cfg.data.port))
        .arg("-e")
        .arg(format!("deploy_user={}", cfg.permissions.defaults.deploy_user))
        .arg("-e")
        .arg(format!("service_user={}", cfg.permissions.defaults.service_user))
        .arg("-e")
        .arg(format!("group={}", cfg.permissions.defaults.group))
        .arg("-e")
        .arg(format!("live_root_parent={live_root_parent}"))
        .arg(playbook)
        .status()
        .context("Failed to run ansible-playbook")?;

    if !status.success() {
        bail!("ansible-playbook failed with status {status}");
    }

    println!("\n{} Server setup complete.", style("Done!").green().bold());

    Ok(())
}

fn ensure_ansible_playbook_installed() -> Result<()> {
    let status = Command::new("ansible-playbook")
        .arg("--version")
        .status()
        .context("Failed to run ansible-playbook --version")?;

    if !status.success() {
        bail!("ansible-playbook is not available. Install Ansible locally and try again.");
    }

    Ok(())
}

fn resolve_live_root_parent(live_root: &str) -> String {
    Path::new(live_root)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map_or_else(|| String::from("/var/www"), |path| path.to_string_lossy().to_string())
}
