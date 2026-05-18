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

    let extra_args: Vec<String> = env::args().skip(1).collect();

    let mut test_command = Command::new("cargo");
    test_command.args([
        "test",
        "--manifest-path",
        workspace_root.join("Cargo.toml").to_str().unwrap(),
        "-p",
        "bonesdeploy-e2e-tests",
        "--",
        "--ignored",
        "--nocapture",
        "--test-threads=1",
    ]);
    test_command.args(extra_args);

    let status = test_command.status().map_err(|error| format!("Failed to run cargo test: {error}"))?;

    if status.success() {
        return Ok(ExitCode::SUCCESS);
    }

    Ok(ExitCode::FAILURE)
}

fn workspace_root() -> Result<PathBuf, String> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .map(|path| path.to_path_buf())
        .ok_or_else(|| String::from("Failed to resolve workspace root from CARGO_MANIFEST_DIR"))
}

fn run_command(command: &mut Command, context: &str) -> Result<(), String> {
    let status = command.status().map_err(|error| format!("{context}: {error}"))?;

    if status.success() {
        return Ok(());
    }

    Err(format!("{context}: exited with status {status}"))
}
