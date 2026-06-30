use std::process::Command;

use anyhow::{Context, Result, bail};
use shared::paths;

use crate::privileges;

pub fn run(site: &str) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;

    let service_name = paths::nginx_service_name(site);

    let status = Command::new("systemctl")
        .args(["restart", &service_name])
        .status()
        .with_context(|| format!("Failed to restart {service_name} service"))?;

    if !status.success() {
        bail!("Failed to restart {service_name} service");
    }

    println!("Restarted {service_name} service");
    Ok(())
}

#[cfg(test)]
mod tests {}
