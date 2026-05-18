use std::env;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::support::paths;

pub const DEFAULT_SERVICE: &str = "bonesdeploy-test-server";

pub fn bootstrap_ssh_user() -> String {
    env::var("BONES_E2E_BOOTSTRAP_USER").unwrap_or_else(|_| String::from("root"))
}

pub fn docker_compose_up() -> Result<()> {
    let compose_file = paths::docker_dir().join("docker-compose.yml");
    let status = Command::new("docker")
        .args(["compose", "-f"])
        .arg(&compose_file)
        .args(["up", "-d", "--build", DEFAULT_SERVICE])
        .status()
        .context("Failed to run docker compose up")?;

    if !status.success() {
        bail!("docker compose up failed with status {status}");
    }

    Ok(())
}

pub fn docker_compose_down() -> Result<()> {
    let compose_file = paths::docker_dir().join("docker-compose.yml");
    let status = Command::new("docker")
        .args(["compose", "-f"])
        .arg(&compose_file)
        .args(["down", "--remove-orphans"])
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

pub fn docker_exec(command: &str) -> Result<()> {
    let status = Command::new("docker")
        .args(["exec", DEFAULT_SERVICE, "bash", "-lc", command])
        .status()
        .context("Failed to run docker exec")?;

    if !status.success() {
        bail!("docker exec failed with status {status}");
    }

    Ok(())
}

pub fn docker_exec_output(command: &str) -> Result<String> {
    let output = Command::new("docker")
        .args(["exec", DEFAULT_SERVICE, "bash", "-lc", command])
        .output()
        .context("Failed to run docker exec")?;

    if !output.status.success() {
        bail!(
            "docker exec failed with status {}\nstdout:\n{}\nstderr:\n{}",
            output.status,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
