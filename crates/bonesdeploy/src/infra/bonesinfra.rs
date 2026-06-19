use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

const REPOSITORY_URL: &str = "https://github.com/AlextheYounga/bonesinfra.git";
const CHECKOUT_DIR: &str = "bonesinfra";
const EXECUTABLE_NAME: &str = "bonesinfra";

pub fn prefetch() -> Result<()> {
    ensure_available().map(|_| ())
}

pub(super) fn executable_path() -> Result<PathBuf> {
    ensure_available()
}

fn ensure_available() -> Result<PathBuf> {
    let checkout = checkout_dir();
    let executable = checkout.join("dist").join(EXECUTABLE_NAME);
    let build_script = checkout.join("build.sh");

    if executable.is_file() {
        return Ok(executable);
    }

    if let Ok(metadata) = fs::symlink_metadata(&checkout) {
        if !metadata.file_type().is_dir() {
            reset_checkout(&checkout)?;
        }
    }

    if !checkout.is_dir() || !build_script.is_file() {
        if checkout.is_dir() {
            reset_checkout(&checkout)?;
        }
        install_checkout(&checkout)?;
    }

    build_checkout(&checkout)?;

    if executable.is_file() {
        return Ok(executable);
    }

    let contents: Vec<_> = fs::read_dir(&checkout)
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path().display().to_string())
        .collect();
    if contents.is_empty() {
        bail!("bonesinfra build finished but checkout is empty at {}.", checkout.display());
    }

    bail!(
        "Built bonesinfra at {}, but {} is missing.\nContents of checkout:\n  {}",
        checkout.display(),
        executable.display(),
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

fn build_checkout(checkout: &Path) -> Result<()> {
    let status = Command::new("bash")
        .arg("build.sh")
        .current_dir(checkout)
        .status()
        .with_context(|| format!("Failed to run build.sh in {}", checkout.display()))?;

    if !status.success() {
        bail!("Failed to build bonesinfra in {}.", checkout.display());
    }

    Ok(())
}

fn checkout_dir() -> PathBuf {
    Path::new(paths::LOCAL_BONES_DIR).join(".lib").join(CHECKOUT_DIR)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;

    use anyhow::Result;
    use tempfile::TempDir;

    use super::{checkout_dir, reset_checkout};

    #[test]
    fn checkout_dir_lives_under_local_bones_lib() {
        assert_eq!(checkout_dir(), Path::new(".bones/.lib").join("bonesinfra"));
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
