use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::privileges;
use crate::release_state;

use super::wire_release;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_not_root("bonesremote hooks post-receive")?;

    let cfg = config::load(Path::new(config_path))?;
    let build_root = release_state::build_root(&cfg);

    if !build_root.exists() {
        bail!("Build workspace does not exist: {}", build_root.display());
    }

    println!("Checking out {} to {}...", cfg.data.branch, build_root.display());

    let status = Command::new("git")
        .arg("--work-tree")
        .arg(&build_root)
        .arg("--git-dir")
        .arg(&cfg.data.git_dir)
        .arg("checkout")
        .arg("-f")
        .arg(&cfg.data.branch)
        .status()
        .with_context(|| {
            format!("Failed to run git checkout for branch '{}' into {}", cfg.data.branch, build_root.display())
        })?;

    if !status.success() {
        bail!("git checkout failed for branch '{}': status {status}", cfg.data.branch);
    }

    wire_release::run(config_path)?;
    Ok(())
}
