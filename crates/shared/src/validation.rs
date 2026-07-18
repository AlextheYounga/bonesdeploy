use anyhow::{bail, Result};

const RESERVED_PROJECT_NAMES: &[&str] = &[
    "basic",
    "default",
    "emergency",
    "final",
    "graphical",
    "halt",
    "initrd",
    "local-fs",
    "multi-user",
    "network",
    "network-online",
    "poweroff",
    "reboot",
    "remote-fs",
    "rescue",
    "shutdown",
    "sockets",
    "swap",
    "sysinit",
    "system-update",
    "timers",
    "umount",
];

/// # Errors
/// Returns an error when `project_name` is not a safe site identifier.
pub fn validate_project_name(project_name: &str) -> Result<()> {
    if !project_name.is_empty()
        && project_name.chars().all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
        && !RESERVED_PROJECT_NAMES.contains(&project_name)
    {
        return Ok(());
    }

    bail!("Invalid project name: {project_name}")
}

#[cfg(test)]
mod tests {
    use super::validate_project_name;

    #[test]
    fn accepts_site_identifiers() {
        assert!(validate_project_name("nexttest-2").is_ok());
    }

    #[test]
    fn rejects_unit_name_syntax_and_reserved_targets() {
        assert!(validate_project_name("multi-user.target").is_err());
        assert!(validate_project_name("shop_admin").is_err());
        assert!(validate_project_name("reboot").is_err());
        assert!(validate_project_name("multi-user").is_err());
    }
}
