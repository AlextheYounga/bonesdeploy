use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy::cli::args::Cli;
use clap::Parser;
use serde_json::Value;

#[test]
fn uninitialized_project_reports_a_valid_local_init_command() -> Result<()> {
    let directory = tempfile::tempdir()?;
    let output = Command::new(env!("CARGO_BIN_EXE_bonesdeploy"))
        .args(["skill", "next", "--format", "json"])
        .current_dir(directory.path())
        .output()?;
    ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let report: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(report["state"], "uninitialized");
    assert_eq!(report["next"]["contacts_remote"], false);
    let command = report["next"]["command"].as_str().ok_or_else(|| anyhow::anyhow!("missing next command"))?;
    assert!(command.starts_with("bonesdeploy init "));
    Cli::try_parse_from(command.split_whitespace())?;
    Ok(())
}
