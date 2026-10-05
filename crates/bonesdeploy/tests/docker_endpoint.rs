#![cfg(unix)]

use std::env;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use bonesdeploy::build;
use bonesdeploy_core::config::{Bones, RuntimeBackend};
use flate2::read::GzDecoder;
use tar::Archive;

const LOCAL_ENDPOINT: &str = "unix:///tmp/bonesdeploy-local.sock";

#[test]
fn local_non_default_context_binds_native_and_compose_commands_to_one_endpoint() -> Result<()> {
    let project = tempfile::tempdir()?;
    initialize_project(project.path())?;
    let tools = tempfile::tempdir()?;
    let docker_log = tools.path().join("docker.log");
    write_fake_docker(tools.path())?;

    let output = run_helper(
        DockerFixture { project: project.path(), tools: tools.path(), docker_log: &docker_log },
        "local_context_helper",
        [("DOCKER_CONTEXT", "desktop-local"), ("FAKE_DOCKER_ENDPOINT", LOCAL_ENDPOINT)],
    )?;
    assert!(
        !String::from_utf8_lossy(&output.stdout).contains("builder-image-inspect-output"),
        "builder image inspection output leaked to the terminal"
    );

    let commands = fs::read_to_string(docker_log)?;
    assert!(!commands.is_empty(), "fake Docker received no commands");
    for command in commands.lines() {
        assert!(command.starts_with(&format!("{LOCAL_ENDPOINT}|")), "command used another endpoint: {command}");
        assert!(!command.contains("|desktop-local|"), "command retained the context selector: {command}");
    }
    assert!(commands.contains("|rm --force bonesdeploy-build-"), "native cleanup was not bound: {commands}");
    assert!(commands.contains("|image tag example/web"), "Compose tag was not bound: {commands}");
    assert!(commands.contains("|image save --output"), "Compose save was not bound: {commands}");
    Ok(())
}

#[test]
fn remote_and_invalid_docker_selectors_fail_before_docker_mutation() -> Result<()> {
    for (name, selector, endpoint) in [
        ("tcp", Some(("DOCKER_HOST", "tcp://daemon.example:2376")), None),
        ("ssh", Some(("DOCKER_HOST", "ssh://daemon.example")), None),
        ("npipe", Some(("DOCKER_HOST", "npipe:////./pipe/docker_engine")), None),
        ("malformed", Some(("DOCKER_HOST", "not-an-endpoint")), None),
        ("whitespace", Some(("DOCKER_HOST", "unix:///tmp/docker socket")), None),
        ("newline", Some(("DOCKER_HOST", "unix:///tmp/docker.sock\nremote")), None),
        ("control", Some(("DOCKER_HOST", "unix:///tmp/docker.sock\u{0007}")), None),
        ("remote_context", Some(("DOCKER_CONTEXT", "remote")), Some("tcp://daemon.example:2376")),
        ("missing_context", Some(("DOCKER_CONTEXT", "missing")), Some("")),
        ("whitespace_context", Some(("DOCKER_CONTEXT", "whitespace")), Some("unix:///tmp/docker socket")),
        ("newline_context", Some(("DOCKER_CONTEXT", "newline")), Some("unix:///tmp/docker.sock\nremote")),
    ] {
        let project = tempfile::tempdir()?;
        initialize_project(project.path())?;
        let tools = tempfile::tempdir()?;
        let docker_log = tools.path().join("docker.log");
        write_fake_docker(tools.path())?;
        let mut variables = vec![("DOCKER_ENDPOINT_CASE", name)];
        if let Some((key, value)) = selector {
            variables.push((key, value));
        }
        if let Some(endpoint) = endpoint {
            variables.push(("FAKE_DOCKER_ENDPOINT", endpoint));
        }

        run_helper(
            DockerFixture { project: project.path(), tools: tools.path(), docker_log: &docker_log },
            "rejected_selector_helper",
            variables,
        )?;
        let commands = fs::read_to_string(&docker_log).unwrap_or_default();
        assert!(commands.is_empty(), "{name} selector reached a Docker mutation: {commands}");
    }
    Ok(())
}

