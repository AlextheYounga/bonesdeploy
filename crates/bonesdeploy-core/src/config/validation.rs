use anyhow::{Result, bail};

const RESERVED_PROJECT_NAMES: &[&str] = &[
    "basic",
    "default",
    "deploy",
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

/// # Errors
/// Returns an error when `site` is not a valid site name.
pub fn validate_site_name(site: &str) -> Result<()> {
    validate_project_name(site).map_err(|error| anyhow::anyhow!("Invalid site name: {error}"))
}

/// Validates the DNS name accepted by managed nginx and Certbot commands.
///
/// # Errors
/// Returns an error when the value is not a lower-case, multi-label DNS name.
pub fn validate_domain(domain: &str) -> Result<()> {
    let labels = domain.split('.').collect::<Vec<_>>();
    let valid_label = |label: &&str| {
        !label.is_empty()
            && label.len() <= 63
            && label.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
            && !label.starts_with('-')
            && !label.ends_with('-')
    };
    if domain.len() <= 253 && !domain.ends_with('.') && labels.len() >= 2 && labels.iter().all(valid_label) {
        return Ok(());
    }
    bail!("Domain must be a valid lower-case DNS name: {domain}")
}

/// Validates the basic email shape required by Certbot.
///
/// # Errors
/// Returns an error when the value is empty, non-ASCII, contains whitespace,
/// exceeds the standard address length, or lacks one `@` and a dotted domain.
pub fn validate_email(email: &str) -> Result<()> {
    let Some((local, domain)) = email.split_once('@') else {
        bail!("Email must be a valid address: {email}");
    };
    if email.len() <= 254
        && email.is_ascii()
        && !email.bytes().any(|byte| byte.is_ascii_whitespace())
        && !local.is_empty()
        && !domain.is_empty()
        && !domain.contains('@')
        && domain.contains('.')
    {
        return Ok(());
    }
    bail!("Email must be a valid address: {email}")
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
