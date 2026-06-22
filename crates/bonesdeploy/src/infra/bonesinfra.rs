use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

const REPOSITORY_URL: &str = "https://github.com/AlextheYounga/bonesinfra.git";
const CHECKOUT_DIR: &str = "bonesinfra";

pub fn prefetch() -> Result<()> {
    ensure_available().map(|_| ())
}

pub(super) fn executable_path() -> Result<PathBuf> {
    ensure_available()
}

fn ensure_available() -> Result<PathBuf> {
    let checkout = checkout_dir();
    let venv_python = checkout.join(".venv").join("bin").join("python");

    if let Ok(metadata) = fs::symlink_metadata(&checkout)
        && !metadata.file_type().is_dir()
    {
        reset_checkout(&checkout)?;
    }

    if !checkout.is_dir() {
        install_checkout(&checkout)?;
    }

    if venv_python.is_file() {
        return Ok(venv_python);
    }

    setup_venv(&checkout)?;

    if venv_python.is_file() {
        return Ok(venv_python);
    }

    let contents: Vec<_> = fs::read_dir(&checkout)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path().display().to_string())
        .collect();
    if contents.is_empty() {
        bail!("bonesinfra setup finished but checkout is empty at {}.", checkout.display());
    }

    bail!(
        "bonesinfra setup finished at {}, but {} is missing.\nContents of checkout:\n  {}",
        checkout.display(),
        venv_python.display(),
        contents.join("\n  ")
    );
}

fn reset_checkout(checkout: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(checkout)
        .with_context(|| format!("Failed to inspect stale bonesinfra checkout at {}", checkout.display()))?;

    if metadata.file_type().is_dir() {
        fs::remove_dir_all(checkout)
            .with_context(|| format!("Failed to remove stale bonesinfra checkout at {}", checkout.display()))?;
        return Ok(());
    }

    fs::remove_file(checkout)
        .with_context(|| format!("Failed to remove stale bonesinfra checkout at {}", checkout.display()))?;
    Ok(())
}

fn install_checkout(checkout: &Path) -> Result<()> {
    if let Some(parent) = checkout.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let status = Command::new("git")
        .args(["clone", "--depth", "1", REPOSITORY_URL, &checkout.to_string_lossy()])
        .status()
        .context("Failed to run git clone for bonesinfra install")?;

    if !status.success() {
        bail!("Failed to install bonesinfra from {} into {}.", REPOSITORY_URL, checkout.display());
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
    paths::bones_config_root().join(CHECKOUT_DIR)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use anyhow::Result;
    use shared::paths;
    use tempfile::TempDir;

    use super::{checkout_dir, reset_checkout};

    #[test]
    fn checkout_dir_lives_under_bones_config_root() {
        assert_eq!(checkout_dir(), paths::bones_config_root().join("bonesinfra"));
    }

    #[test]
    fn reset_checkout_removes_stale_directory() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let checkout = temp_dir.path().join("bonesinfra");
        fs::create_dir_all(checkout.join("nested"))?;

        reset_checkout(&checkout)?;

        assert!(!checkout.exists());
        Ok(())
    }

    #[test]
    fn reset_checkout_removes_stale_file() -> Result<()> {
        let temp_dir = TempDir::new()?;
        let checkout = temp_dir.path().join("bonesinfra");
        fs::write(&checkout, "stale")?;

        reset_checkout(&checkout)?;

        assert!(!checkout.exists());
        Ok(())
    }
}
