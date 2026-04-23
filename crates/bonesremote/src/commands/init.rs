use std::fs;
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use nix::unistd::geteuid;

use crate::config;

pub fn run() -> Result<()> {
    if !geteuid().is_root() {
        bail!("{} init must be run as root (sudo)", config::Constants::BINARY_NAME);
    }

    println!("{}", style(format!("{} init", config::Constants::BINARY_NAME)).bold());

    let sudoers_path = config::Constants::SUDOERS_PATH;
    let bonesdeploy_path = which_bonesdeploy_remote()?;

    // The sudoers rule allows the deploy user (git) to run
    // bonesremote commands without a password.
    let sudoers_content = format!(
        "# Installed by bonesremote init\n\
         git ALL=(ALL) NOPASSWD: {bonesdeploy_path} *\n"
    );

    fs::write(sudoers_path, &sudoers_content).with_context(|| format!("Failed to write {sudoers_path}"))?;

    // Set correct permissions (sudoers drop-ins must be 0440)
    Command::new("chmod").args(["0440", sudoers_path]).status().context("Failed to chmod sudoers drop-in")?;

    // Validate with visudo
    let status = Command::new("visudo").args(["-c", "-f", sudoers_path]).status().context("Failed to run visudo")?;

    if !status.success() {
        fs::remove_file(sudoers_path).ok();
        bail!("visudo validation failed — sudoers drop-in removed for safety");
    }

    println!("{} Installed sudoers drop-in at {}", style("Done!").green().bold(), sudoers_path);

    Ok(())
}

fn which_bonesdeploy_remote() -> Result<String> {
    let output = Command::new("which")
        .arg(config::Constants::BINARY_NAME)
        .output()
        .context(format!("Failed to run 'which {}'", config::Constants::BINARY_NAME))?;

    if !output.status.success() {
        bail!(
            "{} is not in PATH. \
             Install it globally before running init.",
            config::Constants::BINARY_NAME
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
