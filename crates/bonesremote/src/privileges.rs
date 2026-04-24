use std::io::Error;

use anyhow::{Result, bail};
use nix::libc;
use nix::unistd::geteuid;

pub fn ensure_root(command_name: &str) -> Result<()> {
    if geteuid().is_root() {
        return Ok(());
    }

    bail!("{command_name} must be run as root (sudo)")
}

pub fn ensure_not_root(command_name: &str) -> Result<()> {
    if !geteuid().is_root() {
        return Ok(());
    }

    bail!("{command_name} must not run as root")
}

pub fn set_no_new_privs() -> Result<()> {
    // SAFETY: PR_SET_NO_NEW_PRIVS is process-scoped and takes integer arguments.
    let result = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
    if result == 0 {
        return Ok(());
    }

    let error = Error::last_os_error();
    bail!("Failed to set PR_SET_NO_NEW_PRIVS: {error}")
}
