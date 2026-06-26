use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;

use crate::privileges;
use shared::paths;

pub fn run() -> Result<()> {
    privileges::ensure_root("bonesremote init")?;

    println!("{}", style(format!("{} init", paths::BONESREMOTE_BINARY)).bold());

    let sudoers_path = paths::SUDOERS_PATH;
    let bonesdeploy_path = which_bonesdeploy_remote()?;

    // Only the commands that need ownership or live-state changes run via sudo.
    let sudoers_content = format!(
        "# Installed by bonesremote init\n\
             {} ALL=(root) NOPASSWD: {} hook post-receive --site *, {} service restart --config *\n",
        paths::DEPLOY_USER,
        bonesdeploy_path.display(),
        bonesdeploy_path.display()
    );

    fs::write(sudoers_path, &sudoers_content).with_context(|| format!("Failed to write {sudoers_path}"))?;

    Command::new("chmod").args(["0440", sudoers_path]).status().context("Failed to chmod sudoers drop-in")?;

    let status = Command::new("visudo").args(["-c", "-f", sudoers_path]).status().context("Failed to run visudo")?;

    if !status.success() {
        fs::remove_file(sudoers_path).ok();
        bail!("visudo validation failed — sudoers drop-in removed for safety");
    }

    println!("{} Installed sudoers drop-in at {}", style("Done!").green().bold(), sudoers_path);

    Ok(())
}

fn which_bonesdeploy_remote() -> Result<PathBuf> {
    let output = Command::new("which")
        .arg(paths::BONESREMOTE_BINARY)
        .output()
        .context(format!("Failed to run 'which {}'", paths::BONESREMOTE_BINARY))?;

    if !output.status.success() {
        bail!(
            "{} is not in PATH. \
             Install it globally before running init.",
            paths::BONESREMOTE_BINARY
        );
    }

    let path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
    let path = fs::canonicalize(&path).with_context(|| format!("Failed to canonicalize {}", path.display()))?;
    if !approved_bonesdeploy_path(&path) {
        bail!("Refusing to write sudoers entry for {}: expected {} or /usr/bin", path.display(), paths::USR_LOCAL_BIN);
    }

    Ok(path)
}

fn approved_bonesdeploy_path(path: &Path) -> bool {
    path.starts_with(paths::USR_LOCAL_BIN) || path.starts_with("/usr/bin")
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::approved_bonesdeploy_path;

    #[test]
    fn approved_bonesdeploy_path_accepts_standard_bin_dirs() {
        assert!(approved_bonesdeploy_path(Path::new("/usr/local/bin/bonesremote")));
        assert!(approved_bonesdeploy_path(Path::new("/usr/bin/bonesremote")));
    }

    #[test]
    fn approved_bonesdeploy_path_rejects_unexpected_dirs() {
        assert!(!approved_bonesdeploy_path(Path::new("/home/alex/.local/bin/bonesremote")));
    }
}
