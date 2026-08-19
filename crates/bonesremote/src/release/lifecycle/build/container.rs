use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command, ExitStatus, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::environment;

use super::build_user::{BuildScriptEnv, build_script_command, build_user_command, build_user_control_command};
use super::ownership;
use crate::release::output;

pub const BUILD_IMAGE: &str = "docker.io/library/buildpack-deps:bookworm";

pub fn service_command(build_user: &str, container_name: &str) -> Command {
    let mut command = Command::new("systemd-run");
    // Conmon reports readiness while Podman remains responsible for stopping the container.
    command
        .arg(format!("--machine={build_user}@"))
        .args(["--quiet", "--user", "--collect", "--unit"])
        .arg(container_name)
        .args(["--service-type=notify", "--property=NotifyAccess=all", "--property=KillMode=none"]);
    command
}

pub struct BuildContainer<'a> {
    env: &'a BuildScriptEnv<'a>,
    source_root: &'a Path,
    name: String,
    build_env_file: Option<PathBuf>,
    removed: bool,
}

impl<'a> BuildContainer<'a> {
    pub fn start(source_root: &'a Path, env: &'a BuildScriptEnv<'a>) -> Result<Self> {
        let name = container_name(env.project_name);
        remove_existing(source_root, env, &name)?;
        ensure_image(source_root, env, &name)?;

        let build_env_file = write_build_env_file(source_root, env)?;
        if let Err(error) = ownership::chown_tree_to_user(&build_env_file, env.build_user, env.build_group) {
            fs::remove_file(&build_env_file).ok();
            return Err(error);
        }
        let mut command = service_command(env.build_user, &name);
        configure_create(
            &mut command,
            &ContainerCreate { source_root, env, container_name: &name, build_env_file: &build_env_file },
        );
        let status = match command.status().with_context(|| format!("Failed to start build container {name}")) {
            Ok(status) => status,
            Err(error) => {
                fs::remove_file(&build_env_file).ok();
                return Err(error);
            }
        };
        if !status.success() {
            fs::remove_file(&build_env_file).ok();
            bail!("Failed to start build container {name}: {status}");
        }

        let container = Self { env, source_root, name, build_env_file: Some(build_env_file), removed: false };
        container.copy_deployment_tree()?;
        Ok(container)
    }

    pub fn run_script(&self, script: &Path, log_path: &Path) -> Result<ExitStatus> {
        let script_file =
            fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;
        let description = format!("podman build script {}", script.display());
        let mut command = match self.env.script_timeout_seconds {
            Some(timeout) => build_script_command(self.env.build_user, timeout),
            None => build_user_command(self.env.build_user),
        };
        configure_exec(&mut command, self.source_root, &self.name);
        let mut child = command
            .stdin(Stdio::from(script_file))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute {description} in podman"))?;
        output::stream_child_output(&mut child, log_path, &description)
    }

    pub fn remove(&mut self) -> Result<()> {
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
        self.remove_build_env_file();
        Ok(())
    }

    pub fn remove_build_env_file(&mut self) {
        if let Some(path) = self.build_env_file.take() {
            fs::remove_file(path).ok();
        }
    }

    pub fn copy_deployment_tree(&self) -> Result<()> {
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
        self.remove_build_env_file();
        self.removed = true;
    }
}

pub struct ContainerCreate<'a> {
    source_root: &'a Path,
    env: &'a BuildScriptEnv<'a>,
    container_name: &'a str,
    build_env_file: &'a Path,
}

pub fn build_container_command(
    source_root: &Path,
    env: &BuildScriptEnv<'_>,
    container_name: &str,
    build_env_file: &Path,
) -> Command {
    let mut command = service_command(env.build_user, container_name);
    configure_create(&mut command, &ContainerCreate { source_root, env, container_name, build_env_file });
    command
}

pub fn configure_create(command: &mut Command, create: &ContainerCreate<'_>) {
    let source_mount = format!("{}:/workspace/source", create.source_root.display());
    let cache_mount = format!("{}:/workspace/cache:rw", create.env.build_cache_dir.display());
    command
        .current_dir(create.source_root)
        .args(["podman", "run", "-d", "--pull=never"])
        .args([
            "--sdnotify=conmon",
            "--cgroups=no-conmon",
            "--security-opt=no-new-privileges",
            "--workdir=/workspace/source",
            "--name",
        ])
        .arg(create.container_name)
        .args([
            "--env",
            &format!("{}={}", environment::PROJECT_NAME, create.env.project_name),
            "--env",
            &format!("{}=/workspace", environment::PROJECT_ROOT),
            "--env",
            &format!("{}=", environment::REPO_PATH),
        ])
        .args([
            "--env",
            &format!("{}={}", environment::WEB_ROOT, create.env.web_root),
            "--env",
            &format!("{}={}", environment::SERVICE_USER, create.env.project_name),
        ]);

    command
        .args(["--env-file"])
        .arg(create.build_env_file)
        .args(["--env", &format!("{}=/workspace/cache", environment::BUILD_CACHE_DIR), "--volume"])
        .arg(source_mount)
        .args(["--volume"])
        .arg(cache_mount)
        .arg(BUILD_IMAGE)
        .args(["sleep", "infinity"]);
}

pub fn write_build_env_file(source_root: &Path, env: &BuildScriptEnv<'_>) -> Result<PathBuf> {
    let path = source_root.join(format!(".env.build.{}", process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(&path)
        .with_context(|| format!("Failed to create protected build environment file {}", path.display()))?;

    for (key, value) in env.build_env_vars {
        if let Err(error) = writeln!(file, "{key}={value}")
            .with_context(|| format!("Failed to write protected build environment file {}", path.display()))
        {
            fs::remove_file(&path).ok();
            return Err(error);
        }
    }
    Ok(path)
}

pub fn configure_exec(command: &mut Command, source_root: &Path, container_name: &str) {
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

pub fn configure_deployment_extract_command(command: &mut Command, source_root: &Path, container_name: &str) {
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

pub fn configure_remove(command: &mut Command, source_root: &Path, container_name: &str) {
    command.current_dir(source_root).args(["podman", "rm", "--force", "--time", "0", "--ignore", container_name]);
}

pub fn container_name(project_name: &str) -> String {
    format!("bonesdeploy-build-{project_name}")
}

pub fn remove_build_container(build_user: &str, project_name: &str, working_dir: &Path) -> Result<()> {
    let name = container_name(project_name);
    let mut remove = build_user_control_command(build_user);
    configure_remove(&mut remove, working_dir, &name);
    let status = remove.status().with_context(|| format!("Failed to remove build container {name}"))?;
    if !status.success() {
        bail!("Failed to remove build container {name}: {status}");
    }
    Ok(())
}

pub fn remove_existing(source_root: &Path, env: &BuildScriptEnv<'_>, container_name: &str) -> Result<()> {
    let mut remove = build_user_control_command(env.build_user);
    configure_remove(&mut remove, source_root, container_name);
    let status =
        remove.status().with_context(|| format!("Failed to remove existing build container {container_name}"))?;
    if !status.success() {
        bail!("Failed to remove existing build container {container_name}: {status}");
    }
    Ok(())
}

pub fn ensure_image(source_root: &Path, env: &BuildScriptEnv<'_>, container_name: &str) -> Result<()> {
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
