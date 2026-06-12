use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use crate::config;
use shared::paths::{self, DeploymentPaths};

fn flatten_data_vars(value: &Value, out: &mut Vec<String>) {
    if let Value::Object(map) = value {
        for (key, val) in map {
            flatten_data_entries(&key.clone(), val, out);
        }
    }
}

fn flatten_data_entries(prefix: &str, value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                flatten_data_entries(&format!("{prefix}.{key}"), val, out);
            }
        }
        Value::String(s) => {
            out.push(format!("{prefix}={s}"));
        }
        Value::Number(n) => {
            out.push(format!("{prefix}={n}"));
        }
        Value::Bool(b) => {
            out.push(format!("{prefix}={b}"));
        }
        _ => {}
    }
}

pub struct PyinfraDeploy<'a> {
    pub deploy_file: &'a Path,
    pub extra_args: &'a [String],
}

pub fn run() -> Result<()> {
    let bones_yaml = Path::new(config::Constants::BONES_YAML);
    let cfg = config::load(bones_yaml)?;

    let deploy_file = Path::new(config::Constants::BONES_REMOTE_SETUP_DEPLOY);
    if !deploy_file.is_file() {
        bail!("Missing remote setup deploy file: {}", deploy_file.display());
    }

    ensure_pyinfra_installed()?;

    let ssh_user = resolve_bootstrap_ssh_user();
    let deploy_authorized_key = resolve_deploy_authorized_key()?;

    let data_vars = build_setup_data_vars(&cfg, &deploy_authorized_key)?;
    run_pyinfra_deploy(&cfg, &ssh_user, &data_vars, &PyinfraDeploy { extra_args: &[], deploy_file })?;

    println!("{} Remote setup complete.", style("Done!").green().bold());

    Ok(())
}

fn build_setup_data_vars(cfg: &config::BonesConfig, deploy_authorized_key: &str) -> Result<serde_json::Value> {
    let paths =
        DeploymentPaths::new(&cfg.data.project_name, &cfg.data.repo_path, &cfg.data.project_root, &cfg.data.web_root);
    let mut vars = serde_json::Map::new();

    vars.insert(String::from("ssh_port"), Value::String(cfg.data.port.clone()));
    vars.insert(String::from("deploy_user"), Value::String(String::from(paths::DEPLOY_USER)));
    vars.insert(String::from("service_user"), Value::String(config::service_user(&cfg.data.project_name)));
    vars.insert(String::from("service_group"), Value::String(String::from(paths::DEFAULT_GROUP)));
    vars.insert(String::from("project_root_parent"), Value::String(paths.project_root_parent.clone()));
    vars.insert(String::from("project_root"), Value::String(cfg.data.project_root.clone()));
    vars.insert(String::from("web_root"), Value::String(cfg.data.web_root.clone()));
    vars.insert(String::from("project_name"), Value::String(cfg.data.project_name.clone()));
    vars.insert(String::from("repo_path"), Value::String(cfg.data.repo_path.clone()));
    vars.insert(String::from("paths"), serde_json::to_value(paths)?);
    vars.insert(String::from("deploy_authorized_key"), Value::String(deploy_authorized_key.to_string()));
    vars.insert(String::from("setup_label"), Value::String(String::from("bonesdeploy")));

    Ok(Value::Object(vars))
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

pub(crate) fn resolve_bootstrap_ssh_user() -> String {
    resolve_bootstrap_ssh_user_from(env::var("BONES_BOOTSTRAP_SSH_USER").ok())
}

fn resolve_bootstrap_ssh_user_from(value: Option<String>) -> String {
    value.map(|raw| raw.trim().to_string()).filter(|raw| !raw.is_empty()).unwrap_or_else(|| String::from("root"))
}

pub fn run_pyinfra_deploy(
    cfg: &config::BonesConfig,
    ssh_user: &str,
    data_vars: &serde_json::Value,
    deploy: &PyinfraDeploy<'_>,
) -> Result<()> {
    let pyinfra = resolve_pyinfra_binary()?;

    println!(
        "Running {} against {} as {}...",
        style("pyinfra deploy").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(ssh_user).cyan(),
    );

    let mut data_args: Vec<String> = Vec::new();
    flatten_data_vars(data_vars, &mut data_args);

    let mut command = Command::new(&pyinfra);
    command
        .arg(&cfg.data.host)
        .arg(deploy.deploy_file)
        .arg("--ssh-user")
        .arg(ssh_user)
        .arg("--ssh-port")
        .arg(&cfg.data.port);

    for arg in deploy.extra_args {
        command.arg(arg);
    }

    for data_arg in data_args {
        command.arg("--data").arg(data_arg);
    }

    command.arg("-vv");

    let status = command.status().context("Failed to run pyinfra")?;

    if !status.success() {
        bail!("pyinfra failed with status {status}");
    }

    Ok(())
}

pub(crate) fn ensure_pyinfra_installed() -> Result<()> {
    resolve_pyinfra_binary()?;
    Ok(())
}

pub(crate) fn resolve_pyinfra_binary() -> Result<PathBuf> {
    if pyinfra_available(Path::new("pyinfra"))? {
        return Ok(PathBuf::from("pyinfra"));
    }

    let managed = paths::managed_pyinfra_binary();
    if pyinfra_available(&managed)? {
        return Ok(managed);
    }

    ensure_managed_pyinfra_installed()?;

    if pyinfra_available(&managed)? {
        return Ok(managed);
    }

    bail!("Installed pyinfra into a managed environment but the binary is still unavailable at {}.", managed.display())
}

fn pyinfra_available(binary: &Path) -> Result<bool> {
    let status = Command::new(binary).arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status();
    if status.as_ref().is_err_and(|error| error.kind() == ErrorKind::NotFound) {
        return Ok(false);
    }

    let status = status.with_context(|| format!("Failed to run {} --version", binary.display()))?;
    Ok(status.success())
}

fn ensure_managed_pyinfra_installed() -> Result<()> {
    verify_python3_available()?;
    verify_venv_module_available()?;

    let venv_dir = paths::managed_pyinfra_venv_dir();
    let venv_pip = venv_dir.join("bin").join("pip");
    let venv_pyinfra = paths::managed_pyinfra_binary();

    println!(
        "{}",
        style(format!("pyinfra not found. Installing pyinfra into a managed environment at {}...", venv_dir.display()))
            .yellow()
    );

    if let Some(parent) = venv_dir.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let status = Command::new("python3")
        .args(["-m", "venv", "--clear"])
        .arg(&venv_dir)
        .status()
        .with_context(|| format!("Failed to run python3 -m venv {}", venv_dir.display()))?;

    if !status.success() {
        bail!("Failed to create managed virtualenv at {}", venv_dir.display());
    }

    let status = Command::new(&venv_pip)
        .args(["install", "pyinfra", "PyYAML"])
        .status()
        .with_context(|| format!("Failed to run {} install pyinfra", venv_pip.display()))?;

    if !status.success() {
        bail!(
            "Failed to install pyinfra into the managed environment.\n\
             Install pyinfra manually with `uv tool install pyinfra` or `pipx install pyinfra` and retry."
        );
    }

    if !pyinfra_available(&venv_pyinfra)? {
        bail!(
            "pyinfra was installed into the managed environment but is not usable at {}.\n\
             Install pyinfra manually with `uv tool install pyinfra` or `pipx install pyinfra` and retry.",
            venv_pyinfra.display()
        );
    }

    Ok(())
}

fn verify_python3_available() -> Result<()> {
    match Command::new("python3").arg("--version").status() {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => bail!("python3 is installed but failed to execute. Install a working Python 3 runtime and retry."),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!("python3 is required to install pyinfra. Install python3 and retry.")
        }
        Err(error) => Err(error).context("Failed to run python3 --version"),
    }
}

