use std::io::Write;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Runs the hidden bonesinfra entrypoint with the provided args.
pub fn run_python(args: &[&str]) -> Result<String> {
    let checkout = super::bonesinfra::checkout_path()?;

    let output = base_command(&checkout)
        .current_dir(&checkout)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), checkout.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("bonesinfra failed:\n{}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Runs the hidden bonesinfra entrypoint with JSON piped to stdin.
pub fn run_python_with_stdin(args: &[&str], stdin_json: &str) -> Result<String> {
    let checkout = super::bonesinfra::checkout_path()?;

    let mut child = base_command(&checkout)
        .current_dir(&checkout)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to run bonesinfra {} from {}", args.join(" "), checkout.display()))?;

    let mut stdin = child.stdin.take().context("Failed to capture python3 stdin")?;
    stdin.write_all(stdin_json.as_bytes()).context("Failed to write JSON data to python3 stdin")?;

    let output = child
        .wait_with_output()
        .with_context(|| format!("Failed to wait on bonesinfra {} from {}", args.join(" "), checkout.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("bonesinfra failed:\n{}", stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
pub fn run_python_json(args: &[&str]) -> Result<Value> {
    let mut json_args = args.to_vec();
    json_args.push("--json");
    let stdout = run_python(&json_args)?;
    serde_json::from_str(&stdout).context("Failed to parse JSON output from Python infra CLI")
}

fn base_command(checkout: &std::path::Path) -> Command {
    let mut command = Command::new("uv");
    command.args(["run", "--project"]);
    command.arg(checkout);
    command.arg("bonesinfra");
    command
}

/// Returns the list of available runtime names from Python.
pub fn list_runtimes() -> Result<Vec<String>> {
    let value = run_python_json(&["runtime", "list"])?;
    match value {
        Value::Array(runtimes) => {
            runtimes.into_iter().map(|v| v.as_str().map(String::from).context("Runtime name is not a string")).collect()
        }
        _ => bail!("Expected JSON array from runtime list"),
    }
}

/// Returns the questions for a given runtime from Python.
pub fn runtime_questions(runtime: &str) -> Result<Value> {
    run_python_json(&["runtime", "questions", runtime])
}

/// Returns the defaults for a given runtime from Python.
pub fn runtime_defaults(runtime: &str) -> Result<Value> {
    run_python_json(&["runtime", "defaults", runtime])
}
