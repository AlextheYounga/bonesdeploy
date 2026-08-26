use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};
use bonesdeploy_core::config::variables;

use super::command::{APP_PATH, SHARED_PATH, runtime_identity};
use crate::release::output;

pub(crate) struct PrepareRequest<'a> {
    pub project: &'a str,
    pub project_root: &'a Path,
    pub release: &'a Path,
    pub runtime_user: &'a str,
    pub image: &'a str,
    pub scripts: &'a [PathBuf],
    pub functions: &'a Path,
    pub logs_dir: &'a Path,
}

pub(crate) fn run_scripts(request: &PrepareRequest<'_>) -> Result<()> {
    let shared = request.project_root.join("shared");
    let identity = runtime_identity(request.runtime_user)?;
    if !shared.is_dir() {
        bail!("Shared directory is missing: {}", shared.display());
    }

    for script in request.scripts {
        let name = script.file_name().and_then(|value| value.to_str()).unwrap_or("<unknown>");
        let log = request.logs_dir.join(format!("{name}.log"));
        let mut command = Command::new("docker");
        command
            .args([
                "run",
                "--rm",
                "--cap-drop=ALL",
                "--security-opt=no-new-privileges",
                "--mount",
                &format!("type=bind,src={},dst={},rw", request.release.display(), APP_PATH),
                "--mount",
                &format!("type=bind,src={},dst={},rw", shared.display(), SHARED_PATH),
                "--workdir",
                APP_PATH,
                "--user",
                &identity,
                "--env",
                &format!("{}={}", variables::PROJECT_NAME, request.project),
                "--env",
                &format!("{}={}", variables::PROJECT_ROOT, request.project_root.display()),
                request.image,
                "bash",
                "-s",
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let mut child =
            command.spawn().with_context(|| format!("Failed to start Docker prepare container for {name}"))?;
        let stdin = child.stdin.take().context("Failed to capture Docker prepare stdin")?;
        let functions_file = fs::File::open(request.functions)?;
        let script_file = fs::File::open(script)?;
        let input = thread::spawn(move || stream_input(stdin, functions_file, script_file));
        let status = output::stream_child_output(&mut child, &log, &format!("Docker prepare script {name}"))?;
        input.join().map_err(|_| anyhow::anyhow!("Docker prepare input thread panicked"))??;
        if !status.success() {
            bail!("Prepare script {name} exited with status {status}");
        }
    }
    Ok(())
}

fn stream_input<W: Write>(mut output: W, mut functions: fs::File, mut script: fs::File) -> io::Result<()> {
    io::copy(&mut functions, &mut output)?;
    output.write_all(b"\n")?;
    io::copy(&mut script, &mut output)?;
    output.flush()
}
