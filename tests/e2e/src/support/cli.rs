use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::OnceLock;

use anyhow::{Context, Result, bail};

use crate::support::paths;

pub fn run_bonesdeploy<I, S>(cwd: &Path, args: I) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    run_bonesdeploy_with_env(cwd, args, std::iter::empty::<(OsString, OsString)>())
}

pub fn run_bonesdeploy_with_env<I, S, E, K, V>(cwd: &Path, args: I, envs: E) -> Result<Output>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
    E: IntoIterator<Item = (K, V)>,
    K: AsRef<OsStr>,
    V: AsRef<OsStr>,
{
    let binary = bonesdeploy_binary_path()?;
    let status = Command::new(binary)
        .args(args)
        .envs(envs)
        .current_dir(cwd)
        .output()
        .context("Failed to execute bonesdeploy binary")?;

    Ok(status)
}

fn bonesdeploy_binary_path() -> Result<PathBuf> {
    static READY: OnceLock<Result<(), String>> = OnceLock::new();
    let build_result = READY.get_or_init(|| {
        let output = Command::new("cargo")
            .arg("build")
            .arg("-q")
            .arg("-p")
            .arg("bonesdeploy")
            .current_dir(paths::workspace_root())
            .output()
            .map_err(|error| format!("Failed to run cargo build for bonesdeploy: {error}"))?;

        if output.status.success() {
            return Ok(());
        }

        Err(format!(
            "cargo build -p bonesdeploy failed.\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ))
    });

    if let Err(message) = build_result {
        bail!("{message}");
    }

    let binary = paths::workspace_root().join("target/debug/bonesdeploy");
    if !binary.is_file() {
        bail!("bonesdeploy binary is missing at {}", binary.display());
    }

    Ok(binary)
}

pub fn assert_success(output: &Output) -> Result<()> {
    if output.status.success() {
        return Ok(());
    }

    bail!(
        "Command unexpectedly failed.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub fn assert_failure(output: &Output) -> Result<()> {
    if !output.status.success() {
        return Ok(());
    }

    bail!(
        "Command unexpectedly succeeded.\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub fn assert_stdout_contains(output: &Output, expected: &str) -> Result<()> {
    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains(expected) {
        return Ok(());
    }

    bail!("Expected stdout to include '{expected}', got:\n{stdout}")
}
