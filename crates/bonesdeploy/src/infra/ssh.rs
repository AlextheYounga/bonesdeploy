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
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Remote command failed: {cmd}\n{stderr}");
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
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Remote command failed: {cmd}\n{stderr}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::external_command;

    #[test]
    fn external_command_uses_expected_ssh_options() {
        let command = external_command("root", "deploy.example.com", "2222");
        let args = command.get_args();
        let args = args.map(|arg| arg.to_string_lossy().into_owned()).collect::<Vec<_>>();

        assert_eq!(
            args,
            vec![
                String::from("-p"),
                String::from("2222"),
                String::from("-o"),
                String::from("StrictHostKeyChecking=no"),
                String::from("-o"),
                String::from("UserKnownHostsFile=/dev/null"),
                String::from("root@deploy.example.com"),
            ]
        );
    }
}
