use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Output};

use anyhow::{Context, Result};
use bonesdeploy::build;
use bonesdeploy_core::config::Bones;

#[test]
fn native_builder_reuses_one_root_container_and_restores_mount_ownership() -> Result<()> {
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
        .env("DOCKER_HOST", "unix:///tmp/bonesdeploy-native-build.sock")
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
    assert_eq!(commands.lines().filter(|line| line.starts_with("run --detach ")).count(), 1, "{commands}");
    assert_eq!(commands.lines().filter(|line| line.starts_with("exec -i ")).count(), 2, "{commands}");
    assert!(commands.contains("find -P /workspace/source /workspace/cache -exec chown -h"));
    assert!(commands.lines().any(|line| line.starts_with("rm --force bonesdeploy-build-")), "{commands}");
    Ok(())
}

#[test]
fn timed_out_native_script_stops_workload_before_restoring_ownership_and_removing_container() -> Result<()> {
    let (output, commands) = run_native_helper("timeout")?;

    anyhow::ensure!(!output.status.success(), "timed-out build unexpectedly succeeded");
    let stop = commands.find("stop --time 10 bonesdeploy-build-").context("timed-out workload was not stopped")?;
    let stopped = commands.find("workload-stopped").context("timed-out workload stop did not complete")?;
    let ownership = commands
        .find("run --rm --pull=never --platform linux/amd64 --volume")
        .context("cleanup container did not restore ownership")?;
    let removal = commands.find("rm --force bonesdeploy-build-").context("stale container was not removed")?;
    assert!(stop < stopped && stopped < ownership && ownership < removal, "cleanup order was incorrect: {commands}");
    assert!(String::from_utf8_lossy(&output.stderr).contains("exceeded its 1-second timeout"));
    Ok(())
}

#[test]
fn native_failure_reports_bounded_command_stdout_and_stderr() -> Result<()> {
    let (output, _) = run_native_helper("failure")?;

    anyhow::ensure!(!output.status.success(), "failing build unexpectedly succeeded");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("native stdout diagnostic"), "{stderr}");
    assert!(stderr.contains("native stderr diagnostic"), "{stderr}");
    Ok(())
}

#[test]
fn native_failure_preserves_the_primary_error_when_ownership_cleanup_also_fails() -> Result<()> {
    let (output, _) = run_native_helper("cleanup-failure")?;

    anyhow::ensure!(!output.status.success(), "failing build unexpectedly succeeded");
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("primary native failure"), "{stderr}");
    assert!(stderr.contains("Local build cleanup also failed"), "{stderr}");
    assert!(stderr.contains("ownership cleanup failed"), "{stderr}");
    Ok(())
}

#[test]
fn native_failure_drains_verbose_output_and_reports_only_the_bounded_tail() -> Result<()> {
    let (output, _, error) = run_native_helper_with_error_log("verbose")?;

    anyhow::ensure!(!output.status.success(), "failing build unexpectedly succeeded");
    assert!(error.contains("stdout tail end"), "{error}");
    assert!(error.contains("stderr tail end"), "{error}");
    assert!(!error.contains("stdout tail beginning"), "{error}");
    assert!(!error.contains("stderr tail beginning"), "{error}");
    assert_eq!(error.matches('@').count(), 65_519, "{error}");
    assert_eq!(error.matches('#').count(), 65_519, "{error}");
    Ok(())
}

#[test]
fn native_cleanup_uses_a_short_lived_container_when_the_builder_disappears() -> Result<()> {
    let (output, commands) = run_native_helper("builder-disappeared")?;

    anyhow::ensure!(!output.status.success(), "failing build unexpectedly succeeded");
    assert!(commands.contains("exec bonesdeploy-build-"), "{commands}");
    assert!(commands.contains("run --rm --pull=never --platform linux/amd64 --volume"), "{commands}");
    Ok(())
}

#[test]
fn native_removal_failure_is_retried_by_drop() -> Result<()> {
    let (output, commands) = run_native_helper("removal-failure")?;

    anyhow::ensure!(!output.status.success(), "removal failure unexpectedly succeeded");
    assert_eq!(
        commands.lines().filter(|line| line.starts_with("rm --force bonesdeploy-build-")).count(),
        2,
        "{commands}"
    );
    Ok(())
}

#[test]
fn native_builder_helper() -> Result<()> {
    if env::var_os("BONESDEPLOY_NATIVE_BUILD_HELPER").is_none() {
        return Ok(());
    }
    let mut config = Bones::for_site("native-root-test");
    config.branch = "main".into();
    let scenario = env::var("BONESDEPLOY_NATIVE_SCENARIO").unwrap_or_else(|_| "success".into());
    if scenario == "timeout" {
        config.build.timeout_seconds = 1;
    }
    let artifact = match build::package(&config) {
        Ok(artifact) => artifact,
        Err(error) => {
            if let Some(path) = env::var_os("BONESDEPLOY_NATIVE_ERROR_LOG") {
                fs::write(path, format!("{error:#}")).context("write native build error log")?;
            }
            eprintln!("{error:#}");
            return Err(error);
        }
    };
    anyhow::ensure!(artifact.path().is_file(), "build artifact was not created");
    Ok(())
}

