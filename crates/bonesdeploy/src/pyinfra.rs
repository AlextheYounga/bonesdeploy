use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use console::style;
use serde_json::Value;

use crate::config::BonesConfig;
use shared::paths;

pub struct PyinfraDeploy<'a> {
    pub deploy_file: &'a Path,
    pub extra_args: &'a [String],
}

pub fn ensure_pyinfra_installed() -> Result<()> {
    resolve_pyinfra_binary()?;
    Ok(())
}

pub fn run_pyinfra_deploy(
    config: &BonesConfig,
    ssh_user: &str,
    deploy_data: &Value,
    deploy: &PyinfraDeploy<'_>,
) -> Result<()> {
    let pyinfra = resolve_pyinfra_binary()?;

    println!(
        "Running {} against {} as {}...",
        style("pyinfra deploy").cyan().bold(),
        style(&config.data.host).cyan(),
        style(ssh_user).cyan(),
    );

    let mut data_args: Vec<String> = Vec::new();
    flatten_data(deploy_data, &mut data_args);

    let mut command = Command::new(&pyinfra);
    command
        .arg(&config.data.host)
        .arg(deploy.deploy_file)
        .arg("--ssh-user")
        .arg(ssh_user)
        .arg("--ssh-port")
        .arg(&config.data.port);

    for arg in deploy.extra_args {
        command.arg(arg);
    }

    for data_arg in data_args {
        command.arg("--data").arg(data_arg);
    }

    let status = command.status().context("Failed to run pyinfra")?;

    if !status.success() {
        bail!("pyinfra failed with status {status}");
    }

    Ok(())
}

fn flatten_data(value: &Value, out: &mut Vec<String>) {
    if let Value::Object(map) = value {
        for (key, val) in map {
            flatten_entries(key, val, out);
        }
    }
}

fn flatten_entries(prefix: &str, value: &Value, out: &mut Vec<String>) {
    match value {
        Value::Object(map) => {
            for (key, val) in map {
                flatten_entries(&format!("{prefix}.{key}"), val, out);
            }
        }
        Value::String(s) => out.push(format!("{prefix}={s}")),
        Value::Number(n) => out.push(format!("{prefix}={n}")),
        Value::Bool(b) => out.push(format!("{prefix}={b}")),
        _ => {}
    }
}

fn resolve_pyinfra_binary() -> Result<PathBuf> {
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
