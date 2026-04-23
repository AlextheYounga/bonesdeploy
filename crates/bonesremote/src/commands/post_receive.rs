use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::release_state;

use super::wire_release;

pub fn run(config_path: &str) -> Result<()> {
    let cfg = config::load(Path::new(config_path))?;
    let release_name = release_state::read_staged_release(&cfg)?;
    let release_path = release_state::release_dir(&cfg, &release_name);

    if !release_path.exists() {
        bail!("Staged release directory does not exist: {}", release_path.display());
    }

    println!("Checking out {} to {}...", cfg.data.branch, release_path.display());

    let status = Command::new("git")
        .arg("--work-tree")
        .arg(&release_path)
        .arg("--git-dir")
        .arg(&cfg.data.git_dir)
        .arg("checkout")
        .arg("-f")
        .arg(&cfg.data.branch)
        .status()
        .with_context(|| {
            format!("Failed to run git checkout for branch '{}' into {}", cfg.data.branch, release_path.display())
        })?;

    if !status.success() {
        bail!("git checkout failed for branch '{}': status {status}", cfg.data.branch);
    }

    wire_release::run(config_path)?;
    Ok(())
}
