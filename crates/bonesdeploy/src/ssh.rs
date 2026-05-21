use anyhow::{Context, Result, bail};
use openssh::{KnownHosts, Session, SessionBuilder, Stdio};
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::config::BonesConfig;

pub async fn connect(config: &BonesConfig) -> Result<Session> {
    let host = &config.data.host;
    let port: u16 = config.data.port.parse().with_context(|| format!("Invalid port: {}", config.data.port))?;
    let user = &config.permissions.defaults.deploy_user;

    let session = SessionBuilder::default()
        .known_hosts_check(KnownHosts::Accept)
        .user(user.clone())
        .port(port)
        .connect(host)
        .await
        .with_context(|| format!("Failed to connect to {user}@{host}:{port}"))?;

    Ok(session)
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
