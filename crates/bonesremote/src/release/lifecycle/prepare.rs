use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::{RuntimeBackend, is_numbered_shell_script, project_env, runtime_user_for};
use bonesdeploy_core::paths;

use crate::privileges;
use crate::release::output;
use crate::release::state as release_state;
use crate::runtime::docker;

struct PrepareScriptEnv<'a> {
    project_name: &'a str,
    project_root: &'a str,
    runtime_user: &'a str,
    web_root: &'a str,
    shared_functions: &'a Path,
}

pub fn run(snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release prepare")?;

    let cfg = &snapshot.config;
    let deployment_dir = &snapshot.deployment_dir;
    let scripts_dir = deployment_dir.join(paths::DEPLOYMENT_PREPARE_DIR);
    if !scripts_dir.is_dir() {
        println!("No prepare scripts at {}; skipping prepare.", scripts_dir.display());
        return Ok(());
    }

    let scripts = list_scripts(&scripts_dir)?;
    if scripts.is_empty() {
        println!("No prepare scripts found at {}; skipping prepare.", scripts_dir.display());
        return Ok(());
    }
    let shared_functions = deployment_dir.join(paths::DEPLOYMENT_FUNCTIONS_FILE);
    if !shared_functions.is_file() {
        bail!("Shared prepare functions are missing or not a regular file: {}", shared_functions.display());
    }
    fs::File::open(&shared_functions)
        .with_context(|| format!("Shared prepare functions are unreadable: {}", shared_functions.display()))?;

    let release_name = release_state::read_staged_release(&snapshot.site)?;
    let release_dir = release_state::release_dir(&snapshot.project_root.to_string_lossy(), &release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let web_root = cfg.runtime.web_root.clone();
    let runtime_user = runtime_user_for(&cfg.project_name);
    let logs_dir = paths::bonesremote_site_logs(&snapshot.site);
    fs::create_dir_all(&logs_dir).with_context(|| format!("Failed to create logs directory {}", logs_dir.display()))?;

    let env = PrepareScriptEnv {
        project_name: &cfg.project_name,
        project_root: &cfg.project_root,
        runtime_user: &runtime_user,
        web_root: &web_root,
        shared_functions: &shared_functions,
    };

    if cfg.runtime.backend == RuntimeBackend::Docker {
        let image = docker::command::image_name(&cfg.project_name)?;
        docker::prepare::run_scripts(&docker::prepare::PrepareRequest {
            project: &cfg.project_name,
            project_root: &snapshot.project_root,
            release: &release_dir,
            runtime_user: &runtime_user,
            image: &image,
            scripts: &scripts,
            functions: &shared_functions,
            logs_dir: &logs_dir,
        })?;
        return Ok(());
    }

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running prepare script {script_name}...");

        let status = run_prepare_script(&script, &release_dir, &logs_dir.join(format!("{script_name}.log")), &env)
            .with_context(|| format!("Failed to execute prepare script {}", script.display()))?;

        if !status.success() {
            bail!("Prepare script {script_name} exited with status {status}");
        }
    }

    Ok(())
}

fn run_prepare_script(
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
        .env(project_env::PROJECT_NAME, env.project_name)
        .env("PROJECT_ROOT", env.project_root)
        .env("REPO_PATH", "")
        .env(project_env::WEB_ROOT, env.web_root)
        .env("SERVICE_USER", env.runtime_user);
}

fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in
        fs::read_dir(scripts_dir).with_context(|| format!("Failed to read scripts dir: {}", scripts_dir.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.file_name().and_then(|name| name.to_str()).is_some_and(is_numbered_shell_script) {
            scripts.push(path);
        }
    }
    scripts.sort();
    Ok(scripts)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::{Command, ExitStatus, Stdio};
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::{Context, Result};

    use super::{join_prepare_input, list_scripts, stream_prepare_input};
    use crate::release::output;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{nanos}"));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

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
    fn list_scripts_sorts_prepare_scripts() -> Result<()> {
        let root = temp_dir("bonesremote-prepare-list")?;
        fs::write(root.join("02_second.sh"), "")?;
        fs::write(root.join("01_first.sh"), "")?;
        fs::write(root.join("README.md"), "# Prepare Scripts")?;
        fs::create_dir_all(root.join("nested"))?;

        let scripts = list_scripts(&root)?;

        assert_eq!(scripts, vec![root.join("01_first.sh"), root.join("02_second.sh")]);

        fs::remove_dir_all(&root).ok();
        Ok(())
    }

    #[test]
    fn shared_functions_precede_prepare_script() -> Result<()> {
        let root = temp_dir("bonesremote-prepare-prelude")?;
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
        let root = temp_dir("bonesremote-prepare-failure")?;
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
        let root = temp_dir("bonesremote-prepare-early-exit")?;
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
    fn framework_prepare_templates_do_not_source_control_plane_files() -> Result<()> {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../bonesdeploy/assets/frameworks");
        for framework in ["laravel", "rails", "django"] {
            let template = fs::read_to_string(
                root.join(framework).join("deployment/prepare").join(format!("01_prepare_{framework}.sh")),
            )?;
            assert!(
                !template.contains("DEPLOYMENT_DIR"),
                "{framework} prepare template still references DEPLOYMENT_DIR"
            );
            assert!(!template.contains("functions.sh"), "{framework} prepare template still sources functions.sh");
        }
        Ok(())
    }
}
