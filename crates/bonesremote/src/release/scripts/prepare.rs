use std::fs;
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};

use anyhow::{Context, Result};

use super::output;

pub(crate) struct PrepareScriptEnv<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) project_root: &'a str,
    pub(crate) runtime_user: &'a str,
    pub(crate) web_root: &'a str,
}

pub(crate) fn run_prepare_script(
    script: &Path,
    release_root: &Path,
    log_path: &Path,
    env: &PrepareScriptEnv<'_>,
) -> Result<ExitStatus> {
    let script_file =
        fs::File::open(script).with_context(|| format!("Failed to open prepare script {}", script.display()))?;

    let mut command = Command::new("runuser");
    configure_prepare_command(&mut command, release_root, env);

    let mut child =
        command.stdin(Stdio::from(script_file)).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().with_context(
            || format!("Failed to execute prepare script {} as {}", script.display(), env.runtime_user),
        )?;

    output::stream_child_output(&mut child, log_path, &format!("prepare script {}", script.display()))
}

fn configure_prepare_command(command: &mut Command, release_root: &Path, env: &PrepareScriptEnv<'_>) {
    command
        .args(["-u", env.runtime_user, "--", "bash", "-c", "umask 0002; exec bash -s"])
        .current_dir(release_root)
        .env("PROJECT_NAME", env.project_name)
        .env("PROJECT_ROOT", env.project_root)
        .env("REPO_PATH", "")
        .env("WEB_ROOT", env.web_root)
        .env("SERVICE_USER", env.runtime_user);
}

#[cfg(test)]
#[test]
fn prepare_command_runs_as_runtime_user_in_release() {
    let mut command = Command::new("runuser");
    configure_prepare_command(
        &mut command,
        Path::new("/srv/sites/demo/releases/20260626_120000"),
        &PrepareScriptEnv {
            project_name: "demo",
            project_root: "/srv/sites/demo",
            runtime_user: "demo",
            web_root: "public",
        },
    );

    let args = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();
    assert_eq!(args[0], "-u");
    assert_eq!(args[1], "demo");
    assert_eq!(command.get_current_dir(), Some(Path::new("/srv/sites/demo/releases/20260626_120000")));
    assert!(!args.iter().any(|arg| arg.contains("/root/.config/bonesremote")));
    let service_user = command.get_envs().find_map(|(key, value)| {
        (key.to_string_lossy() == "SERVICE_USER")
            .then(|| value.map(|value| value.to_string_lossy().into_owned()))
            .flatten()
    });
    assert_eq!(service_user, Some(String::from("demo")));
    assert!(!args.iter().any(|arg| arg.contains("podman")));
}
