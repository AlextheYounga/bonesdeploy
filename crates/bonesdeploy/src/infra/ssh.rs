use std::process::Command;

use anyhow::{Context, Result, bail};
use openssh::{KnownHosts, Session, SessionBuilder, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::config::Bones;
use shared::config::{default_deploy_user, parse_port};

pub async fn connect(config: &Bones) -> Result<Session> {
    let host = &config.host;
    let port = parse_port(&config.port)?;
    let user = default_deploy_user();

    connect_as(&user, host, port).await
}

pub async fn connect_privileged(config: &Bones) -> Result<Session> {
    let host = &config.host;
    let port = parse_port(&config.port)?;

    connect_as(&config.ssh_user, host, port).await
}

pub async fn connect_as(user: &str, host: &str, port: u16) -> Result<Session> {
    SessionBuilder::default()
        .known_hosts_check(KnownHosts::Accept)
        .user(user.into())
        .port(port)
        .connect(host)
        .await
        .with_context(|| format!("Failed to connect to {user}@{host}:{port}"))
}

pub fn external_command(user: &str, host: &str, port: &str) -> Command {
    let mut command = Command::new("ssh");
    command
        .args(["-p", port, "-o", "StrictHostKeyChecking=no", "-o", "UserKnownHostsFile=/dev/null"])
        .arg(format!("{user}@{host}"));
    command
}

pub async fn run_cmd(session: &Session, cmd: &str) -> Result<String> {
    let output = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .output()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    if !output.status.success() {
        bail!("{}", format_remote_command_failure(cmd, &output.stdout, &output.stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn stream_cmd(session: &Session, cmd: &str) -> Result<()> {
    let mut child = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    let stdout = child.stdout().take().ok_or_else(|| anyhow::anyhow!("stdout was not piped"))?;
    let stderr = child.stderr().take().ok_or_else(|| anyhow::anyhow!("stderr was not piped"))?;

    let stdout_task = tokio::spawn(async move {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            println!("{line}");
        }
    });

    let stderr_task = tokio::spawn(async move {
        let reader = BufReader::new(stderr);
        let mut lines = reader.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            eprintln!("{line}");
        }
    });

    // Drain both streams concurrently before checking exit status
    let _ = tokio::join!(stdout_task, stderr_task);

    let status = child.wait().await.context("Failed to wait for remote command")?;

    if !status.success() {
        bail!("Remote command failed: {cmd}");
    }

    Ok(())
}

pub async fn run_cmd_with_stdin(session: &Session, cmd: &str, stdin_bytes: &[u8]) -> Result<()> {
    let mut child = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    let mut stdin = child.stdin().take().ok_or_else(|| anyhow::anyhow!("stdin was not piped"))?;
    stdin.write_all(stdin_bytes).await.context("Failed to write stdin to remote command")?;
    stdin.shutdown().await.context("Failed to close stdin for remote command")?;
    drop(stdin);

    let output = child.wait_with_output().await.context("Failed to wait for remote command")?;
    if !output.status.success() {
        bail!("{}", format_remote_command_failure(cmd, &output.stdout, &output.stderr));
    }

    Ok(())
}

fn format_remote_command_failure(cmd: &str, stdout: &[u8], stderr: &[u8]) -> String {
    let stdout = String::from_utf8_lossy(stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(stderr).trim().to_string();
    let mut message = format!("Remote command failed: {cmd}");

    if !stdout.is_empty() {
        message.push_str("\nstdout:\n");
        message.push_str(&stdout);
    }

    if !stderr.is_empty() {
        message.push_str("\nstderr:\n");
        message.push_str(&stderr);
    }

    message
}

#[cfg(test)]
mod tests {
    use super::format_remote_command_failure;

    #[test]
    fn remote_command_failure_includes_stdout_and_stderr() {
        let message = format_remote_command_failure(
            "bonesremote doctor --site demo",
            b"issue one\nissue two\n",
            b"Doctor found 2 issues\n",
        );

        assert!(message.contains("Remote command failed: bonesremote doctor --site demo"));
        assert!(message.contains("stdout:\nissue one\nissue two"));
        assert!(message.contains("stderr:\nDoctor found 2 issues"));
    }
}
