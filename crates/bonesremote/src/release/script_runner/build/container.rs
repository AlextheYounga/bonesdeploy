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

        Ok(Self { env, source_root, name, removed: false })
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
    let deployment_mount = format!("{}:/workspace/deployment:ro", env.deployment_dir.display());
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
        .arg(deployment_mount)
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
#[test]
fn podman_build_command_mounts_source_and_deployment_tree() {
    let mut command = service_command("demo-build", "demo-container");
    configure_create(&mut command, Path::new("/tmp/source"), &test_env(&[]), "demo-container");
    let args = arguments(&command);
    assert_eq!(command.get_program(), "systemd-run");
    assert_build_command_identity(&args);
    assert_build_command_mounts(&args, &command);
}

#[cfg(test)]
#[test]
fn podman_build_command_places_environment_before_image() {
    let vars = [(String::from("PHP_VERSION"), String::from("8.5"))];
    let mut command = service_command("demo-build", "demo-container");
    configure_create(&mut command, Path::new("/tmp/source"), &test_env(&vars), "demo-container");
    let args = arguments(&command);
    let image_index = args.iter().position(|arg| arg == BUILD_IMAGE).unwrap();
    let env_index = args.iter().position(|arg| arg == "PHP_VERSION=8.5").unwrap();
    assert!(env_index < image_index);
    assert_eq!(&args[image_index + 1..], ["sleep", "infinity"]);
}

#[cfg(test)]
fn test_env<'a>(build_env_vars: &'a [(String, String)]) -> BuildScriptEnv<'a> {
    BuildScriptEnv {
        project_name: "demo",
        build_user: "demo-build",
        web_root: "public",
        deployment_dir: Path::new("/tmp/deployment"),
        build_cache_dir: Path::new("/tmp/cache"),
        build_env_vars,
    }
}

#[cfg(test)]
fn arguments(command: &Command) -> Vec<String> {
    command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect()
}

#[cfg(test)]
fn assert_build_command_identity(args: &[String]) {
    assert_eq!(args[0], "--machine=demo-build@");
    assert_eq!(&args[1..7], ["--quiet", "--user", "--collect", "--unit", "demo-container", "--service-type=notify"]);
    assert!(args.contains(&String::from("--property=NotifyAccess=all")));
    assert!(args.contains(&String::from("--property=KillMode=none")));
    assert!(!args.iter().any(|arg| arg == "--pipe" || arg == "--wait"));
}

#[cfg(test)]
fn assert_build_command_mounts(args: &[String], command: &Command) {
    assert!(args.windows(4).any(|window| window == ["podman", "run", "-d", "--pull=never"]));
    assert!(args.contains(&String::from("--sdnotify=conmon")));
    assert!(args.contains(&String::from("--cgroups=no-conmon")));
    assert!(args.contains(&String::from("--security-opt=no-new-privileges")));
    assert!(!args.iter().any(|arg| arg == "--cap-drop=all"));
    assert!(args.contains(&String::from(BUILD_IMAGE)));
    assert!(args.contains(&String::from("/tmp/source:/workspace/source")));
    assert!(args.contains(&String::from("/tmp/deployment:/workspace/deployment:ro")));
    assert!(args.contains(&String::from("/tmp/cache:/workspace/cache:rw")));
    assert!(args.contains(&String::from("BUILD_CACHE_DIR=/workspace/cache")));
    assert_eq!(command.get_current_dir(), Some(Path::new("/tmp/source")));
}

#[cfg(test)]
#[test]
fn podman_exec_and_remove_commands_use_source_working_directory() {
    let mut exec = build_user_command("demo-build");
    configure_exec(&mut exec, Path::new("/tmp/source"), "demo-container");
    assert_eq!(exec.get_current_dir(), Some(Path::new("/tmp/source")));
    let mut remove = build_user_command("demo-build");
    configure_remove(&mut remove, Path::new("/tmp/source"), "demo-container");
    let args = arguments(&remove);
    assert_eq!(remove.get_current_dir(), Some(Path::new("/tmp/source")));
    assert!(args.windows(3).any(|window| window == ["podman", "rm", "--force"]));
    assert!(args.windows(3).any(|window| window == ["--time", "0", "--ignore"]));
}

#[cfg(test)]
#[test]
fn build_image_commands_use_the_foreground_user_session() {
    let mut exists = build_user_command("demo-build");
    exists.current_dir("/tmp/source").args(["podman", "image", "exists", BUILD_IMAGE]);
    let args = arguments(&exists);
    assert_eq!(&args[1..6], ["--quiet", "--user", "--collect", "--pipe", "--wait"]);
    assert!(args.windows(4).any(|window| window == ["podman", "image", "exists", BUILD_IMAGE]));
}

#[cfg(test)]
#[test]
fn build_container_name_is_deterministic_per_project() {
    assert_eq!(container_name("demo"), "bonesdeploy-build-demo");
}
