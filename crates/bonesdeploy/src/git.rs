use std::process::Command;

use anyhow::{Context, Result, bail};

pub fn ensure_git_repository() -> Result<()> {
    let output =
        Command::new("git").args(["rev-parse", "--is-inside-work-tree"]).output().context("Failed to run git")?;

    if !output.status.success() {
        bail!("Not a git repository");
    }

    Ok(())
}

pub fn validate_remote_exists(remote_name: &str) -> Result<()> {
    let status = Command::new("git").args(["remote", "get-url", remote_name]).status().context("Failed to run git")?;

    if !status.success() {
        bail!(
            "No git remote '{remote_name}' found. \
             Please set one up before running bonesdeploy init:\n  \
             git remote add {remote_name} git@<host>:<repo>.git"
        );
    }
    Ok(())
}
