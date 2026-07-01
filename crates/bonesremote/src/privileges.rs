use anyhow::{Result, bail};

unsafe extern "C" {
    fn geteuid() -> u32;
}

pub fn ensure_root(command_name: &str) -> Result<()> {
    // SAFETY: geteuid is a POSIX syscall that always succeeds and returns the
    // calling process's effective UID. UB is impossible.
    if unsafe { geteuid() } == 0 {
        return Ok(());
    }

    bail!("{command_name} must be run as root (sudo)")
}
