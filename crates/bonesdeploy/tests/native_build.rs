use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result};
use bonesdeploy::build;
use bonesdeploy_core::config::Bones;

#[test]
fn native_builder_runs_as_container_root_and_restores_mount_ownership() -> Result<()> {
    let project = tempfile::tempdir()?;
    initialize_project(project.path())?;
    let tools = tempfile::tempdir()?;
    let docker_log = tools.path().join("docker.log");
    write_fake_docker(tools.path())?;
    let path = format!("{}:{}", tools.path().display(), env::var("PATH").unwrap_or_default());

    let output = Command::new(env::current_exe()?)
        .args(["--exact", "native_builder_helper", "--nocapture"])
        .current_dir(project.path())
        .env("BONESDEPLOY_NATIVE_BUILD_HELPER", "1")
        .env("DOCKER_LOG", &docker_log)
        .env("PATH", path)
        .env("XDG_CACHE_HOME", tools.path().join("cache"))
        .output()?;

    anyhow::ensure!(
        output.status.success(),
        "helper failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let commands = fs::read_to_string(docker_log)?;
    assert!(!commands.contains("--user"), "build command must retain container root: {commands}");
    assert!(commands.contains("find -P /workspace/source /workspace/cache -exec chown -h"));
    Ok(())
}

#[test]
fn native_builder_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_NATIVE_BUILD_HELPER").is_none() {
        return Ok(());
    }
    let mut config = Bones::for_site("native-root-test");
    config.branch = "main".into();
    let artifact = build::package(&config)?;
    anyhow::ensure!(artifact.path().is_file(), "build artifact was not created");
    Ok(())
}

fn initialize_project(project: &Path) -> Result<()> {
    run_git(project, ["init", "--initial-branch=main"])?;
    run_git(project, ["config", "user.email", "test@example.com"])?;
    run_git(project, ["config", "user.name", "Test"])?;
    let build = project.join("infra/deployment/build");
    fs::create_dir_all(&build)?;
    fs::write(build.join("01_requires_root.sh"), "test \"$(id -u)\" = 0\n")?;
    run_git(project, ["add", "."])?;
    run_git(project, ["commit", "-m", "initial"])
}

fn write_fake_docker(tools: &Path) -> Result<()> {
    let docker = tools.join("docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
printf '%s\n' "$*" >>"$DOCKER_LOG"
if [ "$1" = info ]; then
	printf 'linux\n'
	exit 0
fi
if [ "$1" = image ] && [ "$2" = inspect ]; then
	exit 0
fi
for argument in "$@"; do
	[ "$argument" = --user ] && exit 97
	[ "$argument" = --interactive ] && interactive=1
done
[ "${interactive:-0}" = 1 ] && cat >/dev/null
exit 0
"#,
    )?;
    fs::set_permissions(docker, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn run_git<const N: usize>(project: &Path, arguments: [&str; N]) -> Result<()> {
    let status = Command::new("git").arg("-C").arg(project).args(arguments).status().context("run git")?;
    anyhow::ensure!(status.success(), "git command failed");
    Ok(())
}
