use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Stdio};
use std::thread;

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Runs the local bonesinfra executable with the provided args.
pub fn run(args: &[&str]) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    run_command(&executable, args, None)
}

/// Runs the local bonesinfra executable with JSON piped to stdin.
pub fn run_with_stdin(args: &[&str], stdin_json: &str) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    run_command(&executable, args, Some(stdin_json))
}
pub fn run_json(args: &[&str]) -> Result<Value> {
    let stdout = run(args)?;
    parse_json_output(&stdout)
}

fn parse_json_output(stdout: &str) -> Result<Value> {
    serde_json::from_str(stdout).context("Failed to parse JSON output from bonesinfra")
}

fn base_command(executable: &Path) -> Command {
    Command::new(executable)
}

fn run_command(executable: &Path, args: &[&str], stdin_json: Option<&str>) -> Result<String> {
    let mut command = base_command(executable);
    command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    if stdin_json.is_some() {
        command.stdin(Stdio::piped());
    }

    let mut child = command
        .spawn()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if let Some(stdin_json) = stdin_json {
        let mut stdin = child.stdin.take().context("Failed to capture bonesinfra stdin")?;
        stdin.write_all(stdin_json.as_bytes()).context("Failed to write JSON data to bonesinfra stdin")?;
    }

    let stdout = child.stdout.take().context("Failed to capture bonesinfra stdout")?;
    let stderr = child.stderr.take().context("Failed to capture bonesinfra stderr")?;

    let stdout_handle = thread::spawn(move || stream_reader(stdout, std::io::stdout()));
    let stderr_handle = thread::spawn(move || stream_reader(stderr, std::io::stderr()));

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait on bonesinfra {} from {}", args.join(" "), executable.display()))?;

    let stdout = stdout_handle.join().map_err(|_| anyhow::anyhow!("Failed to join bonesinfra stdout reader"))??;
    stderr_handle.join().map_err(|_| anyhow::anyhow!("Failed to join bonesinfra stderr reader"))??;

    if !status.success() {
        bail!("bonesinfra failed");
    }

    Ok(stdout)
}

fn stream_reader<R: Read, W: Write>(mut reader: R, mut writer: W) -> Result<String> {
    let mut buffer = [0_u8; 8192];
    let mut collected = Vec::new();

    loop {
        let count = reader.read(&mut buffer).context("Failed to read bonesinfra output")?;
        if count == 0 {
            break;
        }

        writer.write_all(&buffer[..count]).context("Failed to write bonesinfra output")?;
        writer.flush().context("Failed to flush bonesinfra output")?;
        collected.extend_from_slice(&buffer[..count]);
    }

    String::from_utf8(collected).context("bonesinfra output was not valid UTF-8")
}

/// Returns the questions for a given runtime from bonesinfra.
pub fn runtime_questions(runtime: &str) -> Result<Value> {
    run_json(&["runtime", "questions", runtime])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Result;

    use super::{base_command, parse_json_output};

    #[test]
    fn base_command_launches_bonesinfra_executable() {
        let command = base_command(Path::new("/tmp/bonesinfra/dist/bonesinfra"));

        assert_eq!(command.get_program().to_string_lossy(), "/tmp/bonesinfra/dist/bonesinfra");
        assert_eq!(command.get_args().count(), 0);
    }

    #[test]
    fn parse_json_output_reads_cli_stdout() -> Result<()> {
        let parsed = parse_json_output("[\"django\",\"rails\"]")?;
        assert_eq!(parsed, serde_json::json!(["django", "rails"]));
        Ok(())
    }
}
