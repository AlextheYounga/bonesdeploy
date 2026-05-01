use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::privileges;
use crate::release_state;

use super::wire_release;

pub fn run(config_path: &str, revision: Option<&str>) -> Result<()> {
    privileges::ensure_not_root("bonesremote hooks post-receive")?;

    let cfg = config::load(Path::new(config_path))?;
    let build_root = release_state::build_root(&cfg);

    if !build_root.exists() {
        bail!("Build workspace does not exist: {}", build_root.display());
    }

    let checkout_target = revision.unwrap_or(cfg.data.branch.as_str());
    println!("Checking out {checkout_target} to {}...", build_root.display());

    let status = Command::new("git")
        .arg("--work-tree")
        .arg(&build_root)
        .arg("--git-dir")
        .arg(&cfg.data.git_dir)
        .arg("checkout")
        .arg("-f")
        .arg(checkout_target)
        .status()
        .with_context(|| {
            format!("Failed to run git checkout for target '{checkout_target}' into {}", build_root.display())
        })?;

    if !status.success() {
        bail!("git checkout failed for target '{checkout_target}': status {status}");
    }

    wire_release::run(config_path)?;
    Ok(())
}
