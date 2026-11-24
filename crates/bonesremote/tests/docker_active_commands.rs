use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesremote::runtime::docker::command::{active_start, active_stop};

#[test]
fn active_compose_start_and_stop_use_safe_reconciliation_commands() -> Result<()> {
    let root = tempfile::tempdir()?;
    let bin = root.path().join("bin");
    fs::create_dir(&bin)?;
    let docker = bin.join("docker");
    fs::write(&docker, "#!/bin/sh\nprintf '%s\\n' \"$PWD\" \"$@\" > \"$BONES_TEST_DOCKER_LOG\"\n")?;
    fs::set_permissions(&docker, fs::Permissions::from_mode(0o755))?;
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "exercise_active_compose_commands", "--nocapture"])
        .env("BONES_DOCKER_COMMAND_TEST", root.path())
        .env("BONES_TEST_DOCKER_LOG", root.path().join("docker.log"))
        .env("PATH", bin)
        .output()?;
    ensure!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn exercise_active_compose_commands() -> Result<()> {
    let Some(root) = env::var_os("BONES_DOCKER_COMMAND_TEST") else { return Ok(()) };
    let root = PathBuf::from(root);
    let current = root.join("current");
    fs::create_dir(&current)?;
    fs::write(current.join("compose.yaml"), "services: {}\n")?;
    active_start("atlas", &root, 45)?;
    let log = fs::read_to_string(root.join("docker.log"))?;
    let lines = log.lines().collect::<Vec<_>>();
    assert_eq!(lines.first().copied(), current.to_str());
    assert!(lines.ends_with(&[
        "up",
        "--detach",
        "--no-build",
        "--pull",
        "never",
        "--remove-orphans",
        "--wait",
        "--wait-timeout",
        "45"
    ]));
    active_stop("atlas", &root)?;
    let log = fs::read_to_string(root.join("docker.log"))?;
    let lines = log.lines().collect::<Vec<_>>();
    assert_eq!(lines.last(), Some(&"stop"));
    assert!(!lines.iter().any(|arg| matches!(*arg, "down" | "rm" | "--volumes" | "-v")));
    Ok(())
}
