mod common;

use std::fs;
use std::process::Command;

use anyhow::{Result, ensure};
use bonesdeploy::infra::ssh::shell_quote;
use common::{TestEnv, ssh::SshFixture};
use serde_json::json;

#[test]
fn site_status_renders_active_starting_and_inactive_previews() -> Result<()> {
    let env = TestEnv::new()?;
    let response = env.repo().join("remote-status.json");
    let script = format!(
        "#!/bin/sh\ncase \"$SSH_ORIGINAL_COMMAND\" in *'bonesremote status'*) cat {};; *) exit 0;; esac\n",
        shell_quote(&response.display().to_string())
    );
    let ssh = SshFixture::start(&script)?;
    let init = env.run(&[
        "init",
        "--non-interactive",
        "--project-name",
        "atlas",
        "--host",
        "127.0.0.1",
        "--port",
        &ssh.port.to_string(),
    ])?;
    ensure!(init.status.success(), "{}", String::from_utf8_lossy(&init.stderr));
    let username = String::from_utf8(Command::new("id").arg("-un").output()?.stdout)?.trim().to_owned();
    let dotenv = env.repo().join(".env");
    fs::write(&dotenv, fs::read_to_string(&dotenv)?.replace("SSH_USER=root", &format!("SSH_USER={username}")))?;

    for (preview, expected, absent) in [
        (
            json!({"active":true,"url":"https://preview.trycloudflare.com"}),
            Some("Preview: https://preview.trycloudflare.com"),
            "Quick Tunnel is starting",
        ),
        (
            json!({"active":true,"url":null}),
            Some("Quick Tunnel is starting; run `bonesdeploy site status` for its URL."),
            "Preview: ",
        ),
        (json!({"active":false,"url":null}), None, "Quick Tunnel is starting"),
    ] {
        fs::write(
            &response,
            serde_json::to_vec(&json!({
                "current_release":"release-1", "ssl":{"enabled":false,"domain":""},
                "preview":preview, "services":[], "compose":null
            }))?,
        )?;
        let output = ssh.run(&env, &["site", "status"])?;
        ensure!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(!stdout.contains("Remote unavailable"), "{stdout}");
        if let Some(expected) = expected {
            assert!(stdout.contains(expected), "{stdout}");
        }
        assert!(!stdout.contains(absent), "{stdout}");
        if expected.is_none() {
            assert!(!stdout.contains("Preview: "), "{stdout}");
        }
    }
    Ok(())
}
