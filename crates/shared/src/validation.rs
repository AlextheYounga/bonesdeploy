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

#[must_use]
pub fn is_numbered_shell_script(name: &str) -> bool {
    let Some((number, script_name)) = name.split_once('_') else {
        return false;
    };

    number.len() == 2
        && number.bytes().all(|byte| byte.is_ascii_digit())
        && script_name.strip_suffix(".sh").is_some_and(|script_name| !script_name.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{is_numbered_shell_script, validate_project_name};

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

    #[test]
    fn numbered_shell_scripts_require_digits_underscore_and_name() {
        assert!(is_numbered_shell_script("01_build.sh"));
        assert!(!is_numbered_shell_script("999_prepare.sh"));
        assert!(!is_numbered_shell_script("1_prepare.sh"));
        assert!(!is_numbered_shell_script("build.sh"));
        assert!(!is_numbered_shell_script("01build.sh"));
        assert!(!is_numbered_shell_script("01_.sh"));
        assert!(!is_numbered_shell_script("01_build.py"));
    }
}
