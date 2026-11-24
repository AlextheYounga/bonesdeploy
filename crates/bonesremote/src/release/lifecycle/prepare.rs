use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::{RuntimeBackend, is_numbered_shell_script, runtime_user_for, variables};
use bonesdeploy_core::paths;

use crate::privileges;
use crate::release::SiteMutation;
use crate::release::output;

/// Executes a prepare script with its generic site runtime context.
pub struct PrepareScript<'a> {
    pub project_name: &'a str,
    pub project_root: &'a str,
    pub runtime_user: &'a str,
    pub web_root: &'a str,
    pub shared_functions: &'a Path,
}

pub fn run(mutation: &SiteMutation, snapshot: &super::DeploymentSnapshot) -> Result<()> {
    privileges::ensure_root("bonesremote release prepare")?;

    let cfg = &snapshot.config;
    if cfg.runtime.backend == RuntimeBackend::Docker {
        println!("Compose deployment; skipping numbered prepare scripts.");
        return Ok(());
    }
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

    let release_name = mutation.required_staged_release()?;
    let release_dir = mutation.release_dir(&release_name);
    if !release_dir.is_dir() {
        bail!("Promoted release is missing: {}", release_dir.display());
    }

    let web_root = cfg.runtime.web_root.clone();
    let runtime_user = runtime_user_for(&cfg.project_name);
    let logs_dir = paths::bonesremote_site_logs(&snapshot.site);
    fs::create_dir_all(&logs_dir).with_context(|| format!("Failed to create logs directory {}", logs_dir.display()))?;

    let prepare_script = PrepareScript {
        project_name: &cfg.project_name,
        project_root: &cfg.project_root,
        runtime_user: &runtime_user,
        web_root: &web_root,
        shared_functions: &shared_functions,
    };

    for script in scripts {
        let script_name = script.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>");
        println!("Running prepare script {script_name}...");

        let status = prepare_script
            .run(&script, &release_dir, &logs_dir.join(format!("{script_name}.log")))
            .with_context(|| format!("Failed to execute prepare script {}", script.display()))?;

        if !status.success() {
            bail!("Prepare script {script_name} exited with status {status}");
        }
    }

    Ok(())
}

impl<'a> PrepareScript<'a> {
    /// Runs the script as the runtime user, streaming output to the log.
    /// The caller is responsible for authorizing the operation.
    pub fn run(&self, script: &Path, release_root: &Path, log_path: &Path) -> Result<ExitStatus> {
        let functions_file = fs::File::open(self.shared_functions)
            .with_context(|| format!("Failed to open shared prepare functions {}", self.shared_functions.display()))?;
        let script_file =
            fs::File::open(script).with_context(|| format!("Failed to open prepare script {}", script.display()))?;

        let mut command = Command::new("runuser");
        configure_prepare_command(&mut command, release_root, self);

        let mut child =
            command.stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().with_context(|| {
                format!("Failed to execute prepare script {} as {}", script.display(), self.runtime_user)
            })?;
        let stdin = child.stdin.take().context("Failed to capture prepare script stdin")?;
        let input_handle = thread::spawn(move || stream_prepare_input(stdin, functions_file, script_file));

        let status = output::stream_child_output(&mut child, log_path, &format!("prepare script {}", script.display()));
        join_prepare_input(input_handle, self.shared_functions, script)?;
        status
    }
}

pub fn stream_prepare_input<W: Write>(
    mut stdin: W,
    mut functions_file: fs::File,
    mut script_file: fs::File,
) -> io::Result<()> {
    io::copy(&mut functions_file, &mut stdin)?;
    stdin.write_all(b"\n")?;
    io::copy(&mut script_file, &mut stdin)?;
    stdin.flush()
}

pub fn join_prepare_input(
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

fn configure_prepare_command(command: &mut Command, release_root: &Path, env: &PrepareScript<'_>) {
    command
        .args(["-u", env.runtime_user, "--", "bash", "-c", "umask 0002; exec bash -s"])
        .current_dir(release_root)
        .env(variables::PROJECT_NAME, env.project_name)
        .env(variables::PROJECT_ROOT, env.project_root)
        .env(variables::REPO_PATH, "")
        .env(variables::WEB_ROOT, env.web_root)
        .env(variables::SERVICE_USER, env.runtime_user);
}

pub fn list_scripts(scripts_dir: &Path) -> Result<Vec<PathBuf>> {
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
