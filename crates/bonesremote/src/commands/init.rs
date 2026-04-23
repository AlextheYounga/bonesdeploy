use std::fs;
use std::process::Command;

use anyhow::{Context, Result, bail};
use console::style;
use nix::unistd::geteuid;

const SUDOERS_PATH: &str = "/etc/sudoers.d/bonesdeploy";

pub fn run() -> Result<()> {
    if !geteuid().is_root() {
        bail!("bonesremote init must be run as root (sudo)");
    }

    println!("{}", style("bonesremote init").bold());

    let bonesdeploy_path = which_bonesdeploy_remote()?;

    // The sudoers rule allows the deploy user (git) to run
    // bonesremote commands without a password.
    let sudoers_content = format!(
        "# Installed by bonesremote init\n\
         git ALL=(ALL) NOPASSWD: {bonesdeploy_path} *\n"
    );

    fs::write(SUDOERS_PATH, &sudoers_content).with_context(|| format!("Failed to write {SUDOERS_PATH}"))?;

    // Set correct permissions (sudoers drop-ins must be 0440)
    Command::new("chmod").args(["0440", SUDOERS_PATH]).status().context("Failed to chmod sudoers drop-in")?;

    // Validate with visudo
    let status = Command::new("visudo").args(["-c", "-f", SUDOERS_PATH]).status().context("Failed to run visudo")?;

    if !status.success() {
        fs::remove_file(SUDOERS_PATH).ok();
        bail!("visudo validation failed — sudoers drop-in removed for safety");
    }

    println!("{} Installed sudoers drop-in at {SUDOERS_PATH}", style("Done!").green().bold());

    Ok(())
}

fn which_bonesdeploy_remote() -> Result<String> {
    let output = Command::new("which").arg("bonesremote").output().context("Failed to run 'which bonesremote'")?;

    if !output.status.success() {
        bail!(
            "bonesremote is not in PATH. \
             Install it globally before running init."
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
