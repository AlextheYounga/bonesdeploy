use std::env;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use crate::config;
use shared::paths::DeploymentPaths;

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
    vars.insert(String::from("deploy_user"), Value::String(cfg.permissions.defaults.deploy_user.clone()));
    vars.insert(String::from("service_user"), Value::String(cfg.permissions.defaults.service_user.clone()));
    vars.insert(String::from("group"), Value::String(cfg.permissions.defaults.group.clone()));
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
    let inventory_file = write_inventory_file(cfg, ssh_user)?;
    let data_file = write_data_file(data_vars)?;

    let pyinfra = resolve_pyinfra_binary()?;

    println!(
        "Running {} against {} as {}...",
        style("pyinfra deploy").cyan().bold(),
        style(&cfg.data.host).cyan(),
        style(ssh_user).cyan(),
    );

    let mut command = Command::new(&pyinfra);
    command.arg(&inventory_file).arg(deploy.deploy_file);

    for arg in deploy.extra_args {
        command.arg(arg);
    }

    command.arg("--data").arg(&data_file);
    command.arg("-vv");

    let status = command.status().context("Failed to run pyinfra")?;

    if !status.success() {
        bail!("pyinfra failed with status {status}");
    }

    Ok(())
}

fn write_inventory_file(cfg: &config::BonesConfig, _ssh_user: &str) -> Result<PathBuf> {
    let temp = tempfile::NamedTempFile::new().context("Failed to create temp inventory file")?;
    let content = format!(
        r"# Auto-generated inventory
{host}
",
        host = cfg.data.host
    );
    fs::write(temp.path(), content).context("Failed to write inventory file")?;
    Ok(temp.path().to_path_buf())
}

fn write_data_file(data_vars: &serde_json::Value) -> Result<PathBuf> {
    let temp = tempfile::NamedTempFile::new().context("Failed to create temp data file")?;
    let content = serde_json::to_string_pretty(data_vars).context("Failed to serialize data vars")?;
    fs::write(temp.path(), content).context("Failed to write data file")?;
    Ok(temp.path().to_path_buf())
}

pub(crate) fn ensure_pyinfra_installed() -> Result<()> {
    resolve_pyinfra_binary()?;
    Ok(())
}

pub(crate) fn resolve_pyinfra_binary() -> Result<PathBuf> {
    if pyinfra_available(Path::new("pyinfra"))? {
        return Ok(PathBuf::from("pyinfra"));
    }

    if let Some(local_pyinfra) = user_pyinfra_path()?
        && pyinfra_available(&local_pyinfra)?
    {
        return Ok(local_pyinfra);
    }

    ensure_local_python3_available()?;
    ensure_pip_available()?;
    install_pyinfra_with_pip()?;

    if pyinfra_available(Path::new("pyinfra"))? {
        return Ok(PathBuf::from("pyinfra"));
    }

    if let Some(local_pyinfra) = user_pyinfra_path()?
        && pyinfra_available(&local_pyinfra)?
    {
        return Ok(local_pyinfra);
    }

    bail!("Installed pyinfra with pip, but pyinfra is still unavailable. Ensure ~/.local/bin is in PATH.")
}

fn pyinfra_available(binary: &Path) -> Result<bool> {
    let status = Command::new(binary).arg("--version").stdout(Stdio::null()).stderr(Stdio::null()).status();
    if status.as_ref().is_err_and(|error| error.kind() == ErrorKind::NotFound) {
        return Ok(false);
    }

    let status = status.with_context(|| format!("Failed to run {} --version", binary.display()))?;
    Ok(status.success())
}

fn install_pyinfra_with_pip() -> Result<()> {
    println!(
        "{}",
        style("pyinfra not found. Installing pyinfra with python3 -m pip install --user pyinfra...").yellow()
    );

    let status = Command::new("python3")
        .args(["-m", "pip", "install", "--user", "pyinfra"])
        .status()
        .context("Failed to run python3 -m pip install --user pyinfra")?;

    if !status.success() {
        bail!("Automatic pyinfra installation failed. Run `python3 -m pip install --user pyinfra` and retry.");
    }

    Ok(())
}

fn user_pyinfra_path() -> Result<Option<PathBuf>> {
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

    Ok(Some(Path::new(&user_base).join("bin").join("pyinfra")))
}

fn ensure_local_python3_available() -> Result<()> {
    match Command::new("python3").arg("--version").status() {
        Ok(status) if status.success() => Ok(()),
        Ok(_) => bail!("python3 is installed but failed to execute. Install a working Python 3 runtime and retry."),
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!("python3 is required to install pyinfra. Install python3 and retry.")
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
