use std::process::ExitCode;

#[tokio::main]
async fn main() -> ExitCode {
    bonesdeploy::run_cli().await
}
