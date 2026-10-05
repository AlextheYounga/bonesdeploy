#![cfg(windows)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use bonesdeploy::build;
use bonesdeploy_core::config::{Bones, RuntimeBackend};

const LOCAL_PIPE: &str = "npipe:////./pipe/dockerDesktopLinuxEngine";

#[test]
fn windows_docker_uses_the_local_pipe_and_preserves_drive_letter_bind_arguments() -> Result<()> {
    let workspace = tempfile::tempdir()?;
    let project = workspace.path().join("project with spaces");
    fs::create_dir(&project)?;
    initialize_project(&project)?;
    let tools = tempfile::tempdir()?;
    let docker_log = tools.path().join("docker.log");
    write_fake_docker(tools.path())?;

    let output = run_helper(
        WindowsDockerFixture { project: &project, tools: tools.path(), log: &docker_log },
        "windows_docker_helper",
        [],
    )?;
    anyhow::ensure!(output.status.success(), "helper failed: {}", String::from_utf8_lossy(&output.stderr));

    let commands = fs::read_to_string(&docker_log)?;
    for command in commands.lines() {
        assert!(command.starts_with(&format!("{LOCAL_PIPE}|")), "wrong endpoint: {command}");
        assert!(!command.contains("desktop-local"), "context selector leaked: {command}");
    }
    assert!(commands.contains(project.to_string_lossy().as_ref()), "source bind lost its Windows path: {commands}");
    assert!(commands.contains(":/workspace/source"), "source bind was not passed to Docker: {commands}");
    assert!(!commands.contains("chown -h"), "Windows invoked POSIX ownership cleanup: {commands}");
    assert!(commands.contains("image tag example/web"), "Compose tag was not created: {commands}");
    assert!(commands.contains("image save --output"), "Compose images were not saved: {commands}");
    assert!(
        commands.contains("image rm bonesdeploy-windows-docker-test-web-"),
        "Compose tag was not removed: {commands}"
    );
    Ok(())
}

#[test]
fn windows_docker_rejects_remote_and_unstable_selectors_before_mutation() -> Result<()> {
    let project = tempfile::tempdir()?;
    initialize_project(project.path())?;
    let tools = tempfile::tempdir()?;
    let docker_log = tools.path().join("docker.log");
    write_fake_docker(tools.path())?;

    let fixture = WindowsDockerFixture { project: project.path(), tools: tools.path(), log: &docker_log };
    let rejected =
        run_helper(fixture, "windows_rejected_endpoint_helper", [("DOCKER_HOST", "tcp://daemon.example:2376")])?;
    anyhow::ensure!(rejected.status.success(), "rejected-endpoint helper failed");
    assert!(fs::read_to_string(&docker_log).unwrap_or_default().is_empty(), "remote endpoint mutated Docker");

    let marker = tools.path().join("selector-change-ready");
    let stable = run_helper(
        fixture,
        "windows_selector_change_helper",
        [("FAKE_DOCKER_CHANGE_MARKER", marker.to_string_lossy().as_ref())],
    )?;
    anyhow::ensure!(stable.status.success(), "selector-change helper failed");
    let commands = fs::read_to_string(docker_log)?;
    assert!(commands.contains("info --format {{.OSType}}"), "availability was not checked: {commands}");
    assert!(!commands.contains("image inspect"), "changed selector reached Docker: {commands}");
    Ok(())
}

#[test]
fn windows_docker_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_WINDOWS_DOCKER_HELPER").is_none() {
        return Ok(());
    }
    let mut config = Bones::for_site("windows-docker-test");
    config.branch = "main".into();
    build::package(&config)?;
    config.runtime.backend = RuntimeBackend::Docker;
    build::package(&config)?;
    Ok(())
}

#[test]
fn windows_rejected_endpoint_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_WINDOWS_DOCKER_HELPER").is_some() {
        anyhow::ensure!(package_native().is_err(), "remote Docker endpoint was accepted");
    }
    Ok(())
}

