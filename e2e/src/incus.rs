//! Thin wrapper over the `incus` CLI.

use std::process::Command;

use anyhow::{Context, Result, bail};

/// Runs `incus <args>` and returns stdout, failing loudly with stderr attached.
pub fn incus(args: &[&str]) -> Result<String> {
    let output = Command::new("incus").args(args).output().context("Failed to run `incus` — is Incus installed?")?;

    if !output.status.success() {
        bail!(
            "`incus {}` failed ({}):\n{}",
            args.join(" "),
            output.status,
            String::from_utf8_lossy(&output.stderr).trim(),
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Fails fast with setup instructions when the Incus daemon is unreachable.
pub fn check_server() -> Result<()> {
    if incus(&["info"]).is_ok() {
        return Ok(());
    }
    bail!(
        "The Incus server is unreachable.\n\n\
         - start the daemon:            sudo systemctl start incus\n\
         - first-time initialization:   sudo incus admin init --minimal\n\
         - socket access for your user: sudo usermod -aG incus-admin $USER (then re-login)"
    )
}
