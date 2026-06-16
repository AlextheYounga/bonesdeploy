use std::path::Path;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde_json::Value;

/// Runs `python3 .bones/infra/main.py <args>` and returns stdout as a String.
pub fn run_python(args: &[&str]) -> Result<String> {
    let main_py = Path::new(super::config::Constants::BONES_INFRA_MAIN);
    if !main_py.exists() {
        bail!(
            "{} not found. Run `bonesdeploy init` first to scaffold the infra entrypoint.",
            main_py.display()
        );
    }

    let output = Command::new("python3")
        .arg(main_py)
        .args(args)
        .output()
        .with_context(|| format!("Failed to run python3 {} {}", main_py.display(), args.join(" ")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("python3 {} failed:\n{}", main_py.display(), stderr.trim());
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// Runs a Python infra command that returns JSON, parses and returns it.
pub fn run_python_json(args: &[&str]) -> Result<Value> {
    let mut json_args = args.to_vec();
    json_args.push("--json");
    let stdout = run_python(&json_args)?;
    serde_json::from_str(&stdout).context("Failed to parse JSON output from Python infra CLI")
}

/// Returns the list of available runtime names from Python.
pub fn list_runtimes() -> Result<Vec<String>> {
    let value = run_python_json(&["runtime", "list"])?;
    match value {
        Value::Array(runtimes) => {
            runtimes
                .into_iter()
                .map(|v| {
                    v.as_str()
                        .map(String::from)
                        .context("Runtime name is not a string")
                })
                .collect()
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