fn run_native_helper(scenario: &str) -> Result<(Output, String)> {
    let (output, commands, _) = run_native_helper_with_error_log(scenario)?;
    Ok((output, commands))
}

fn run_native_helper_with_error_log(scenario: &str) -> Result<(Output, String, String)> {
    let project = tempfile::tempdir()?;
    initialize_project(project.path())?;
    let tools = tempfile::tempdir()?;
    let docker_log = tools.path().join("docker.log");
    let error_log = tools.path().join("error.log");
    let removal_count = tools.path().join("removal-count");
    write_fake_docker(tools.path())?;
    let path = format!("{}:{}", tools.path().display(), env::var("PATH").unwrap_or_default());
    let output = Command::new(env::current_exe()?)
        .args(["--exact", "native_builder_helper", "--nocapture"])
        .current_dir(project.path())
        .env("BONESDEPLOY_NATIVE_BUILD_HELPER", "1")
        .env("BONESDEPLOY_NATIVE_SCENARIO", scenario)
        .env("BONESDEPLOY_NATIVE_ERROR_LOG", &error_log)
        .env("DOCKER_LOG", &docker_log)
        .env("REMOVAL_COUNT", &removal_count)
        .env("PATH", path)
        .env("XDG_CACHE_HOME", tools.path().join("cache"))
        .output()
        .context("run native build helper")?;
    Ok((output, fs::read_to_string(docker_log)?, fs::read_to_string(error_log).unwrap_or_default()))
}

fn initialize_project(project: &Path) -> Result<()> {
    run_git(project, ["init", "--initial-branch=main"])?;
    run_git(project, ["config", "user.email", "test@example.com"])?;
    run_git(project, ["config", "user.name", "Test"])?;
    let build = project.join("infra/deployment/build");
    fs::create_dir_all(&build)?;
    fs::write(build.join("01_requires_root.sh"), "test \"$(id -u)\" = 0\n")?;
    fs::write(build.join("02_reuses_container.sh"), "true\n")?;
    run_git(project, ["add", "."])?;
    run_git(project, ["commit", "-m", "initial"])
}

fn write_fake_docker(tools: &Path) -> Result<()> {
    let docker = tools.join("docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
printf '%s\n' "$*" >>"$DOCKER_LOG"
if [ "$1" = context ]; then
	printf 'unix:///tmp/bonesdeploy-native-build.sock\n'
	exit 0
fi
if [ "$1" = info ]; then
	printf 'linux\n'
	exit 0
fi
if [ "$1" = image ] && [ "$2" = inspect ]; then
	exit 0
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = timeout ] && [ "$1" = exec ] && [ "$2" = -i ]; then
	exec sleep 10
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = timeout ] && [ "$1" = stop ]; then
	sleep 1
	printf 'workload-stopped\n' >>"$DOCKER_LOG"
	exit 0
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = failure ] && [ "$1" = exec ] && [ "$2" = -i ]; then
	printf 'native stdout diagnostic\n'
	printf 'native stderr diagnostic\n' >&2
	exit 23
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = cleanup-failure ] && [ "$1" = exec ] && [ "$2" = -i ]; then
	printf 'primary native failure\n' >&2
	exit 23
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = cleanup-failure ] && [ "$1" = exec ]; then
	printf 'ownership cleanup failed\n' >&2
	exit 91
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = cleanup-failure ] && [ "$1" = run ]; then
	case "$*" in *' find '*) printf 'cleanup container ownership failed\n' >&2; exit 92 ;; esac
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = builder-disappeared ] && [ "$1" = exec ] && [ "$2" = -i ]; then
	printf 'builder disappeared\n' >&2
	exit 23
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = builder-disappeared ] && [ "$1" = exec ]; then
	printf 'builder is gone\n' >&2
	exit 91
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = verbose ] && [ "$1" = exec ] && [ "$2" = -i ]; then
	(
		printf 'stdout tail beginning\n'
		head -c 70000 /dev/zero | tr '\0' @
		printf '\nstdout tail end\n'
	) &
	(
		printf 'stderr tail beginning\n' >&2
		head -c 70000 /dev/zero | tr '\0' '#' >&2
		printf '\nstderr tail end\n' >&2
	) &
	wait
	exit 23
fi
if [ "${BONESDEPLOY_NATIVE_SCENARIO:-}" = removal-failure ] && [ "$1" = rm ]; then
	count=0
	[ -f "$REMOVAL_COUNT" ] && count=$(cat "$REMOVAL_COUNT")
	count=$((count + 1))
	printf '%s\n' "$count" >"$REMOVAL_COUNT"
	[ "$count" -eq 1 ] && exit 91
fi
for argument in "$@"; do
	[ "$argument" = --user ] && exit 97
	[ "$argument" = --time ] && exit 98
	[ "$argument" = -i ] && interactive=1
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
