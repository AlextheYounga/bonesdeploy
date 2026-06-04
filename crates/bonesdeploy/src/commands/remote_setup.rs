use std::env;
use std::fs;
use std::io::{ErrorKind, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use console::style;

use crate::commands::remote_setup_output;
use crate::config;
use crate::embedded;

pub fn run() -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let playbook = Path::new(config::Constants::BONES_REMOTE_SETUP_PLAYBOOK);
    if !playbook.is_file() {
        bail!("Missing remote setup playbook: {}", playbook.display());
    }

    ensure_ansible_playbook_installed()?;

    let ssh_user = resolve_bootstrap_ssh_user();
    let deploy_authorized_key = resolve_deploy_authorized_key()?;

    let extra_args = vec![String::from("-e"), build_extra_var_json("deploy_authorized_key", &deploy_authorized_key)];
    run_ansible_playbook(&cfg, &ssh_user, &extra_args)?;

    println!("{} Remote setup complete.", style("Done!").green().bold());

    Ok(())
}

fn resolve_deploy_authorized_key() -> Result<String> {
    if let Some(path) = env::var("BONES_DEPLOY_PUBLIC_KEY_PATH").ok().filter(|value| !value.trim().is_empty()) {
        return read_public_key(Path::new(path.trim()));
    }

    let home = env::var("HOME").context("HOME is not set; cannot discover SSH public key")?;
    let ssh_dir = Path::new(&home).join(".ssh");
    let candidates = ["id_ed25519.pub", "id_ecdsa.pub", "id_rsa.pub"];

    for candidate in candidates {
        let path = ssh_dir.join(candidate);
        if path.is_file() {
            return read_public_key(&path);
        }
    }

    bail!(
        "No SSH public key found for deploy user setup. Set BONES_DEPLOY_PUBLIC_KEY_PATH or create one of: ~/.ssh/id_ed25519.pub, ~/.ssh/id_ecdsa.pub, ~/.ssh/id_rsa.pub"
    )
}

fn read_public_key(path: &Path) -> Result<String> {
    let key = fs::read_to_string(path).with_context(|| format!("Failed to read SSH public key: {}", path.display()))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        bail!("SSH public key file is empty: {}", path.display());
    }
    Ok(key)
}

fn build_extra_var_json(name: &str, value: &str) -> String {
    format!("{{\"{}\":\"{}\"}}", escape_json_string(name), escape_json_string(value))
}

fn escape_json_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n").replace('\r', "\\r").replace('\t', "\\t")
}

pub(crate) fn resolve_bootstrap_ssh_user() -> String {
    resolve_bootstrap_ssh_user_from(env::var("BONES_BOOTSTRAP_SSH_USER").ok())
}

fn resolve_bootstrap_ssh_user_from(value: Option<String>) -> String {
    value.map(|raw| raw.trim().to_string()).filter(|raw| !raw.is_empty()).unwrap_or_else(|| String::from("root"))
}

pub fn run_ansible_playbook(cfg: &config::BonesConfig, ssh_user: &str, extra_args: &[String]) -> Result<()> {
    remote_setup_output::run(cfg, ssh_user, extra_args)
}

pub(crate) fn ensure_remote_python3_available(cfg: &config::BonesConfig, ssh_user: &str) -> Result<()> {
    let host = format!("{ssh_user}@{}", cfg.data.host);
    let script = embedded::read_asset(config::Constants::PYTHON_BOOTSTRAP_SCRIPT_ASSET)?;

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
    let _ = resolve_ansible_playbook_binary()?;
    Ok(())
}

pub(crate) fn resolve_ansible_playbook_binary() -> Result<PathBuf> {
    if ansible_playbook_available(Path::new("ansible-playbook"))? {
        return Ok(PathBuf::from("ansible-playbook"));
    }

    if let Some(local_ansible_playbook) = user_ansible_playbook_path()?
        && ansible_playbook_available(&local_ansible_playbook)?
    {
        return Ok(local_ansible_playbook);
    }

    ensure_local_python3_available()?;
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
    let status = Command::new(binary).arg("--version").status();
    if status.as_ref().is_err_and(|error| error.kind() == ErrorKind::NotFound) {
        return Ok(false);
    }

    let status = status.with_context(|| format!("Failed to run {} --version", binary.display()))?;
    Ok(status.success())
}

fn ensure_local_python3_available() -> Result<()> {
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
        Err(error) if error.kind() == ErrorKind::NotFound => {
            let _ = error;
            return Ok(None);
        }
        Err(error) => return Err(error).context("Failed to discover python3 user base"),
    };

    let user_base = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if user_base.is_empty() {
        return Ok(None);
    }

    Ok(Some(Path::new(&user_base).join("bin").join("ansible-playbook")))
}

pub(crate) fn resolve_project_root_parent(project_root: &str) -> String {
    Path::new(project_root)
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .map_or_else(|| String::from("/srv/deployments"), |path| path.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::{build_extra_var_json, resolve_bootstrap_ssh_user_from, resolve_project_root_parent};

    // Verifies parent extraction for normal project_root paths used by playbook variables.
    #[test]
    fn resolve_project_root_parent_uses_parent_directory_for_nested_absolute_path() {
        let parent = resolve_project_root_parent("/srv/deployments/my-app/current");
        assert_eq!(parent, "/srv/deployments/my-app");
    }

    // Verifies root-level project_root values keep '/' parent instead of fallback.
    #[test]
    fn resolve_project_root_parent_uses_root_for_single_segment_absolute_path() {
        let parent = resolve_project_root_parent("/my-app");
        assert_eq!(parent, "/");
    }

    // Verifies relative project_root input still derives a usable parent path.
    #[test]
    fn resolve_project_root_parent_uses_relative_parent_for_relative_path() {
        let parent = resolve_project_root_parent("deploy/project");
        assert_eq!(parent, "deploy");
    }

    // Empty input should hit safe fallback to documented default parent.
    #[test]
    fn resolve_project_root_parent_falls_back_for_empty_path() {
        let parent = resolve_project_root_parent("");
        assert_eq!(parent, "/srv/deployments");
    }

    #[test]
    fn resolve_bootstrap_ssh_user_defaults_to_root() {
        let user = resolve_bootstrap_ssh_user_from(None);
        assert_eq!(user, "root");
    }

    #[test]
    fn resolve_bootstrap_ssh_user_uses_env_override() {
        let user = resolve_bootstrap_ssh_user_from(Some(String::from("ubuntu")));
        assert_eq!(user, "ubuntu");
    }

    #[test]
    fn resolve_bootstrap_ssh_user_trims_and_rejects_blank_values() {
        let user = resolve_bootstrap_ssh_user_from(Some(String::from("   ")));
        assert_eq!(user, "root");

        let user = resolve_bootstrap_ssh_user_from(Some(String::from("  ubuntu  ")));
        assert_eq!(user, "ubuntu");
    }

    #[test]
    fn build_extra_var_json_preserves_spaces_in_ssh_public_key() {
        let extra_var =
            build_extra_var_json("deploy_authorized_key", "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFoo comment@host");

        assert_eq!(extra_var, "{\"deploy_authorized_key\":\"ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIFoo comment@host\"}");
    }
}
