use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use shared::paths;

const REPOSITORY_URL: &str = "https://github.com/AlextheYounga/bonesinfra.git";
const CHECKOUT_DIR: &str = "bonesinfra";

pub fn run(args: &[&str]) -> Result<()> {
    let executable = ensure_available()?;
    let mut command = base_command(&executable, args);
    command.stdin(Stdio::null());

    let status = command
        .spawn()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?
        .wait()
        .with_context(|| format!("Failed to wait on bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !status.success() {
        bail!("bonesinfra failed");
    }
    Ok(())
}

#[expect(dead_code)]
pub fn run_with_stdin(args: &[&str], stdin_json: &str) -> Result<()> {
    let executable = ensure_available()?;
    let mut command = base_command(&executable, args);
    command.stdin(Stdio::piped());
    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(stdin_json.as_bytes()).context("Failed to write JSON data to bonesinfra stdin")?;
    }
    let status = child
        .wait()
        .with_context(|| format!("Failed to wait on bonesinfra {} from {}", args.join(" "), executable.display()))?;
    if !status.success() {
        bail!("bonesinfra failed");
    }
    Ok(())
}

fn base_command(executable: &Path, args: &[&str]) -> Command {
    let mut command = Command::new(executable);
    command.args(["-m", "bonesinfra"]).args(args);
    command
}

fn ensure_available() -> Result<PathBuf> {
    let checkout = checkout_dir();
    let venv_python = checkout.join(".venv/bin/python");
    if !checkout.is_dir() {
        install_checkout(&checkout)?;
    }
    if venv_python.is_file() {
        return Ok(venv_python);
    }

    let status = Command::new("python3")
        .args(["-m", "venv", ".venv"])
        .current_dir(&checkout)
        .status()
        .with_context(|| format!("Failed to create venv in {}", checkout.display()))?;
    if !status.success() {
        bail!("Failed to create venv in {}.", checkout.display());
    }

    let status = Command::new(&venv_python)
        .args(["-m", "pip", "install", "--upgrade", "pip"])
        .current_dir(&checkout)
        .status()
        .with_context(|| format!("Failed to upgrade pip in {}", checkout.display()))?;
    if !status.success() {
        bail!("Failed to upgrade pip in {}.", checkout.display());
    }

    let status = Command::new(&venv_python)
        .args(["-m", "pip", "install", "-e", "."])
        .current_dir(&checkout)
        .status()
        .with_context(|| format!("Failed to install bonesinfra dependencies in {}", checkout.display()))?;
    if !status.success() {
        bail!("Failed to install bonesinfra dependencies in {}.", checkout.display());
    }
    Ok(venv_python)
}

fn install_checkout(checkout: &Path) -> Result<()> {
    if let Some(parent) = checkout.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }
    let status = Command::new("git")
        .args(["clone", "--depth", "1", REPOSITORY_URL])
        .arg(checkout)
        .status()
        .context("Failed to run git clone for bonesinfra install")?;
    if !status.success() {
        bail!("Failed to install bonesinfra from {} into {}.", REPOSITORY_URL, checkout.display());
    }
    Ok(())
}

fn checkout_dir() -> PathBuf {
    paths::bones_config_lib_root().join(CHECKOUT_DIR)
}
