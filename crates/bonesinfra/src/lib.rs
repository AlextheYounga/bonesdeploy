//! Embedded bonesinfra Python runtime.
//!
//! The Python package under `python/` is embedded into the binary and
//! materialized on demand into `~/.config/bonesdeploy/_lib/bonesinfra`,
//! where a venv is created and the package installed. A content-hash stamp
//! keeps the materialized copy in sync with the embedded source: any change
//! to the embedded tree triggers a fresh extraction and reinstall.

use std::fs;
use std::hash::{DefaultHasher, Hasher};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use rust_embed::Embed;
use shared::paths;

#[derive(Embed)]
#[folder = "python/"]
#[exclude = ".venv/**"]
#[exclude = "**/__pycache__/**"]
#[exclude = "**/*.egg-info/**"]
#[exclude = "docs/**"]
#[exclude = "tests/**"]
struct PythonSource;

const CHECKOUT_DIR: &str = "bonesinfra";
const STAMP_FILE: &str = ".stamp";

/// Runs `python -m bonesinfra` with the given arguments and no stdin.
///
/// # Errors
/// Fails when the runtime cannot be materialized or the command exits non-zero.
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

/// Runs `python -m bonesinfra` with the given arguments, writing `stdin_json` to stdin.
///
/// # Errors
/// Fails when the runtime cannot be materialized or the command exits non-zero.
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
    let mut cmd = Command::new(executable);
    cmd.args(["-m", "bonesinfra"]);
    cmd.args(args);
    cmd
}

fn ensure_available() -> Result<PathBuf> {
    let checkout = checkout_dir();
    let venv_python = checkout.join(".venv").join("bin").join("python");
    let stamp = embedded_stamp();

    if venv_python.is_file() && materialized_stamp(&checkout).as_deref() == Some(stamp.as_str()) {
        return Ok(venv_python);
    }

    materialize(&checkout)?;
    setup_venv(&checkout)?;

    if !venv_python.is_file() {
        bail!("bonesinfra setup finished at {}, but {} is missing.", checkout.display(), venv_python.display());
    }

    // Written last so an interrupted setup re-materializes on the next run.
    fs::write(checkout.join(STAMP_FILE), &stamp)
        .with_context(|| format!("Failed to write bonesinfra stamp in {}", checkout.display()))?;

    Ok(venv_python)
}

fn embedded_stamp() -> String {
    let mut files: Vec<_> = PythonSource::iter().collect();
    files.sort();

    let mut hasher = DefaultHasher::new();
    for file_path in files {
        hasher.write(file_path.as_bytes());
        if let Some(asset) = PythonSource::get(&file_path) {
            hasher.write(asset.data.as_ref());
        }
    }

    format!("{:016x}", hasher.finish())
}

fn materialized_stamp(checkout: &Path) -> Option<String> {
    fs::read_to_string(checkout.join(STAMP_FILE)).ok().map(|s| s.trim().to_string())
}

fn materialize(checkout: &Path) -> Result<()> {
    if let Ok(metadata) = fs::symlink_metadata(checkout) {
        if metadata.file_type().is_dir() {
            fs::remove_dir_all(checkout)
                .with_context(|| format!("Failed to remove stale bonesinfra checkout at {}", checkout.display()))?;
        } else {
            fs::remove_file(checkout)
                .with_context(|| format!("Failed to remove stale bonesinfra checkout at {}", checkout.display()))?;
        }
    }

    fs::create_dir_all(checkout).with_context(|| format!("Failed to create {}", checkout.display()))?;

    for file_path in PythonSource::iter() {
        let Some(asset) = PythonSource::get(&file_path) else {
            continue;
        };

        let dest = checkout.join(file_path.as_ref());
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
        }
        fs::write(&dest, asset.data.as_ref()).with_context(|| format!("Failed to write {}", dest.display()))?;
    }

    Ok(())
}

fn setup_venv(checkout: &Path) -> Result<()> {
    let venv_python = checkout.join(".venv").join("bin").join("python");

    if !venv_python.is_file() {
        let status = Command::new("python3")
            .args(["-m", "venv", ".venv"])
            .current_dir(checkout)
            .status()
            .with_context(|| format!("Failed to create venv in {}", checkout.display()))?;

        if !status.success() {
            bail!("Failed to create venv in {}.", checkout.display());
        }
    }

    let status = Command::new(&venv_python)
        .args(["-m", "pip", "install", "--upgrade", "pip"])
        .status()
        .with_context(|| format!("Failed to upgrade pip in {}", checkout.display()))?;

    if !status.success() {
        bail!("Failed to upgrade pip in {}.", checkout.display());
    }

    let status = Command::new(&venv_python)
        .args(["-m", "pip", "install", "-e", "."])
        .current_dir(checkout)
        .status()
        .with_context(|| format!("Failed to install bonesinfra dependencies in {}", checkout.display()))?;

    if !status.success() {
        bail!("Failed to install bonesinfra dependencies in {}.", checkout.display());
    }

    Ok(())
}

fn checkout_dir() -> PathBuf {
    paths::bones_config_lib_root().join(CHECKOUT_DIR)
}

#[cfg(test)]
mod tests {
    use super::PythonSource;

    #[test]
    fn embeds_the_python_package_and_install_metadata() {
        assert!(PythonSource::get("pyproject.toml").is_some());
        assert!(PythonSource::get("README.md").is_some(), "pyproject readme reference must be embedded");
        assert!(PythonSource::get("src/bonesinfra/__main__.py").is_some());
        assert!(PythonSource::get("src/bonesinfra/runtimes/__init__.py").is_some());
    }

    #[test]
    fn excludes_dev_only_and_derived_trees() {
        for file_path in PythonSource::iter() {
            assert!(
                !file_path.starts_with("docs/")
                    && !file_path.starts_with("tests/")
                    && !file_path.starts_with(".venv/")
                    && !file_path.contains("__pycache__")
                    && !file_path.contains(".egg-info"),
                "unexpected embedded file: {file_path}"
            );
        }
    }

    #[test]
    fn stamp_is_stable_across_calls() {
        assert_eq!(super::embedded_stamp(), super::embedded_stamp());
    }
}
