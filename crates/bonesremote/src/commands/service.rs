use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::privileges;

pub fn run(site_name: &str) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;

    if !is_valid_site_name(site_name) {
        bail!(
            "invalid site name: {site_name}. Must be 1-32 chars, lowercase letters, digits, hyphens, underscores, starting with a letter or underscore."
        );
    }

    let service_name = format!("{site_name}-nginx");

    let status = Command::new("systemctl")
        .args(["restart", &service_name])
        .status()
        .context("Failed to restart nginx service")?;

    if !status.success() {
        bail!("Failed to restart {service_name} service");
    }
    println!("Restarted {service_name} service");

    Ok(())
}

fn is_valid_site_name(name: &str) -> bool {
    if name.is_empty() || name.len() > 32 {
        return false;
    }
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}
