use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{Context, Result};
use bonesdeploy::build;
use bonesdeploy_core::config::{Bones, RuntimeBackend};

#[test]
fn compose_build_removes_generated_release_tags_after_saving_images() -> Result<()> {
    let (output, commands) = run_compose_helper("success")?;

    anyhow::ensure!(output.status.success(), "helper failed: {}", String::from_utf8_lossy(&output.stderr));
    let tag = commands.find("image tag bonesdeploy-compose-test-web").context("release tag was not created")?;
    let save = commands.find("image save --output").context("release images were not saved")?;
    let remove = commands.find("image rm bonesdeploy-compose-test-web-").context("release tag was not removed")?;
    assert!(tag < save && save < remove, "tag lifecycle order was incorrect: {commands}");
    Ok(())
}

#[test]
fn compose_build_removes_created_tags_and_preserves_diagnostics_when_image_save_fails() -> Result<()> {
    let (output, commands) = run_compose_helper("save-failure")?;

    anyhow::ensure!(!output.status.success(), "failing helper unexpectedly succeeded");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("compose save stdout"), "{stderr}");
    assert!(stderr.contains("compose save stderr"), "{stderr}");
    assert!(commands.contains("image rm bonesdeploy-compose-test-web-"), "{commands}");
    Ok(())
}

#[test]
fn compose_tag_failure_removes_tags_created_before_the_failure() -> Result<()> {
    let (output, commands) = run_compose_helper("tag-failure")?;

    anyhow::ensure!(!output.status.success(), "failing helper unexpectedly succeeded");
    assert!(String::from_utf8_lossy(&output.stderr).contains("tag image for service `worker`"));
    assert!(commands.contains("image rm bonesdeploy-compose-test-web-"), "{commands}");
    assert!(!commands.contains("image rm bonesdeploy-compose-test-worker-"), "{commands}");
    Ok(())
}

#[test]
fn compose_cleanup_failure_preserves_the_post_tagging_operation_error() -> Result<()> {
    let (output, commands) = run_compose_helper("save-cleanup-failure")?;

    anyhow::ensure!(!output.status.success(), "failing helper unexpectedly succeeded");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("compose save stderr"), "{stderr}");
    assert!(stderr.contains("Generated Compose release tag cleanup also failed"), "{stderr}");
    assert!(stderr.contains("Compose tag cleanup failed"), "{stderr}");
    assert!(commands.contains("image rm bonesdeploy-compose-test-web-"), "{commands}");
    Ok(())
}

#[test]
fn compose_operations_honor_configured_deadlines_and_allow_an_explicitly_unbounded_timeout() -> Result<()> {
    let (timed_out, _) = run_compose_helper("timeout")?;
    anyhow::ensure!(!timed_out.status.success(), "timed-out helper unexpectedly succeeded");
    assert!(String::from_utf8_lossy(&timed_out.stderr).contains("exceeded its 1-second timeout"));

    let (unbounded, _) = run_compose_helper("unbounded")?;
    anyhow::ensure!(
        unbounded.status.success(),
        "unbounded helper failed: {}",
        String::from_utf8_lossy(&unbounded.stderr)
    );
    Ok(())
}

#[test]
fn compose_builder_helper() -> Result<()> {
    let Some(scenario) = env::var_os("BONESDEPLOY_COMPOSE_SCENARIO") else {
        return Ok(());
    };
    let mut config = Bones::for_site("compose-test");
    config.branch = "main".into();
    config.runtime.backend = RuntimeBackend::Docker;
    if scenario == "timeout" {
        config.build.timeout_seconds = 1;
    }
    if scenario == "unbounded" {
        config.build.timeout_seconds = 0;
    }
    let artifact = build::package(&config).map_err(|error| {
        eprintln!("{error:#}");
        error
    })?;
    anyhow::ensure!(artifact.path().is_file(), "Compose artifact was not created");
    Ok(())
}

fn run_compose_helper(scenario: &str) -> Result<(Output, String)> {
    let project = tempfile::tempdir()?;
    initialize_project(project.path())?;
    let tools = tempfile::tempdir()?;
    let docker_dir = tools.path().join(scenario);
    fs::create_dir(&docker_dir)?;
    let docker_log = tools.path().join(format!("{scenario}.log"));
    write_fake_docker(&docker_dir)?;
    let path = format!("{}:{}", docker_dir.display(), env::var("PATH").unwrap_or_default());
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "compose_builder_helper", "--nocapture"])
        .current_dir(project.path())
        .env("BONESDEPLOY_COMPOSE_SCENARIO", scenario)
        .env("DOCKER_LOG", &docker_log)
        .env("DOCKER_HOST", "unix:///tmp/bonesdeploy-compose-build.sock")
        .env("PATH", path)
        .env("XDG_CACHE_HOME", tools.path().join("cache"))
        .output()
        .context("run Compose build helper")?;
    Ok((output, fs::read_to_string(docker_log).unwrap_or_default()))
}

fn initialize_project(project: &Path) -> Result<()> {
    run_git(project, ["init", "--initial-branch=main"])?;
    run_git(project, ["config", "user.email", "test@example.com"])?;
    run_git(project, ["config", "user.name", "Test"])?;
    fs::write(project.join("compose.yaml"), "services:\n  web:\n    build: .\n")?;
    run_git(project, ["add", "."])?;
    run_git(project, ["commit", "-m", "initial"])
}

fn write_fake_docker(tools: &Path) -> Result<()> {
    let docker = tools.join("docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
printf '%s\n' "$*" >>"$DOCKER_LOG"
if [ "$1" = compose ]; then
	for argument in "$@"; do last=$argument; done
	if [ "$last" = --quiet ] || [ "$last" = build ]; then exit 0; fi
	if [ "$last" = pull ]; then
		case "$0" in *timeout*) exec sleep 10 ;; *unbounded*) sleep 2 ;; esac
		exit 0
	fi
	if [ "$last" = json ]; then
		case "$0" in
			*tag-failure*) printf '{"services":{"web":{"image":"bonesdeploy-compose-test-web","platform":"linux/amd64"},"worker":{"image":"bonesdeploy-compose-test-worker","platform":"linux/amd64"}}}\n' ;;
			*) printf '{"services":{"web":{"image":"bonesdeploy-compose-test-web","platform":"linux/amd64"}}}\n' ;;
		esac
		exit 0
	fi
fi
if [ "$1" = image ] && [ "$2" = tag ]; then
	case "$0" in *tag-failure*) case "$4" in *worker*) printf 'worker tagging failed\n' >&2; exit 43 ;; esac ;; esac
fi
if [ "$1" = image ] && [ "$2" = save ]; then
	case "$0" in *save-failure*|*cleanup-failure*)
		printf 'compose save stdout\n'
		printf 'compose save stderr\n' >&2
		exit 42
		;; esac
	output=
	while [ "$#" -gt 0 ]; do
		if [ "$1" = --output ]; then output=$2; break; fi
		shift
	done
	: >"$output"
fi
if [ "$1" = image ] && [ "$2" = rm ]; then
	case "$0" in *cleanup-failure*) printf 'Compose tag cleanup failed\n' >&2; exit 77 ;; esac
fi
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
