use std::env;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use anyhow::{Context, Result, bail};

use crate::support::paths;

pub const DEFAULT_SERVICE: &str = "bonesdeploy-test-server";

pub struct DockerSession {
    _lock: MutexGuard<'static, ()>,
}

pub fn bootstrap_ssh_user() -> String {
    env::var("BONES_E2E_BOOTSTRAP_USER").unwrap_or_else(|_| String::from("root"))
}

pub fn docker_compose_up() -> Result<()> {
    let compose_file = paths::docker_dir().join("docker-compose.yml");
    let status = Command::new("docker")
        .args(["compose", "-f"])
        .arg(&compose_file)
        .args(["up", "-d", DEFAULT_SERVICE])
        .status()
        .context("Failed to run docker compose up")?;

    if !status.success() {
        bail!("docker compose up failed with status {status}");
    }

    Ok(())
}

pub fn docker_session() -> Result<DockerSession> {
    static DOCKER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    static STARTUP: OnceLock<Result<(), String>> = OnceLock::new();

    let lock = DOCKER_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let startup_result = STARTUP.get_or_init(|| setup_docker_session().map_err(|error| format!("{error:#}")));

    if let Err(message) = startup_result {
        bail!("{message}");
    }

    Ok(DockerSession { _lock: lock })
}

fn setup_docker_session() -> Result<()> {
    let _ = docker_compose_down();
    docker_compose_up()
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
