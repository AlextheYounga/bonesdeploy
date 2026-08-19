use std::fs::{self, OpenOptions};
use std::io::Write;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::process::{self, Command, ExitStatus, Stdio};

use anyhow::{Context, Result, bail};

use super::build_user::{BuildScriptEnv, build_script_command, build_user_command, build_user_control_command};
use super::ownership;
use crate::release::output;

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

pub(super) struct BuildContainer<'a> {
    env: &'a BuildScriptEnv<'a>,
    source_root: &'a Path,
    name: String,
    build_env_file: Option<PathBuf>,
    removed: bool,
}

impl<'a> BuildContainer<'a> {
    pub(super) fn start(source_root: &'a Path, env: &'a BuildScriptEnv<'a>) -> Result<Self> {
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

    pub(super) fn run_script(&self, script: &Path, log_path: &Path) -> Result<ExitStatus> {
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

    pub(super) fn remove(&mut self) -> Result<()> {
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

    fn remove_build_env_file(&mut self) {
        if let Some(path) = self.build_env_file.take() {
            fs::remove_file(path).ok();
        }
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
        self.remove_build_env_file();
        self.removed = true;
    }
}

struct ContainerCreate<'a> {
    source_root: &'a Path,
    env: &'a BuildScriptEnv<'a>,
    container_name: &'a str,
    build_env_file: &'a Path,
}

fn configure_create(command: &mut Command, create: &ContainerCreate<'_>) {
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
            &format!("PROJECT_NAME={}", create.env.project_name),
            "--env",
            "PROJECT_ROOT=/workspace",
            "--env",
            "REPO_PATH=",
        ])
        .args([
            "--env",
            &format!("WEB_ROOT={}", create.env.web_root),
            "--env",
            &format!("SERVICE_USER={}", create.env.project_name),
        ]);

    command
        .args(["--env-file"])
        .arg(create.build_env_file)
        .args(["--env", "BUILD_CACHE_DIR=/workspace/cache", "--volume"])
        .arg(source_mount)
        .args(["--volume"])
        .arg(cache_mount)
        .arg(BUILD_IMAGE)
        .args(["sleep", "infinity"]);
}

fn write_build_env_file(source_root: &Path, env: &BuildScriptEnv<'_>) -> Result<PathBuf> {
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

#[cfg(test)]
mod tests {
    use std::{env, fs, os::unix::fs::PermissionsExt, process};

    use anyhow::Result;

    use super::*;

    #[test]
    fn build_env_values_use_a_private_env_file_instead_of_command_arguments() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-build-env-file-{}", process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root)?;
        let variables = vec![("PUBLIC_API_URL".to_string(), "https://example.test/private-value".to_string())];
        let env = BuildScriptEnv {
            project_name: "demo",
            build_user: "demo-build",
            build_group: "demo-build",
            web_root: ".output/public",
            deployment_dir: &root,
            build_cache_dir: &root,
            build_env_vars: &variables,
            script_timeout_seconds: None,
        };

        let env_file = write_build_env_file(&root, &env)?;
        let mut command = service_command(env.build_user, "bonesdeploy-build-demo");
        let create = ContainerCreate {
            source_root: &root,
            env: &env,
            container_name: "bonesdeploy-build-demo",
            build_env_file: &env_file,
        };
        configure_create(&mut command, &create);
        let arguments: Vec<_> = command.get_args().map(|argument| argument.to_string_lossy().into_owned()).collect();

        let env_file_argument = env_file.to_string_lossy();
        assert!(arguments.windows(2).any(|pair| pair[0] == "--env-file" && pair[1] == env_file_argument.as_ref()));
        assert!(!arguments.iter().any(|argument| argument.contains("private-value")));
        assert_eq!(fs::metadata(&env_file)?.permissions().mode() & 0o777, 0o600);
        assert_eq!(fs::read_to_string(&env_file)?, "PUBLIC_API_URL=https://example.test/private-value\n");

        fs::remove_file(env_file).ok();
        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
