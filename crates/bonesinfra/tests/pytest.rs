//! Runs the embedded Python package's pytest suite as part of `cargo test`.
//!
//! The venv lives under the cargo target directory so it never collides with a
//! developer's own venv in `python/` and never leaks into the rust-embed assets.
//! Set `BONES_SKIP_PYTEST=1` to skip during tight Rust-only iteration loops.

use std::env;
use std::fs;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use anyhow::{Context, Result, bail};

const SKIP_ENV: &str = "BONES_SKIP_PYTEST";

#[test]
fn python_test_suite_passes() -> Result<()> {
    if env::var_os(SKIP_ENV).is_some() {
        eprintln!("skipping Python test suite: {SKIP_ENV} is set");
        return Ok(());
    }

    let python_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("python");
    let venv = venv_dir();
    ensure_venv(&python_dir, &venv)?;

    let status = Command::new(venv_python(&venv))
        .current_dir(&python_dir)
        .args(["-m", "pytest"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to start pytest")?;

    if !status.success() {
        bail!("pytest suite failed");
    }
    Ok(())
}

/// Venv location under the cargo target directory, so `cargo clean` resets it.
fn venv_dir() -> PathBuf {
    let target = env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target"), PathBuf::from);
    target.join("bonesinfra-pytest-venv")
}

fn ensure_venv(python_dir: &Path, venv: &Path) -> Result<()> {
    let host_python = host_python();
    validate_python(Path::new(host_python))?;
    let stamp_file = venv.join(".stamp");
    let stamp = dependency_stamp(python_dir);
    if fs::read_to_string(&stamp_file).is_ok_and(|existing| existing == stamp) {
        validate_python(&venv_python(venv))?;
        return Ok(());
    }

    if venv.exists() {
        fs::remove_dir_all(venv).with_context(|| format!("Failed to reset the pytest venv at {}", venv.display()))?;
    }

    let mut create = Command::new(&host_python);
    create.arg("-m").arg("venv").arg(venv);
    run(create, "Python 3.12 or newer -m venv")?;

    let mut install = Command::new(venv_python(venv));
    install.current_dir(python_dir).args(["-m", "pip", "install", "--quiet", "-e", ".", "pytest"]);
    run(install, "pip install of bonesinfra and pytest into the test venv")?;

    // Written last so an interrupted setup rebuilds from scratch on the next run.
    fs::write(&stamp_file, stamp)
        .with_context(|| format!("Failed to write the pytest venv stamp at {}", stamp_file.display()))?;
    Ok(())
}

fn validate_python(python: &Path) -> Result<()> {
    let output = Command::new(python)
        .arg("--version")
        .output()
        .with_context(|| format!("Failed to run {} (Python 3.12 or newer is required)", python.display()))?;
    if !output.status.success() {
        bail!("{} did not report a Python version (Python 3.12 or newer is required)", python.display());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let version = stdout.trim().strip_prefix("Python ").or_else(|| stderr.trim().strip_prefix("Python "));
    let Some(version) = version else {
        bail!("{} reported an invalid Python version", python.display());
    };
    let mut components = version.split('.');
    let major = components.next().and_then(|value| value.parse::<u32>().ok());
    let minor = components.next().and_then(|value| value.parse::<u32>().ok());
    if major != Some(3) || minor.is_none_or(|value| value < 12) {
        bail!("{} is Python {version}; Python 3.12 or newer is required", python.display());
    }
    Ok(())
}

/// Stamp over the dependency manifests; source changes need no rebuild because
/// the package is installed editable.
fn dependency_stamp(python_dir: &Path) -> String {
    let mut hasher = DefaultHasher::new();
    for name in ["pyproject.toml", "uv.lock"] {
        if let Ok(bytes) = fs::read(python_dir.join(name)) {
            name.hash(&mut hasher);
            bytes.hash(&mut hasher);
        }
    }
    format!("{:016x}", hasher.finish())
}

#[cfg(windows)]
fn host_python() -> &'static str {
    "python.exe"
}

#[cfg(not(windows))]
fn host_python() -> &'static str {
    "python3"
}

fn venv_python(venv: &Path) -> PathBuf {
    #[cfg(windows)]
    return venv.join("Scripts/python.exe");

    #[cfg(not(windows))]
    venv.join("bin/python")
}

fn run(mut command: Command, description: &str) -> Result<Output> {
    let output = command.output().with_context(|| format!("Failed to start {description}"))?;
    if !output.status.success() {
        bail!(
            "{description} failed ({}).\n\n--- stdout ---\n{}\n--- stderr ---\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr),
        );
    }
    Ok(output)
}
