use std::io::Write;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Runs the local bonesinfra executable with the provided args.
pub fn run(args: &[&str]) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    let output = base_command(&executable)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("bonesinfra failed:\n{}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Runs the local bonesinfra executable with JSON piped to stdin.
pub fn run_with_stdin(args: &[&str], stdin_json: &str) -> Result<String> {
    let executable = super::bonesinfra::executable_path()?;

    let mut child = base_command(&executable)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), executable.display()))?;

    let mut stdin = child.stdin.take().context("Failed to capture bonesinfra stdin")?;
    stdin.write_all(stdin_json.as_bytes()).context("Failed to write JSON data to bonesinfra stdin")?;

    let output = child
        .wait_with_output()
        .with_context(|| format!("Failed to wait on bonesinfra {} from {}", args.join(" "), executable.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("bonesinfra failed:\n{}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
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
