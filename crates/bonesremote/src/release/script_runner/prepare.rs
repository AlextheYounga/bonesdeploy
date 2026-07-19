use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::thread;

use anyhow::{Context, Result};

use super::output;

pub(crate) struct PrepareScriptEnv<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) project_root: &'a str,
    pub(crate) runtime_user: &'a str,
    pub(crate) web_root: &'a str,
    pub(crate) shared_functions: &'a Path,
}

pub(crate) fn run_prepare_script(
    script: &Path,
    release_root: &Path,
    log_path: &Path,
    env: &PrepareScriptEnv<'_>,
) -> Result<ExitStatus> {
    let functions_file = fs::File::open(env.shared_functions)
        .with_context(|| format!("Failed to open shared prepare functions {}", env.shared_functions.display()))?;
    let script_file =
        fs::File::open(script).with_context(|| format!("Failed to open prepare script {}", script.display()))?;

    let mut command = Command::new("runuser");
    configure_prepare_command(&mut command, release_root, env);

    let mut child =
        command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().with_context(|| {
            format!("Failed to execute prepare script {} as {}", script.display(), env.runtime_user)
        })?;
    let stdin = child.stdin.take().context("Failed to capture prepare script stdin")?;
    let input_handle = thread::spawn(move || stream_prepare_input(stdin, functions_file, script_file));

    let status = output::stream_child_output(&mut child, log_path, &format!("prepare script {}", script.display()));
    join_prepare_input(input_handle, env.shared_functions, script)?;
    status
}

fn stream_prepare_input<W: Write>(
    mut stdin: W,
    mut functions_file: fs::File,
    mut script_file: fs::File,
) -> io::Result<()> {
    io::copy(&mut functions_file, &mut stdin)?;
    stdin.write_all(b"\n")?;
    io::copy(&mut script_file, &mut stdin)?;
    stdin.flush()
}

fn join_prepare_input(
    handle: thread::JoinHandle<io::Result<()>>,
    shared_functions: &Path,
    script: &Path,
) -> Result<()> {
    match handle.join() {
        Ok(Ok(())) => Ok(()),
        Ok(Err(error)) if error.kind() == io::ErrorKind::BrokenPipe => Ok(()),
        Ok(Err(error)) => Err(error).with_context(|| {
            format!(
                "Failed to stream shared prepare functions {} and prepare script {}",
                shared_functions.display(),
                script.display()
            )
        }),
        Err(_) => anyhow::bail!("Prepare script input thread panicked"),
    }
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
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::{Command, ExitStatus, Stdio};
    use std::thread;

    use anyhow::{Context, Result};

    use super::{join_prepare_input, stream_prepare_input};
    use crate::release::script_runner::output;

    fn run_composed_bash(functions: &Path, script: &Path, log_path: &Path) -> Result<ExitStatus> {
        let functions_file = fs::File::open(functions)?;
        let script_file = fs::File::open(script)?;
        let mut child = Command::new("bash")
            .arg("-s")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let stdin = child.stdin.take().context("Failed to capture test Bash stdin")?;
        let input_handle = thread::spawn(move || stream_prepare_input(stdin, functions_file, script_file));
        let status = output::stream_child_output(&mut child, log_path, "composed prepare test")?;
        join_prepare_input(input_handle, functions, script)?;
        Ok(status)
    }

    #[test]
    fn shared_functions_precede_prepare_script() -> Result<()> {
        let root = super::super::temp_dir("bonesremote-prepare-prelude")?;
        let functions = root.join("functions.sh");
        let script = root.join("01_prepare.sh");
        let log = root.join("prepare.log");
        fs::write(&functions, "log() { printf 'helper: %s\\n' \"$*\"; }")?;
        fs::write(&script, "log ready")?;

        let status = run_composed_bash(&functions, &script, &log)?;

        assert!(status.success());
        assert_eq!(fs::read_to_string(log)?, "helper: ready\n");
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn failing_prepare_preserves_status_and_output() -> Result<()> {
        let root = super::super::temp_dir("bonesremote-prepare-failure")?;
        let functions = root.join("functions.sh");
        let script = root.join("01_prepare.sh");
        let log = root.join("prepare.log");
        fs::write(&functions, "")?;
        fs::write(&script, "echo prepare-failed >&2\nexit 7")?;

        let status = run_composed_bash(&functions, &script, &log)?;

        assert_eq!(status.code(), Some(7));
        assert!(fs::read_to_string(log)?.contains("prepare-failed"));
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn early_successful_exit_ignores_broken_pipe() -> Result<()> {
        let root = super::super::temp_dir("bonesremote-prepare-early-exit")?;
        let functions = root.join("functions.sh");
        let script = root.join("01_prepare.sh");
        let log = root.join("prepare.log");
        fs::write(&functions, "exit 0\n")?;
        fs::write(&script, "# padding\n".repeat(1_000_000))?;

        let status = run_composed_bash(&functions, &script, &log)?;

        assert!(status.success());
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn runtime_prepare_templates_do_not_source_control_plane_files() -> Result<()> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../bonesdeploy/runtimes");
        for runtime in ["laravel", "rails", "django"] {
            let template = fs::read_to_string(
                root.join(runtime).join("deployment/prepare").join(format!("01_prepare_{runtime}.sh")),
            )?;
            assert!(!template.contains("DEPLOYMENT_DIR"), "{runtime} prepare template still references DEPLOYMENT_DIR");
            assert!(!template.contains("functions.sh"), "{runtime} prepare template still sources functions.sh");
        }
        Ok(())
    }
}
