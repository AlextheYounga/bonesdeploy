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

    println!(
        "Running {} against {} as {}...",
        style("server setup").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.permissions.defaults.deploy_user).cyan(),
    );

    run_ansible_playbook(&cfg, &cfg.permissions.defaults.deploy_user, &[])?;

    println!("\n{} Server setup complete.", style("Done!").green().bold());

    Ok(())
}

pub fn run_ansible_playbook(cfg: &config::BonesConfig, ssh_user: &str, extra_args: &[String]) -> Result<()> {
    let playbook = Path::new(config::Constants::BONES_SERVER_SETUP_PLAYBOOK);
    if !playbook.is_file() {
        bail!("Missing server setup playbook: {}", playbook.display());
    }

    let live_root_parent = resolve_live_root_parent(&cfg.data.live_root);
    let runtime_config_path = format!("{}/bones/bones.toml", cfg.data.git_dir);

    let inventory = format!("{},", cfg.data.host);

    let mut command = Command::new("ansible-playbook");
    command
        .arg("-i")
        .arg(&inventory)
        .arg("-u")
        .arg(ssh_user)
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
        .arg("-e")
        .arg(format!("live_root={}", cfg.data.live_root))
        .arg("-e")
        .arg(format!("project_name={}", cfg.data.project_name))
        .arg("-e")
        .arg(format!("git_dir={}", cfg.data.git_dir))
        .arg("-e")
        .arg(format!("runtime_config_path={runtime_config_path}"));

    if cfg.ssl.enabled && !cfg.ssl.domain.is_empty() {
        command
            .arg("-e")
            .arg(format!("nginx_server_name={}", cfg.ssl.domain))
            .arg("-e")
            .arg("nginx_ssl_enabled=true")
            .arg("-e")
            .arg(format!("nginx_ssl_certificate_path=/etc/letsencrypt/live/{}/fullchain.pem", cfg.ssl.domain))
            .arg("-e")
            .arg(format!("nginx_ssl_certificate_key_path=/etc/letsencrypt/live/{}/privkey.pem", cfg.ssl.domain));
    }

    command.args(extra_args);
    command.arg(playbook);

    let status = command.status().context("Failed to run ansible-playbook")?;

    if !status.success() {
        bail!("ansible-playbook failed with status {status}");
    }

    Ok(())
}

pub(crate) fn ensure_ansible_playbook_installed() -> Result<()> {
    let status = Command::new("ansible-playbook")
        .arg("--version")
        .status()
        .context("Failed to run ansible-playbook --version")?;

    if !status.success() {
        bail!("ansible-playbook is not available. Install Ansible locally and try again.");
    }

    Ok(())
}

pub(crate) fn resolve_live_root_parent(live_root: &str) -> String {
    Path::new(live_root)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map_or_else(|| String::from("/var/www"), |path| path.to_string_lossy().to_string())
}
