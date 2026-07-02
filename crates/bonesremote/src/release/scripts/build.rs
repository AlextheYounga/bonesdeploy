use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};
use shared::paths;

use super::output;

pub(crate) struct BuildScriptEnv<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) build_user: &'a str,
    pub(crate) build_uid: u32,
    pub(crate) web_root: &'a str,
    pub(crate) build_image: &'a str,
}

pub(crate) fn run_podman_build_script(
    script: &Path,
    source_root: &Path,
    log_path: &Path,
    env: &BuildScriptEnv<'_>,
) -> Result<ExitStatus> {
    let script_file =
        fs::File::open(script).with_context(|| format!("Failed to open build script {}", script.display()))?;

    let mut command = Command::new("runuser");
    configure_podman_build_command(&mut command, source_root, env);

    let mut child = command
        .stdin(Stdio::from(script_file))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to execute build script {} in podman", script.display()))?;

    output::stream_child_output(&mut child, log_path, &format!("podman build script {}", script.display()))
}

fn configure_podman_build_command(command: &mut Command, source_root: &Path, env: &BuildScriptEnv<'_>) {
    let mount = format!("{}:/workspace/source", source_root.display());
    command
        .args(["-u", env.build_user, "--", "env"])
        .arg(format!("HOME={}", paths::bonesdeploy_user_home(env.build_user).display()))
        .arg(format!("XDG_RUNTIME_DIR=/run/user/{}", env.build_uid))
        .current_dir(source_root)
        .args([
            "podman",
            "run",
            "--rm",
            "-i",
            "--pull=missing",
            "--security-opt=no-new-privileges",
            "--cap-drop=all",
            "--workdir=/workspace/source",
            "--volume",
        ])
        .arg(mount)
        .arg("--env")
        .arg(format!("PROJECT_NAME={}", env.project_name))
        .arg("--env")
        .arg("PROJECT_ROOT=/workspace")
        .arg("--env")
        .arg("REPO_PATH=")
        .arg("--env")
        .arg(format!("WEB_ROOT={}", env.web_root))
        .arg("--env")
        .arg(format!("SERVICE_USER={}", env.project_name))
        .arg(env.build_image)
        .args(["bash", "-c", "umask 0002; exec bash -s"]);
}

#[cfg(test)]
#[test]
fn podman_build_command_mounts_only_source_tree() {
    let mut command = Command::new("runuser");
    configure_podman_build_command(
        &mut command,
        Path::new("/tmp/source"),
        &BuildScriptEnv {
            project_name: "demo",
            build_user: "demo-build",
            build_uid: 1234,
            web_root: "public",
            build_image: "docker.io/library/node:22-bookworm",
        },
    );

    let args = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();
    assert_eq!(args[0], "-u");
    assert_eq!(args[1], "demo-build");
    assert_eq!(args[2], "--");
    assert_eq!(args[3], "env");
    assert!(args.contains(&String::from("HOME=/var/lib/bonesdeploy/users/demo-build")));
    assert!(args.contains(&String::from("XDG_RUNTIME_DIR=/run/user/1234")));
    assert!(args.contains(&String::from("podman")));
    assert!(args.contains(&String::from("--rm")));
    assert!(args.contains(&String::from("-i")));
    assert!(args.contains(&String::from("--security-opt=no-new-privileges")));
    assert!(args.contains(&String::from("--cap-drop=all")));
    assert!(args.contains(&String::from("/tmp/source:/workspace/source")));
    assert!(!args.iter().any(|arg| arg.contains("/srv/sites/demo/shared")));
    assert!(!args.iter().any(|arg| arg.contains("/root/.config/bonesremote")));
    assert_eq!(command.get_current_dir(), Some(Path::new("/tmp/source")));
}
