//! Embedded bonesinfra Python framework.
//!
//! The Python package under `python/` is embedded into the binary and
//! materialized on demand into `~/.cache/bonesdeploy/bonesinfra`,
//! where a venv is created and the package installed. A content-hash stamp
//! keeps the materialized copy in sync with the embedded source: any change
//! to the embedded tree triggers a fresh extraction and reinstall.

use std::borrow::Cow;
use std::fs::{self, OpenOptions};
use std::hash::{DefaultHasher, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;
use rust_embed::Embed;

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

/// Runs `python -m bonesinfra` with the given arguments and returns stdout.
///
/// # Errors
/// Fails when the runtime cannot be materialized or the command exits non-zero.
pub fn run_capture(args: &[&str]) -> Result<String> {
    let executable = ensure_available()?;
    let output = base_command(&executable, args)
        .stdin(Stdio::null())
        .output()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !output.status.success() {
        bail!("bonesinfra failed");
    }

    String::from_utf8(output.stdout).context("bonesinfra produced invalid UTF-8 output")
}

fn base_command(executable: &Path, args: &[&str]) -> Command {
    let mut cmd = Command::new(executable);
    cmd.args(["-m", "bonesinfra"]);
    cmd.args(args);
    cmd
}

fn ensure_available() -> Result<PathBuf> {
    ensure_available_in(&checkout_dir())
}

/// Materializes and installs the embedded Python runtime below `cache_root`.
///
/// # Errors
/// Fails when the runtime cannot be materialized or installed.
pub fn prepare_in(cache_root: &Path) -> Result<()> {
    ensure_available_in(&cache_root.join(CHECKOUT_DIR)).map(|_| ())
}

fn ensure_available_in(checkout: &Path) -> Result<PathBuf> {
    let cache_root = checkout.parent().context("bonesinfra checkout has no cache root")?;
    fs::create_dir_all(cache_root).with_context(|| format!("Failed to create {}", cache_root.display()))?;
    let lock_path = cache_root.join(".bonesinfra.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .with_context(|| format!("Failed to open bonesinfra lock at {}", lock_path.display()))?;
    lock.lock().with_context(|| format!("Failed to lock bonesinfra runtime at {}", lock_path.display()))?;

    let venv_python = checkout.join(".venv").join("bin").join("python");
    let stamp = embedded_source_version();

    if venv_python.is_file() && materialized_stamp(&checkout).as_deref() == Some(stamp.as_str()) {
        return Ok(venv_python);
    }

    materialize(checkout)?;
    setup_venv(checkout)?;

    if !venv_python.is_file() {
        bail!("bonesinfra setup finished at {}, but {} is missing.", checkout.display(), venv_python.display());
    }

    // Written last so an interrupted setup re-materializes on the next run.
    fs::write(checkout.join(STAMP_FILE), &stamp)
        .with_context(|| format!("Failed to write bonesinfra stamp in {}", checkout.display()))?;

    Ok(venv_python)
}

/// Returns the paths packaged in the embedded Python distribution.
pub fn embedded_source_paths() -> impl Iterator<Item = Cow<'static, str>> {
    PythonSource::iter()
}

/// Returns the deterministic version identifier for the embedded Python distribution.
pub fn embedded_source_version() -> String {
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

    for file_path in embedded_source_paths() {
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
    paths::bones_cache_root().join(CHECKOUT_DIR)
}
