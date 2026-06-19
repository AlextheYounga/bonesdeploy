use std::process::{Command, ExitStatus, Output};

use anyhow::{Context, Result};

pub fn output(args: &[&str]) -> Result<Output> {
    Command::new("rsync").args(args).output().context("Failed to run rsync — is it installed?")
}

pub fn status(args: &[&str]) -> Result<ExitStatus> {
    Ok(output(args)?.status)
}
