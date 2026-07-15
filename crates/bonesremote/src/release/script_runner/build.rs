use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result, bail};
use shared::paths;

use super::output;

const BUILD_IMAGE: &str = "docker.io/library/buildpack-deps:bookworm";

pub(crate) struct BuildScriptEnv<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) build_user: &'a str,
    pub(crate) build_uid: u32,
    pub(crate) web_root: &'a str,
}

pub(crate) struct BuildContainer<'a> {
    env: &'a BuildScriptEnv<'a>,
    source_root: &'a Path,
    name: String,
    removed: bool,
}

impl<'a> BuildContainer<'a> {
    pub(crate) fn start(source_root: &'a Path, env: &'a BuildScriptEnv<'a>) -> Result<Self> {
        let mut command = Command::new("runuser");
        let name = build_container_name(env.project_name);
        remove_existing_container(source_root, env, &name)?;
        configure_podman_create_command(&mut command, source_root, env, &name);

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

        let mut command = Command::new("runuser");
        configure_podman_exec_command(&mut command, self.source_root, self.env, &self.name);
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

        let mut command = Command::new("runuser");
        configure_podman_remove_command(&mut command, self.source_root, self.env, &self.name);
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

        let mut command = Command::new("runuser");
        configure_podman_remove_command(&mut command, self.source_root, self.env, &self.name);
        let _ = command.status();
        self.removed = true;
    }
}

fn configure_podman_create_command(
    command: &mut Command,
    source_root: &Path,
    env: &BuildScriptEnv<'_>,
    container_name: &str,
) {
    let mount = format!("{}:/workspace/source", source_root.display());
    command
        .args(["-u", env.build_user, "--", "env"])
        .arg(format!("HOME={}", paths::bonesdeploy_user_home(env.build_user).display()))
        .arg(format!("XDG_RUNTIME_DIR=/run/user/{}", env.build_uid))
        .current_dir(source_root)
        .args([
            "podman",
            "run",
            "-d",
            "--pull=missing",
            "--security-opt=no-new-privileges",
            "--workdir=/workspace/source",
            "--name",
        ])
        .arg(container_name)
        .args(["--env"])
        .arg(format!("PROJECT_NAME={}", env.project_name))
        .arg("--env")
        .arg("PROJECT_ROOT=/workspace")
        .arg("--env")
        .arg("REPO_PATH=")
        .arg("--env")
        .arg(format!("WEB_ROOT={}", env.web_root))
        .arg("--env")
        .arg(format!("SERVICE_USER={}", env.project_name))
        .args(["--volume"])
        .arg(mount)
        .arg(BUILD_IMAGE)
        .args(["sleep", "infinity"]);
}

fn configure_podman_exec_command(
    command: &mut Command,
    source_root: &Path,
    env: &BuildScriptEnv<'_>,
    container_name: &str,
) {
    command
        .args(["-u", env.build_user, "--", "env"])
        .arg(format!("HOME={}", paths::bonesdeploy_user_home(env.build_user).display()))
        .arg(format!("XDG_RUNTIME_DIR=/run/user/{}", env.build_uid))
        .current_dir(source_root)
        .args(["podman", "exec", "-i", container_name, "bash", "-c", "umask 0002; exec bash -s"]);
}

fn configure_podman_remove_command(
    command: &mut Command,
    source_root: &Path,
    env: &BuildScriptEnv<'_>,
    container_name: &str,
) {
    command
        .args(["-u", env.build_user, "--", "env"])
        .arg(format!("HOME={}", paths::bonesdeploy_user_home(env.build_user).display()))
        .arg(format!("XDG_RUNTIME_DIR=/run/user/{}", env.build_uid))
        .current_dir(source_root)
        .args(["podman", "rm", "--force", "--ignore", container_name]);
}

fn build_container_name(project_name: &str) -> String {
    format!("bonesdeploy-build-{project_name}")
}

fn remove_existing_container(source_root: &Path, env: &BuildScriptEnv<'_>, container_name: &str) -> Result<()> {
    let mut remove = Command::new("runuser");
    configure_podman_remove_command(&mut remove, source_root, env, container_name);
    let remove_status =
        remove.status().with_context(|| format!("Failed to remove existing build container {container_name}"))?;
    if !remove_status.success() {
        bail!("Failed to remove existing build container {container_name}: {remove_status}");
    }

    Ok(())
}

#[cfg(test)]
#[test]
fn podman_build_command_mounts_only_source_tree() {
    let mut command = Command::new("runuser");
    configure_podman_create_command(
        &mut command,
        Path::new("/tmp/source"),
        &BuildScriptEnv { project_name: "demo", build_user: "demo-build", build_uid: 1234, web_root: "public" },
        "demo-container",
    );

    let args = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();
    assert_build_command_identity(&args);
    assert_build_command_mounts(&args, &command);
}

#[cfg(test)]
fn assert_build_command_identity(args: &[String]) {
    assert_eq!(args[0], "-u");
    assert_eq!(args[1], "demo-build");
    assert_eq!(args[2], "--");
    assert_eq!(args[3], "env");
    assert!(args.contains(&String::from("HOME=/var/lib/bonesdeploy/users/demo-build")));
    assert!(args.contains(&String::from("XDG_RUNTIME_DIR=/run/user/1234")));
}

#[cfg(test)]
fn assert_build_command_mounts(args: &[String], command: &Command) {
    assert!(args.contains(&String::from("podman")));
    assert!(args.contains(&String::from("run")));
    assert!(args.contains(&String::from("-d")));
    assert!(args.contains(&String::from("--security-opt=no-new-privileges")));
    assert!(!args.iter().any(|arg| arg == "--cap-drop=all"));
    assert!(args.contains(&String::from("docker.io/library/buildpack-deps:bookworm")));
    assert!(args.contains(&String::from("/tmp/source:/workspace/source")));
    assert!(!args.iter().any(|arg| arg.contains("/srv/sites/demo/shared")));
    assert!(!args.iter().any(|arg| arg.contains("/root/.config/bonesremote")));
    assert_eq!(command.get_current_dir(), Some(Path::new("/tmp/source")));
}

#[cfg(test)]
#[test]
fn podman_exec_and_remove_commands_use_source_working_directory() {
    let env = BuildScriptEnv { project_name: "demo", build_user: "demo-build", build_uid: 1234, web_root: "public" };

    let mut exec = Command::new("runuser");
    configure_podman_exec_command(&mut exec, Path::new("/tmp/source"), &env, "demo-container");
    assert_eq!(exec.get_current_dir(), Some(Path::new("/tmp/source")));

    let mut remove = Command::new("runuser");
    configure_podman_remove_command(&mut remove, Path::new("/tmp/source"), &env, "demo-container");
    let args = remove.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();
    assert_eq!(remove.get_current_dir(), Some(Path::new("/tmp/source")));
    assert!(args.windows(3).any(|window| window == ["podman", "rm", "--force"]));
    assert!(args.contains(&String::from("--ignore")));
}

#[cfg(test)]
#[test]
fn build_container_name_is_deterministic_per_project() {
    assert_eq!(build_container_name("demo"), "bonesdeploy-build-demo");
}
