use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::config;
use crate::privileges;

pub fn run(config_path: &str) -> Result<()> {
    privileges::ensure_root("bonesremote service restart")?;

    let cfg = config::load(Path::new(config_path))?;
    let service_name = format!("{}-nginx", cfg.data.project_name);

    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", &service_name])
        .status()
        .context("Failed to check nginx service status")?;

    if status.success() {
        let restart_status = Command::new("systemctl")
            .args(["restart", &service_name])
            .status()
            .context("Failed to restart nginx service")?;

        if !restart_status.success() {
            bail!("Failed to restart {service_name} service");
        }
        println!("Restarted {service_name} service");
    }

    Ok(())
}
