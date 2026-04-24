use std::env;
use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use console::style;

use crate::config;
use crate::embedded;

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

    let roles_dir = Path::new(config::Constants::BONES_SERVER_ROLES_DIR);
    if !roles_dir.is_dir() {
        bail!("Missing server roles directory: {}", roles_dir.display());
    }

    ensure_python3_available(cfg, ssh_user)?;

    let live_root_parent = resolve_live_root_parent(&cfg.data.live_root);
    let runtime_config_path = format!("{}/bones/bones.toml", cfg.data.git_dir);

    let inventory = format!("{},", cfg.data.host);
    let roles_path = env::var("ANSIBLE_ROLES_PATH")
        .ok()
        .filter(|value| !value.is_empty())
        .map_or_else(|| roles_dir.display().to_string(), |existing| format!("{}:{existing}", roles_dir.display()));

    let mut command = Command::new("ansible-playbook");
    command.env("ANSIBLE_ROLES_PATH", roles_path);
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

fn ensure_python3_available(cfg: &config::BonesConfig, ssh_user: &str) -> Result<()> {
    let host = format!("{ssh_user}@{}", cfg.data.host);
    let script = embedded::read_asset(config::Constants::PYTHON_BOOTSTRAP_SCRIPT_ASSET)?;

    println!("Ensuring python3 is available on remote host...");

    let mut child = Command::new("ssh")
        .arg("-p")
        .arg(&cfg.data.port)
        .arg("-o")
        .arg("StrictHostKeyChecking=accept-new")
        .arg("-T")
        .arg(host)
        .arg("bash -s")
        .stdin(Stdio::piped())
        .spawn()
        .context("Failed to start remote python3 bootstrap command over SSH")?;

    let mut stdin = child.stdin.take().context("Failed to open stdin for SSH process")?;
    stdin.write_all(script.as_bytes()).context("Failed to send python3 bootstrap script over SSH")?;
    drop(stdin);

    let status = child.wait().context("Failed to run remote python3 bootstrap command over SSH")?;

    if !status.success() {
        bail!("Failed to ensure python3 is installed on the remote host");
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
