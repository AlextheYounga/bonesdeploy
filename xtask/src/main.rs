use std::env;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        return Err(String::from("Missing command. Usage: cargo e2e [test-args...]"));
    };

    match command.as_str() {
        "e2e" => run_e2e(args.collect()),
        _ => Err(format!("Unknown command '{command}'. Supported: e2e")),
    }
}

fn run_e2e(extra_test_args: Vec<String>) -> Result<ExitCode, String> {
    let workspace_root = workspace_root()?;
    let compose_file = workspace_root.join("docker/docker-compose.yml");

    run_command(
        Command::new("docker").arg("compose").arg("-f").arg(&compose_file).arg("down").arg("--remove-orphans"),
        "docker compose down failed",
    )?;

    run_command(
        Command::new("docker").arg("compose").arg("-f").arg(&compose_file).arg("up").arg("-d"),
        "docker compose up failed",
    )?;

    let mut test_command = Command::new("cargo");
    test_command.args(["test", "-p", "bonesdeploy-e2e-tests", "--", "--ignored", "--nocapture"]);
    test_command.args(extra_test_args);
    let status = test_command.status().map_err(|error| format!("Failed to start cargo test: {error}"))?;

    if status.success() {
        return Ok(ExitCode::SUCCESS);
    }

    Ok(ExitCode::FAILURE)
}

fn workspace_root() -> Result<PathBuf, String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.parent().map(|value| value.to_path_buf()).ok_or_else(|| String::from("Failed to resolve workspace root"))
}

fn run_command(command: &mut Command, context: &str) -> Result<(), String> {
    let status = command.status().map_err(|error| format!("{context}: {error}"))?;
    if status.success() {
        return Ok(());
    }

    Err(format!("{context}: exited with status {status}"))
}
