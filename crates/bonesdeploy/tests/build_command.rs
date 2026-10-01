mod common;

use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use common::TestEnv;

const INIT_ARGS: &[&str] = &[
    "init",
    "--non-interactive",
    "--project-name",
    "atlas",
    "--branch",
    "master",
    "--host",
    "unreachable.invalid",
    "--template",
    "none",
];

#[test]
fn build_packages_the_configured_commit_without_contacting_the_configured_host() -> Result<()> {
    let env = TestEnv::new()?;
    let init = env.run(INIT_ARGS)?;
    assert!(init.status.success(), "init failed: {}", String::from_utf8_lossy(&init.stderr));

    fs::write(env.repo().join("application.txt"), "application")?;
    commit_all(env.repo())?;

    let output = env.run(&["build"])?;

    assert!(output.status.success(), "build failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Built atlas at"), "missing build result: {stdout}");
    assert!(stdout.contains("bytes)."), "missing artifact size: {stdout}");
    Ok(())
}

#[test]
fn build_reports_a_missing_configured_branch_without_contacting_the_configured_host() -> Result<()> {
    let env = TestEnv::new()?;
    let init = env.run(INIT_ARGS)?;
    assert!(init.status.success(), "init failed: {}", String::from_utf8_lossy(&init.stderr));

    let env_file = env.repo().join(".env");
    let content = fs::read_to_string(&env_file)?;
    fs::write(&env_file, content.replace("BONES_BRANCH=master", "BONES_BRANCH=missing"))?;

    let output = env.run(&["build"])?;

    assert!(!output.status.success(), "missing branch must fail");
    assert!(String::from_utf8_lossy(&output.stderr).contains("missing"));
    Ok(())
}

fn commit_all(repository: &Path) -> Result<()> {
    let status = Command::new("git")
        .args(["-C"])
        .arg(repository)
        .args(["add", "."])
        .status()
        .context("failed to stage test project")?;
    anyhow::ensure!(status.success(), "git add failed with status {status}");

    let status = Command::new("git")
        .args(["-C"])
        .arg(repository)
        .args(["-c", "user.name=BonesDeploy", "-c", "user.email=bonesdeploy@local", "commit", "-m", "application"])
        .status()
        .context("failed to commit test project")?;
    anyhow::ensure!(status.success(), "git commit failed with status {status}");
    Ok(())
}
