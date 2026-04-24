use std::io::ErrorKind;
use std::path::{Path, PathBuf};
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
    let runtime_config_path = format!("{}/bones/bones.toml", cfg.data.git_dir);

    println!(
        "Running {} against {} as {}...",
        style("server setup").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(&cfg.permissions.defaults.deploy_user).cyan(),
    );

    run_ansible_playbook(&cfg, &runtime_config_path, &live_root_parent, &[])?;

    println!("\n{} Server setup complete.", style("Done!").green().bold());

    Ok(())
}

pub fn run_ansible_playbook(
    cfg: &config::BonesConfig,
    runtime_config_path: &str,
    live_root_parent: &str,
    extra_args: &[String],
) -> Result<()> {
    let playbook = Path::new(config::Constants::BONES_SERVER_SETUP_PLAYBOOK);
    if !playbook.is_file() {
        bail!("Missing server setup playbook: {}", playbook.display());
    }

    let inventory = format!("{},", cfg.data.host);

    let ansible_playbook_binary = resolve_ansible_playbook_binary()?;
    let mut command = Command::new(&ansible_playbook_binary);
    command
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
    let _ = resolve_ansible_playbook_binary()?;

    Ok(())
}

fn resolve_ansible_playbook_binary() -> Result<PathBuf> {
    if ansible_playbook_available(Path::new("ansible-playbook"))? {
        return Ok(PathBuf::from("ansible-playbook"));
    }

    if let Some(local_ansible_playbook) = user_ansible_playbook_path()?
        && ansible_playbook_available(&local_ansible_playbook)?
    {
        return Ok(local_ansible_playbook);
    }

    ensure_python3_available()?;
    ensure_pip_available()?;
    install_ansible_with_pip()?;

    if ansible_playbook_available(Path::new("ansible-playbook"))? {
        return Ok(PathBuf::from("ansible-playbook"));
    }

    if let Some(local_ansible_playbook) = user_ansible_playbook_path()?
        && ansible_playbook_available(&local_ansible_playbook)?
    {
        return Ok(local_ansible_playbook);
    }

    bail!("Installed Ansible with pip, but ansible-playbook is still unavailable. Ensure ~/.local/bin is in PATH.")
}

fn ansible_playbook_available(binary: &Path) -> Result<bool> {
    match Command::new(binary).arg("--version").status() {
        Ok(status) => Ok(status.success()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error).with_context(|| format!("Failed to run {} --version", binary.display())),
    }
}

fn ensure_python3_available() -> Result<()> {
    match Command::new("python3").arg("--version").status() {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => bail!("python3 is installed but failed to execute. Install a working Python 3 runtime and retry."),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!("python3 is required to install Ansible. Install python3 and retry.")
        }
        Err(error) => Err(error).context("Failed to run python3 --version"),
    }
}

fn ensure_pip_available() -> Result<()> {
    if python_module_available("pip")? {
        return Ok(());
    }

    let status = Command::new("python3")
        .args(["-m", "ensurepip", "--upgrade"])
        .status()
        .context("Failed to run python3 -m ensurepip --upgrade")?;

    if !status.success() {
        bail!("pip is not available for python3. Install python3-pip and retry.");
    }

    if !python_module_available("pip")? {
        bail!("pip is still unavailable after running ensurepip. Install python3-pip and retry.");
    }

    Ok(())
}

fn python_module_available(module_name: &str) -> Result<bool> {
    let status = Command::new("python3")
        .args(["-m", module_name, "--version"])
        .status()
        .with_context(|| format!("Failed to run python3 -m {module_name} --version"))?;

    Ok(status.success())
}

fn install_ansible_with_pip() -> Result<()> {
    println!(
        "{}",
        style("ansible-playbook not found. Installing Ansible with python3 -m pip install --user ansible...").yellow()
    );

    let status = Command::new("python3")
        .args(["-m", "pip", "install", "--user", "ansible"])
        .status()
        .context("Failed to run python3 -m pip install --user ansible")?;

    if !status.success() {
        bail!("Automatic Ansible installation failed. Run `python3 -m pip install --user ansible` and retry.");
    }

    Ok(())
}

fn user_ansible_playbook_path() -> Result<Option<PathBuf>> {
    let output = Command::new("python3").args(["-c", "import site; print(site.USER_BASE)"]).output();

    let output = match output {
        Ok(output) if output.status.success() => output,
        Ok(_) => return Ok(None),
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("Failed to discover python3 user base"),
    };

    let user_base = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if user_base.is_empty() {
        return Ok(None);
    }

    Ok(Some(Path::new(&user_base).join("bin").join("ansible-playbook")))
}

pub(crate) fn resolve_live_root_parent(live_root: &str) -> String {
    Path::new(live_root)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map_or_else(|| String::from("/var/www"), |path| path.to_string_lossy().to_string())
}
