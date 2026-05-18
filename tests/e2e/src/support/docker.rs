use std::env;
use std::process::Command;
use std::sync::{Mutex, MutexGuard, OnceLock};

use anyhow::{Context, Result, bail};

pub const DEFAULT_SERVICE: &str = "bonesdeploy-test-server";

pub struct DockerSession {
    _lock: MutexGuard<'static, ()>,
}

pub fn bootstrap_ssh_user() -> String {
    env::var("BONES_E2E_BOOTSTRAP_USER").unwrap_or_else(|_| String::from("root"))
}

pub fn docker_session() -> Result<DockerSession> {
    static DOCKER_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

    let lock = DOCKER_LOCK.get_or_init(|| Mutex::new(())).lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    ensure_container_running()?;

    Ok(DockerSession { _lock: lock })
}

pub fn docker_available() -> bool {
    Command::new("docker").arg("--version").output().is_ok_and(|output| output.status.success())
}

fn ensure_container_running() -> Result<()> {
    let output = Command::new("docker")
        .args(["inspect", "-f", "{{.State.Running}}", DEFAULT_SERVICE])
        .output()
        .context("Failed to inspect e2e Docker container")?;

    if !output.status.success() {
        bail!(
            "Docker container '{}' is unavailable. Run `tests/e2e/run-e2e.sh` to recreate and start it.",
            DEFAULT_SERVICE
        );
    }

    let running = String::from_utf8_lossy(&output.stdout);
    if running.trim() != "true" {
        bail!("Docker container '{}' is not running. Start it with `tests/e2e/run-e2e.sh`.", DEFAULT_SERVICE);
    }

    Ok(())
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
