mod common;

use std::fs;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy::cli::args::Cli;
use clap::Parser;
use common::TestEnv;
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

#[test]
fn missing_server_reports_the_setup_sequence_and_omits_ssl_for_domainless_sites() -> Result<()> {
    for has_domain in [false, true] {
        let env = TestEnv::new()?;
        let args = ["init", "--non-interactive", "--project-name", "atlas", "--host", "127.0.0.1", "--port", "1"];
        let init = env.run(&args)?;
        ensure!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
        if has_domain {
            let path = env.repo().join(".env");
            let contents = fs::read_to_string(&path)?.replace("BONES_DOMAIN=", "BONES_DOMAIN=atlas.example.com");
            fs::write(path, contents)?;
        }
        let output = env.run(&["skill", "next", "--format", "json"])?;
        ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let report: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(report["state"], "server_missing");
        assert_eq!(report["next"]["command"], "bonesdeploy server setup --yes");
        assert_eq!(report["next"]["contacts_remote"], true);
        let commands = report["commands"].as_array().ok_or_else(|| anyhow::anyhow!("missing commands"))?;
        let mut expected = vec!["bonesdeploy server setup --yes", "bonesdeploy site setup --yes"];
        if has_domain {
            expected.push("bonesdeploy site ssl --yes");
        }
        expected.push("bonesdeploy deploy");
        let actual = commands.iter().filter_map(Value::as_str).collect::<Vec<_>>();
        assert_eq!(actual, expected);
        for command in actual {
            Cli::try_parse_from(command.split_whitespace())?;
        }
    }
    Ok(())
}
