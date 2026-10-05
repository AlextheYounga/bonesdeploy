#![cfg(unix)]

use std::env;
use std::ffi::OsString;
use std::fs;
use std::iter;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use bonesdeploy::build;
use bonesdeploy_core::config::Bones;

#[test]
fn same_site_package_fails_immediately_while_another_package_is_building() -> Result<()> {
    let fixture = PackageFixture::new()?;
    let mut first = fixture.start_package("atlas", "atlas-first", true)?;
    fixture.wait_until_ready("atlas-first")?;
    fixture.release("atlas-first")?;
    fixture.wait_until_artifact_packaging("atlas-first")?;

    let output = fixture.package_command("atlas", "atlas-second", false)?.output()?;

    anyhow::ensure!(!output.status.success(), "same-site package unexpectedly succeeded");
    anyhow::ensure!(
        String::from_utf8_lossy(&output.stderr).contains("A local build is already running for atlas"),
        "same-site package did not report lock contention:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );

    fixture.release_artifact_packaging("atlas-first")?;
    assert_package_succeeded(first.wait()?)
}

#[test]
fn packages_for_different_sites_build_concurrently() -> Result<()> {
    let fixture = PackageFixture::new()?;
    let mut first = fixture.start_package("atlas", "atlas", false)?;
    let mut second = fixture.start_package("beacon", "beacon", false)?;

    fixture.wait_until_ready("atlas")?;
    fixture.wait_until_ready("beacon")?;
    fixture.release("atlas")?;
    fixture.release("beacon")?;

    assert_package_succeeded(first.wait()?)?;
    assert_package_succeeded(second.wait()?)
}

#[test]
fn local_build_lock_helper() -> Result<()> {
    let Some(site) = env::var_os("BONESDEPLOY_LOCAL_BUILD_LOCK_SITE") else {
        return Ok(());
    };
    let site = site.into_string().map_err(|_| anyhow::anyhow!("test site name was not UTF-8"))?;
    let mut config = Bones::for_site(&site);
    config.branch = "main".into();
    let artifact = build::package(&config)?;
    anyhow::ensure!(artifact.path().is_file(), "package helper did not create an artifact");
    Ok(())
}

struct PackageFixture {
    temp: tempfile::TempDir,
    project: PathBuf,
    tools: PathBuf,
    cache: PathBuf,
    artifact_open_barrier: PathBuf,
}

impl PackageFixture {
    fn new() -> Result<Self> {
        let temp = tempfile::tempdir()?;
        let project = temp.path().join("project");
        let tools = temp.path().join("tools");
        let cache = temp.path().join("cache");
        fs::create_dir_all(&project)?;
        fs::create_dir_all(&tools)?;
        initialize_project(&project)?;
        write_fake_docker(&tools)?;
        let artifact_open_barrier = compile_artifact_open_barrier(&tools)?;
        Ok(Self { temp, project, tools, cache, artifact_open_barrier })
    }

    fn start_package(&self, site: &str, name: &str, hold_artifact_packaging: bool) -> Result<Child> {
        self.package_command(site, name, hold_artifact_packaging)?.spawn().context("start package helper")
    }

    fn package_command(&self, site: &str, name: &str, hold_artifact_packaging: bool) -> Result<Command> {
        let ready = self.temp.path().join(format!("{name}.ready"));
        let release = self.temp.path().join(format!("{name}.release"));
        let artifact_target = self.temp.path().join(format!("{name}.artifact-target"));
        let artifact_ready = self.temp.path().join(format!("{name}.artifact-ready"));
        let artifact_release = self.temp.path().join(format!("{name}.artifact-release"));
        let mut command = Command::new(env::current_exe().context("locate test executable")?);
        command
            .args(["--exact", "local_build_lock_helper", "--nocapture"])
            .current_dir(&self.project)
            .env("BONESDEPLOY_LOCAL_BUILD_LOCK_SITE", site)
            .env("READY_FILE", ready)
            .env("RELEASE_FILE", release)
            .env("XDG_CACHE_HOME", &self.cache)
            .env("DOCKER_HOST", "unix:///tmp/bonesdeploy-local-build-lock.sock")
            .env("PATH", test_path(&self.tools)?);
        if hold_artifact_packaging {
            command
                .env("ARTIFACT_BARRIER_TARGET_FILE", artifact_target)
                .env("ARTIFACT_BARRIER_READY_FILE", artifact_ready)
                .env("ARTIFACT_BARRIER_RELEASE_FILE", artifact_release)
                .env("LD_PRELOAD", &self.artifact_open_barrier);
        }
        Ok(command)
    }

    fn wait_until_ready(&self, name: &str) -> Result<()> {
        let ready = self.temp.path().join(format!("{name}.ready"));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !ready.exists() {
            if Instant::now() >= deadline {
                bail!("package helper did not reach the build backend: {}", ready.display());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn release(&self, name: &str) -> Result<()> {
        fs::write(self.temp.path().join(format!("{name}.release")), "release")?;
        Ok(())
    }

    fn wait_until_artifact_packaging(&self, name: &str) -> Result<()> {
        let completed = self.temp.path().join(format!("{name}.artifact-ready"));
        let deadline = Instant::now() + Duration::from_secs(10);
        while !completed.exists() {
            if Instant::now() >= deadline {
                bail!("package helper did not enter artifact packaging: {}", completed.display());
            }
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    }

    fn release_artifact_packaging(&self, name: &str) -> Result<()> {
        fs::write(self.temp.path().join(format!("{name}.artifact-release")), "release")?;
        Ok(())
    }
}

fn assert_package_succeeded(status: ExitStatus) -> Result<()> {
    if status.success() { Ok(()) } else { bail!("package helper failed with status {status}") }
}

fn test_path(tools: &Path) -> Result<OsString> {
    let path = env::var_os("PATH").unwrap_or_else(OsString::new);
    env::join_paths(iter::once(tools.to_path_buf()).chain(env::split_paths(&path))).context("construct test PATH")
}

fn initialize_project(project: &Path) -> Result<()> {
    run_git(project, ["init", "--quiet", "--initial-branch=main"])?;
    let build = project.join("infra/deployment/build");
    fs::create_dir_all(&build)?;
    fs::write(build.join("01_wait.sh"), "true\n")?;
    fs::write(project.join("artifact-barrier-input"), "barrier")?;
    run_git(project, ["add", "."])?;
    run_git(project, ["-c", "user.name=BonesDeploy", "-c", "user.email=bonesdeploy@local", "commit", "-m", "initial"])
}

fn write_fake_docker(tools: &Path) -> Result<()> {
    let docker = tools.join("docker");
    fs::write(
        &docker,
        r#"#!/bin/sh
if [ "$1" = info ]; then
	printf 'linux\n'
	exit 0
fi
if [ "$1" = image ] && [ "$2" = inspect ]; then
	exit 0
fi
for argument in "$@"; do
	case "$argument" in
		*:/workspace/source) [ -z "${ARTIFACT_BARRIER_TARGET_FILE:-}" ] || printf '%s/artifact-barrier-input\n' "${argument%:/workspace/source}" >"$ARTIFACT_BARRIER_TARGET_FILE" ;;
	esac
	if [ "$argument" = -i ]; then
		cat >/dev/null
		: >"$READY_FILE"
		while [ ! -e "$RELEASE_FILE" ]; do sleep 0.01; done
		break
	fi
done
exit 0
"#,
    )?;
    fs::set_permissions(docker, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

fn compile_artifact_open_barrier(tools: &Path) -> Result<PathBuf> {
    let source = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/artifact_open_barrier.c");
    let output = tools.join("artifact-open-barrier.so");
    let status = Command::new("cc")
        .args(["-shared", "-fPIC"])
        .arg(&source)
        .args(["-o"])
        .arg(&output)
        .status()
        .context("compile artifact open barrier")?;
    anyhow::ensure!(status.success(), "artifact open barrier compiler failed with status {status}");
    Ok(output)
}

fn run_git<const N: usize>(project: &Path, arguments: [&str; N]) -> Result<()> {
    let status = Command::new("git").arg("-C").arg(project).args(arguments).status().context("run git")?;
    anyhow::ensure!(status.success(), "git command failed");
    Ok(())
}