#[test]
fn docker_host_change_stops_the_next_docker_command() -> Result<()> {
    selector_change_stops_next_command("DOCKER_HOST", LOCAL_ENDPOINT)
}

#[test]
fn docker_context_change_stops_the_next_docker_command() -> Result<()> {
    selector_change_stops_next_command("DOCKER_CONTEXT", "desktop-local")
}

fn selector_change_stops_next_command(selector: &str, initial_value: &str) -> Result<()> {
    let project = tempfile::tempdir()?;
    initialize_project(project.path())?;
    let tools = tempfile::tempdir()?;
    let docker_log = tools.path().join("docker.log");
    let marker = tools.path().join("selector-change-ready");
    write_fake_docker(tools.path())?;

    run_helper(
        DockerFixture { project: project.path(), tools: tools.path(), docker_log: &docker_log },
        "selector_change_helper",
        [
            (selector, initial_value),
            ("FAKE_DOCKER_CHANGE_MARKER", marker.to_string_lossy().as_ref()),
            ("SELECTOR_TO_CHANGE", selector),
        ],
    )?;

    let commands = fs::read_to_string(docker_log)?;
    assert!(commands.contains("|info --format {{.OSType}}"), "Docker availability was not checked: {commands}");
    assert!(!commands.contains("|image inspect"), "selector change did not stop the next Docker command: {commands}");
    Ok(())
}

#[test]
fn local_context_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_DOCKER_ENDPOINT_HELPER").is_none() {
        return Ok(());
    }
    let native = package_native()?;
    assert_selector_variables_are_absent(&native, ".bonesdeploy-native-env-capture")?;
    let compose = package_compose()?;
    assert_selector_variables_are_absent(&compose, ".bonesdeploy-compose-env-capture")
}

#[test]
fn rejected_selector_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_DOCKER_ENDPOINT_HELPER").is_none() {
        return Ok(());
    }
    anyhow::ensure!(package_native().is_err(), "invalid Docker selector was accepted");
    Ok(())
}

#[test]
fn selector_change_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_DOCKER_ENDPOINT_HELPER").is_none() {
        return Ok(());
    }
    let marker = PathBuf::from(env::var("FAKE_DOCKER_CHANGE_MARKER")?);
    let selector = env::var("SELECTOR_TO_CHANGE")?;
    let changer = thread::spawn(move || {
        for _ in 0..100 {
            if marker.exists() {
                // SAFETY: This helper is an isolated process and waits for this thread before returning.
                unsafe { env::set_var(selector, "changed-selector") };
                return true;
            }
            thread::sleep(Duration::from_millis(10));
        }
        false
    });
    let result = package_native();
    let selector_changed = changer.join().map_err(|_| anyhow::anyhow!("selector-changing thread panicked"))?;
    anyhow::ensure!(selector_changed, "fake Docker did not begin its availability check");
    anyhow::ensure!(result.is_err(), "Docker selector change was accepted");
    Ok(())
}

fn package_native() -> Result<build::PackagedArtifact> {
    let mut config = Bones::for_site("docker-endpoint-test");
    config.branch = "main".into();
    build::package(&config)
}

fn package_compose() -> Result<build::PackagedArtifact> {
    let mut config = Bones::for_site("docker-endpoint-test");
    config.branch = "main".into();
    config.runtime.backend = RuntimeBackend::Docker;
    build::package(&config)
}

fn assert_selector_variables_are_absent(artifact: &build::PackagedArtifact, capture: &str) -> Result<()> {
    let file = File::open(artifact.path())?;
    let mut archive = Archive::new(GzDecoder::new(file));
    for entry in archive.entries()? {
        let mut entry = entry?;
        if entry.path()? == Path::new(capture) {
            let mut contents = String::new();
            entry.read_to_string(&mut contents)?;
            anyhow::ensure!(
                !contents.lines().any(|line| line.starts_with("DOCKER_HOST=") || line.starts_with("DOCKER_CONTEXT=")),
                "Docker selector variable leaked into {capture}: {contents}"
            );
            return Ok(());
        }
    }
    anyhow::bail!("fake Docker did not capture {capture}")
}

