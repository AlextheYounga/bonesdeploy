use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::Value;

pub fn run(args: &[&str]) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    run_interactive(&executable, args, None)
}

pub fn run_with_stdin(args: &[&str], stdin_json: &str) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    run_interactive(&executable, args, Some(stdin_json))
}
pub fn run_json(args: &[&str]) -> Result<Value> {
    let executable = super::bonesinfra::executable_path()?;
    let stdout = run_captured(&executable, args)?;
    parse_json_output(&stdout)
}

fn parse_json_output(stdout: &str) -> Result<Value> {
    serde_json::from_str(stdout).context("Failed to parse JSON output from bonesinfra")
}

fn base_command(executable: &Path) -> Command {
    let mut cmd = Command::new(executable);
    cmd.args(["-m", "bonesinfra"]);
    cmd
}

fn run_interactive(executable: &Path, args: &[&str], stdin_json: Option<&str>) -> Result<String> {
    let mut command = base_command(executable);
    command.args(args);
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

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait on bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !status.success() {
        bail!("bonesinfra failed");
    }

    Ok(String::new())
}

fn run_captured(executable: &Path, args: &[&str]) -> Result<String> {
    let output = base_command(executable)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("bonesinfra failed:\n{}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
    fn base_command_launches_venv_python_with_module_flag() {
        let command = base_command(Path::new("/tmp/bonesinfra/.venv/bin/python"));

        assert_eq!(command.get_program().to_string_lossy(), "/tmp/bonesinfra/.venv/bin/python");
        let args: Vec<_> = command.get_args().map(|a| a.to_string_lossy().to_string()).collect();
        assert_eq!(args, vec!["-m", "bonesinfra"]);
    }

    #[test]
    fn parse_json_output_reads_cli_stdout() -> Result<()> {
        let parsed = parse_json_output("[\"django\",\"rails\"]")?;
        assert_eq!(parsed, serde_json::json!(["django", "rails"]));
        Ok(())
    }
}