#[test]
fn windows_selector_change_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_WINDOWS_DOCKER_HELPER").is_none() {
        return Ok(());
    }
    let marker = PathBuf::from(env::var("FAKE_DOCKER_CHANGE_MARKER")?);
    let changer = thread::spawn(move || {
        for _ in 0..100 {
            if marker.exists() {
                // SAFETY: The isolated helper joins this thread before it exits.
                unsafe { env::set_var("DOCKER_CONTEXT", "changed-selector") };
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    });
    let result = package_native();
    anyhow::ensure!(
        changer.join().map_err(|_| anyhow::anyhow!("selector changer panicked"))?,
        "Docker info did not start"
    );
    anyhow::ensure!(result.is_err(), "changed Docker selector was accepted");
    Ok(())
}

fn package_native() -> Result<build::PackagedArtifact> {
    let mut config = Bones::for_site("windows-docker-test");
    config.branch = "main".into();
    build::package(&config)
}

#[derive(Clone, Copy)]
struct WindowsDockerFixture<'a> {
    project: &'a Path,
    tools: &'a Path,
    log: &'a Path,
}

fn run_helper<'a, I>(fixture: WindowsDockerFixture<'_>, helper: &str, variables: I) -> Result<Output>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let path = env::join_paths([fixture.tools.to_path_buf(), env::var_os("PATH").unwrap_or_default().into()])?;
    Command::new(env::current_exe()?)
        .args(["--exact", helper, "--nocapture"])
        .current_dir(fixture.project)
        .env("BONESDEPLOY_WINDOWS_DOCKER_HELPER", "1")
        .env("DOCKER_LOG", fixture.log)
        .env("DOCKER_CONTEXT", "desktop-local")
        .env("FAKE_DOCKER_ENDPOINT", LOCAL_PIPE)
        .env("PATH", path)
        .env("LOCALAPPDATA", fixture.tools.join("local-app-data"))
        .env_remove("DOCKER_HOST")
        .envs(variables)
        .output()
        .context("run Windows Docker helper")
}

fn initialize_project(project: &Path) -> Result<()> {
    run_git(project, ["init", "--initial-branch=main"])?;
    run_git(project, ["config", "user.email", "test@example.com"])?;
    run_git(project, ["config", "user.name", "Test"])?;
    fs::create_dir_all(project.join("infra/deployment/build"))?;
    fs::write(project.join("infra/deployment/build/01_build.sh"), "true\n")?;
    fs::write(project.join("compose.yaml"), "services:\n  web:\n    build: .\n")?;
    run_git(project, ["add", "."])?;
    run_git(project, ["commit", "-m", "initial"])
}

fn write_fake_docker(tools: &Path) -> Result<()> {
    fs::write(
        tools.join("docker.cmd"),
        r#"@echo off
if "%1"=="context" (
  if "%2"=="show" (echo default& exit /b 0)
  echo %FAKE_DOCKER_ENDPOINT%
  exit /b 0
)
echo %DOCKER_HOST%^|%DOCKER_CONTEXT%^|%*>> "%DOCKER_LOG%"
if "%1"=="info" (
  if not "%FAKE_DOCKER_CHANGE_MARKER%"=="" (type nul > "%FAKE_DOCKER_CHANGE_MARKER%" & timeout /t 1 /nobreak >nul)
  echo linux
  exit /b 0
)
if "%1"=="compose" (
  echo {"services":{"web":{"image":"example/web"}}}
  exit /b 0
)
if "%1"=="image" if "%2"=="save" type nul > "%4"
exit /b 0
"#,
    )?;
    Ok(())
}

fn run_git<const N: usize>(project: &Path, arguments: [&str; N]) -> Result<()> {
    let status = Command::new("git").arg("-C").arg(project).args(arguments).status().context("run git")?;
    anyhow::ensure!(status.success(), "git command failed");
    Ok(())
}