#[derive(Clone, Copy)]
struct DockerFixture<'a> {
    project: &'a Path,
    tools: &'a Path,
    docker_log: &'a Path,
}

fn run_helper<'a, I>(fixture: DockerFixture<'_>, helper: &str, variables: I) -> Result<Output>
where
    I: IntoIterator<Item = (&'a str, &'a str)>,
{
    let path = format!("{}:{}", fixture.tools.display(), env::var("PATH").unwrap_or_default());
    let output = Command::new(env::current_exe()?)
        .args(["--exact", helper, "--nocapture"])
        .current_dir(fixture.project)
        .env("BONESDEPLOY_DOCKER_ENDPOINT_HELPER", "1")
        .env("DOCKER_LOG", fixture.docker_log)
        .env("PATH", path)
        .env("XDG_CACHE_HOME", fixture.tools.join("cache"))
        .env_remove("DOCKER_HOST")
        .env_remove("DOCKER_CONTEXT")
        .envs(variables)
        .output()?;
    anyhow::ensure!(
        output.status.success(),
        "helper {helper} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output)
}

fn initialize_project(project: &Path) -> Result<()> {
    run_git(project, ["init", "--initial-branch=main"])?;
    run_git(project, ["config", "user.email", "test@example.com"])?;
    run_git(project, ["config", "user.name", "Test"])?;
    let build = project.join("infra/deployment/build");
    fs::create_dir_all(&build)?;
    fs::write(build.join("01_build.sh"), "true\n")?;
    fs::write(project.join("compose.yaml"), "services:\n  web:\n    build: .\n")?;
    run_git(project, ["add", "."])?;
    run_git(project, ["commit", "-m", "initial"])
}

fn write_fake_docker(tools: &Path) -> Result<()> {
    let docker = tools.join("docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
if [ "$1" = context ]; then
	if [ "$2" = show ]; then
		printf '%s\n' default
		exit 0
	fi
	if [ "${FAKE_DOCKER_ENDPOINT+x}" = x ]; then
		printf '%s\n' "$FAKE_DOCKER_ENDPOINT"
	else
		printf '%s\n' 'unix:///tmp/bonesdeploy-local.sock'
	fi
	exit 0
fi
printf '%s|%s|%s\n' "${DOCKER_HOST-unset}" "${DOCKER_CONTEXT-unset}" "$*" >>"$DOCKER_LOG"
environment_file=
previous=
for argument in "$@"; do
	if [ "$previous" = --env-file ]; then
		environment_file=$argument
	fi
	previous=$argument
done
if [ "$1" = run ] && [ -n "$environment_file" ]; then
	cp "$environment_file" .bonesdeploy-native-env-capture
fi
if [ "$1" = compose ]; then
	cp "$environment_file" .bonesdeploy-compose-env-capture
fi
if [ "$1" = info ]; then
	if [ -n "${FAKE_DOCKER_CHANGE_MARKER:-}" ]; then
		: >"$FAKE_DOCKER_CHANGE_MARKER"
		sleep 1
	fi
	printf 'linux\n'
	exit 0
fi
if [ "$1" = image ] && [ "$2" = inspect ]; then
	printf '%s\n' 'builder-image-inspect-output'
	exit 0
fi
if [ "$1" = compose ]; then
	case " $* " in
		*" --format json "*) printf '%s\n' '{"services":{"web":{"image":"example/web"}}}' ;;
	esac
	exit 0
fi
if [ "$1" = image ] && [ "$2" = save ]; then
	: >"$4"
fi
for argument in "$@"; do
	[ "$argument" = -i ] && cat >/dev/null
done
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
