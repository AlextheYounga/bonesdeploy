use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

const REPOSITORY_URL: &str = "https://github.com/AlextheYounga/bonesinfra.git";
const CHECKOUT_DIR: &str = "bonesinfra";
const ENTRYPOINT: &str = "main.py";

pub fn main_py_path() -> Result<PathBuf> {
    let checkout = ensure_available()?;
    Ok(checkout.join(ENTRYPOINT))
}

fn ensure_available() -> Result<PathBuf> {
    let checkout = checkout_dir();
    let main_py = checkout.join(ENTRYPOINT);
    if main_py.is_file() {
        return Ok(checkout);
    }

    if checkout.exists() {
        bail!("Hidden bonesinfra checkout is incomplete at {}. Remove it and retry.", checkout.display());
    }

    install_checkout(&checkout)?;

    if !main_py.is_file() {
        let contents: Vec<_> = fs::read_dir(&checkout)
            .into_iter()
            .flatten()
            .filter_map(|e| e.ok())
            .map(|e| e.path().display().to_string())
            .collect();
        if contents.is_empty() {
            bail!(
                "Git clone succeeded but checkout is empty at {}. The repository may have no default branch.",
                checkout.display()
            );
        }
        bail!(
            "Installed bonesinfra checkout at {}, but {} is missing.\nContents of checkout:\n  {}",
            checkout.display(),
            main_py.display(),
            contents.join("\n  ")
        );
    }

    Ok(checkout)
}

fn install_checkout(checkout: &Path) -> Result<()> {
    if let Some(parent) = checkout.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create {}", parent.display()))?;
    }

    let status = Command::new("git")
        .args(["clone", "--depth", "1", REPOSITORY_URL, &checkout.to_string_lossy()])
        .status()
        .context("Failed to run git clone for hidden bonesinfra checkout")?;

    if !status.success() {
        bail!("Failed to install hidden bonesinfra checkout from {} into {}.", REPOSITORY_URL, checkout.display());
    }

    Ok(())
}

fn checkout_dir() -> PathBuf {
    paths::bones_state_root().join(CHECKOUT_DIR)
}

#[cfg(test)]
mod tests {
    use shared::paths;

    use super::checkout_dir;

    #[test]
    fn checkout_dir_lives_under_bones_state_root() {
        assert_eq!(checkout_dir(), paths::bones_state_root().join("bonesinfra"));
    }
}
