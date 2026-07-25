use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result, bail};

use super::{BuildScriptEnv, build_user_command, build_user_control_command};
use crate::release::script_runner::output;

const BUILD_IMAGE: &str = "docker.io/library/buildpack-deps:bookworm";

fn service_command(build_user: &str, container_name: &str) -> Command {
    let mut command = Command::new("systemd-run");
    // Conmon reports readiness while Podman remains responsible for stopping the container.
    command
        .arg(format!("--machine={build_user}@"))
        .args(["--quiet", "--user", "--collect", "--unit"])
        .arg(container_name)
        .args(["--service-type=notify", "--property=NotifyAccess=all", "--property=KillMode=none"]);
    command
}

pub(crate) struct BuildContainer<'a> {
    env: &'a BuildScriptEnv<'a>,
    source_root: &'a Path,
    name: String,
    removed: bool,
}

impl<'a> BuildContainer<'a> {
    pub(crate) fn start(source_root: &'a Path, env: &'a BuildScriptEnv<'a>) -> Result<Self> {
        let name = container_name(env.project_name);
        remove_existing(source_root, env, &name)?;
        ensure_image(source_root, env, &name)?;

        let mut command = service_command(env.build_user, &name);
        configure_create(&mut command, source_root, env, &name);
        let status = command.status().with_context(|| format!("Failed to start build container {name}"))?;
        if !status.success() {
            bail!("Failed to start build container {name}: {status}");
        }

        let container = Self { env, source_root, name, removed: false };
        container.copy_deployment_tree()?;
        Ok(container)
    }

    pub(crate) fn run_script(&self, script: &Path, log_path: &Path) -> Result<ExitStatus> {
        let script_file =
            fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;
        let description = format!("podman build script {}", script.display());
        let mut command = build_user_command(self.env.build_user);
        configure_exec(&mut command, self.source_root, &self.name);
        let mut child = command
            .stdin(Stdio::from(script_file))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute {description} in podman"))?;
        output::stream_child_output(&mut child, log_path, &description)
    }

    pub(crate) fn remove(&mut self) -> Result<()> {
        if self.removed {
            return Ok(());
        }
        let mut command = build_user_control_command(self.env.build_user);
        configure_remove(&mut command, self.source_root, &self.name);
        let status = command.status().with_context(|| format!("Failed to remove build container {}", self.name))?;
        if !status.success() {
            bail!("Failed to remove build container {}: {}", self.name, status);
        }
        self.removed = true;
        Ok(())
    }

    fn copy_deployment_tree(&self) -> Result<()> {
        let mut archive = Command::new("tar")
            .current_dir(self.env.deployment_dir)
            .args(["--create", "--file=-", "."])
            .stdout(Stdio::piped())
            .spawn()
            .with_context(|| {
                format!("Failed to archive deployment files from {}", self.env.deployment_dir.display())
            })?;
        let archive_stdout = archive.stdout.take().context("Deployment archive stdout was not piped")?;

        let mut extract = build_user_command(self.env.build_user);
        configure_deployment_extract_command(&mut extract, self.source_root, &self.name);
        let extract_result = extract
            .stdin(Stdio::from(archive_stdout))
            .status()
            .with_context(|| format!("Failed to copy deployment files into build container {}", self.name));
        let archive_status = archive.wait().context("Failed to finish deployment archive")?;
        let extract_status = extract_result?;

        if !extract_status.success() {
            bail!("Failed to copy deployment files into build container {}: {extract_status}", self.name);
        }
        if !archive_status.success() {
            bail!("Failed to archive deployment files from {}: {archive_status}", self.env.deployment_dir.display());
        }
        Ok(())
    }
}

impl Drop for BuildContainer<'_> {
    fn drop(&mut self) {
        if self.removed {
            return;
        }
        let mut command = build_user_control_command(self.env.build_user);
        configure_remove(&mut command, self.source_root, &self.name);
        let _ = command.status();
        self.removed = true;
    }
}

