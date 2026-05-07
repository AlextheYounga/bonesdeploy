use std::env;
use std::process::Command;

use anyhow::{bail, Context, Result};

use crate::support::paths;

pub const DEFAULT_SERVICE: &str = "bonesdeploy-test-server";

pub fn bootstrap_ssh_user() -> String {
    env::var("BONES_E2E_BOOTSTRAP_USER").unwrap_or_else(|_| String::from("root"))
}

pub fn docker_compose_up() -> Result<()> {
    let status = Command::new("docker")
        .args(["compose", "up", "-d", "--build", DEFAULT_SERVICE])
        .current_dir(paths::docker_dir())
        .status()
        .context("Failed to run docker compose up")?;

    if !status.success() {
        bail!("docker compose up failed with status {status}");
    }

    Ok(())
}

pub fn docker_compose_down() -> Result<()> {
    let status = Command::new("docker")
        .args(["compose", "down", "--remove-orphans"])
        .current_dir(paths::docker_dir())
        .status()
        .context("Failed to run docker compose down")?;

    if !status.success() {
        bail!("docker compose down failed with status {status}");
    }

    Ok(())
}

pub fn docker_available() -> bool {
    Command::new("docker").arg("--version").status().is_ok_and(|status| status.success())
}
