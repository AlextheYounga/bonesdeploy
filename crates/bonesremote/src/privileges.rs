use anyhow::{Result, bail};
use nix::unistd::geteuid;

pub fn ensure_root(command_name: &str) -> Result<()> {
    if geteuid().is_root() {
        return Ok(());
    }

    bail!("{command_name} must be run as root (sudo)")
}