fn configure_create(command: &mut Command, source_root: &Path, env: &BuildScriptEnv<'_>, container_name: &str) {
    let source_mount = format!("{}:/workspace/source", source_root.display());
    let cache_mount = format!("{}:/workspace/cache:rw", env.build_cache_dir.display());
    command
        .current_dir(source_root)
        .args(["podman", "run", "-d", "--pull=never"])
        .args([
            "--sdnotify=conmon",
            "--cgroups=no-conmon",
            "--security-opt=no-new-privileges",
            "--workdir=/workspace/source",
            "--name",
        ])
        .arg(container_name)
        .args([
            "--env",
            &format!("PROJECT_NAME={}", env.project_name),
            "--env",
            "PROJECT_ROOT=/workspace",
            "--env",
            "REPO_PATH=",
        ])
        .args(["--env", &format!("WEB_ROOT={}", env.web_root), "--env", &format!("SERVICE_USER={}", env.project_name)]);

    for (key, value) in env.build_env_vars {
        if key != "BUILD_CACHE_DIR" {
            command.args(["--env", &format!("{key}={value}")]);
        }
    }

    command
        .args(["--env", "BUILD_CACHE_DIR=/workspace/cache", "--volume"])
        .arg(source_mount)
        .args(["--volume"])
        .arg(cache_mount)
        .arg(BUILD_IMAGE)
        .args(["sleep", "infinity"]);
}

fn configure_exec(command: &mut Command, source_root: &Path, container_name: &str) {
    command.current_dir(source_root).args([
        "podman",
        "exec",
        "-i",
        container_name,
        "bash",
        "-c",
        "umask 0002; exec bash -s",
    ]);
}

fn configure_deployment_extract_command(command: &mut Command, source_root: &Path, container_name: &str) {
    command.current_dir(source_root).args([
        "podman",
        "exec",
        "-i",
        container_name,
        "sh",
        "-c",
        "mkdir -p /workspace/deployment && tar --extract --file=- --no-same-owner --no-same-permissions --directory=/workspace/deployment",
    ]);
}

fn configure_remove(command: &mut Command, source_root: &Path, container_name: &str) {
    command.current_dir(source_root).args(["podman", "rm", "--force", "--time", "0", "--ignore", container_name]);
}

fn container_name(project_name: &str) -> String {
    format!("bonesdeploy-build-{project_name}")
}

pub(crate) fn remove_build_container(build_user: &str, project_name: &str, working_dir: &Path) -> Result<()> {
    let name = container_name(project_name);
    let mut remove = build_user_control_command(build_user);
    configure_remove(&mut remove, working_dir, &name);
    let status = remove.status().with_context(|| format!("Failed to remove build container {name}"))?;
    if !status.success() {
        bail!("Failed to remove build container {name}: {status}");
    }
    Ok(())
}

fn remove_existing(source_root: &Path, env: &BuildScriptEnv<'_>, container_name: &str) -> Result<()> {
    let mut remove = build_user_control_command(env.build_user);
    configure_remove(&mut remove, source_root, container_name);
    let status =
        remove.status().with_context(|| format!("Failed to remove existing build container {container_name}"))?;
    if !status.success() {
        bail!("Failed to remove existing build container {container_name}: {status}");
    }
    Ok(())
}

fn ensure_image(source_root: &Path, env: &BuildScriptEnv<'_>, container_name: &str) -> Result<()> {
    let mut exists = build_user_command(env.build_user);
    exists.current_dir(source_root).args(["podman", "image", "exists", BUILD_IMAGE]);
    let status = exists.status().with_context(|| format!("Failed to inspect build image for {container_name}"))?;
    match status.code() {
        Some(0) => Ok(()),
        Some(1) => bail!(
            "Build image {BUILD_IMAGE} is unavailable to {}; reapply BonesInfra to seed the shared image store.",
            env.build_user
        ),
        _ => bail!("Failed to inspect build image for {container_name}: {status}"),
    }
}