fn verify_venv_module_available() -> Result<()> {
    match Command::new("python3").args(["-m", "venv", "--help"]).stdout(Stdio::null()).stderr(Stdio::null()).status() {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => bail!(
            "python3 -m venv failed.\n\
             Virtualenv support is missing from your Python installation.\n\
             Install your system's venv package (e.g. `python3-venv` on Debian/Ubuntu) \
             or install pyinfra manually with `uv tool install pyinfra` or `pipx install pyinfra`."
        ),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!(
                "python3 not found. Install Python 3 with venv support (e.g. `python3-venv` on Debian/Ubuntu) \
             or install pyinfra manually with `uv tool install pyinfra` or `pipx install pyinfra`."
            )
        }
        Err(error) => Err(error).context("Failed to run python3 -m venv --help"),
    }
}

#[cfg(test)]
mod tests {
    use super::resolve_bootstrap_ssh_user_from;

    /// Defaults the bootstrap SSH user to root when no override is provided.
    #[test]
    fn resolve_bootstrap_ssh_user_defaults_to_root() {
        let user = resolve_bootstrap_ssh_user_from(None);
        assert_eq!(user, "root");
    }

    /// Uses the environment override when provided for the bootstrap SSH user.
    #[test]
    fn resolve_bootstrap_ssh_user_uses_env_override() {
        let user = resolve_bootstrap_ssh_user_from(Some(String::from("ubuntu")));
        assert_eq!(user, "ubuntu");
    }

    /// Trims whitespace and falls back to root when the bootstrap SSH user is blank.
    #[test]
    fn resolve_bootstrap_ssh_user_trims_and_rejects_blank_values() {
        let user = resolve_bootstrap_ssh_user_from(Some(String::from("   ")));
        assert_eq!(user, "root");

        let user = resolve_bootstrap_ssh_user_from(Some(String::from("  ubuntu  ")));
        assert_eq!(user, "ubuntu");
    }
}
