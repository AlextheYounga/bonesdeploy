mod common;

use std::fs;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy::cli::args::Cli;
use bonesdeploy::infra::ssh::shell_quote;
use clap::Parser;
use common::{TestEnv, ssh::SshFixture};
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

#[test]
fn readiness_guides_site_ssl_and_ready_states_from_remote_checks() -> Result<()> {
    let env = TestEnv::new()?;
    let marker = |name: &str| shell_quote(&env.repo().join(name).display().to_string());
    let script = format!(
        "#!/bin/sh\ncase \"$SSH_ORIGINAL_COMMAND\" in\n  *fullchain.pem*) test -e {};;\n  *shared/.env*) test -e {};;\n  *current*) test -e {};;\n  *) exit 0;;\nesac\n",
        marker("ssl-ready"),
        marker("site-ready"),
        marker("site-ready")
    );
    let fixture = SshFixture::start(&script)?;
    let init = env.run(&[
        "init",
        "--non-interactive",
        "--project-name",
        "atlas",
        "--host",
        "127.0.0.1",
        "--port",
        &fixture.port.to_string(),
    ])?;
    ensure!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let username = String::from_utf8(Command::new("id").arg("-un").output()?.stdout)?.trim().to_owned();
    let dotenv = env.repo().join(".env");
    let original = fs::read_to_string(&dotenv)?.replace("SSH_USER=root", &format!("SSH_USER={username}"));
    let with_domain = original.replace("BONES_DOMAIN=", "BONES_DOMAIN=atlas.example.com");
    fs::write(&dotenv, &with_domain)?;

    for (site_ready, ssl_ready, domain, state, expected) in [
        (
            false,
            false,
            true,
            "site_missing",
            vec!["bonesdeploy site setup --yes", "bonesdeploy site ssl --yes", "bonesdeploy deploy"],
        ),
        (false, false, false, "site_missing", vec!["bonesdeploy site setup --yes", "bonesdeploy deploy"]),
        (true, false, true, "ssl_missing", vec!["bonesdeploy site ssl --yes", "bonesdeploy deploy"]),
        (true, true, true, "ready", vec!["bonesdeploy deploy"]),
        (true, false, false, "ready", vec!["bonesdeploy deploy"]),
    ] {
        fs::write(&dotenv, if domain { &with_domain } else { &original })?;
        for (name, present) in [("site-ready", site_ready), ("ssl-ready", ssl_ready)] {
            let path = env.repo().join(name);
            if present {
                fs::write(path, "")?;
            } else if path.exists() {
                fs::remove_file(path)?;
            }
        }
        let output = fixture.run(&env, &["skill", "next", "--format", "json"])?;
        ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let report: Value = serde_json::from_slice(&output.stdout)?;
        assert_eq!(report["state"], state);
        let actual = report["commands"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("missing commands"))?
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
        assert_eq!(report["next"]["command"], expected[0]);
        for command in actual {
            Cli::try_parse_from(command.split_whitespace())?;
        }
    }
    Ok(())
}
