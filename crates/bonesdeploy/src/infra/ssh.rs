use std::io;

use anyhow::{Context, Result, bail};
use openssh::{Session, SessionBuilder, Stdio};
use tokio::io::{self as tokio_io, AsyncBufReadExt, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};

use crate::config::Bones;
use bonesdeploy_core::config::{default_deploy_user, parse_port};

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
    // Default KnownHosts::Add is accept-new: an unknown host key is enrolled on
    // first contact, but a *changed* key is rejected. Never use KnownHosts::Accept
    // (which maps to StrictHostKeyChecking=no) for control-plane SSH.
    SessionBuilder::default()
        .user(user.into())
        .port(port)
        .connect(host)
        .await
        .with_context(|| format!("Failed to connect to {user}@{host}:{port}"))
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
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
        bail!("{}", remote_command_failure(cmd, &output.stdout, &output.stderr));
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

pub async fn download_cmd<W>(session: &Session, cmd: &str, destination: &mut W) -> Result<()>
where
    W: AsyncWrite + Unpin,
{
    let mut child = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    let mut stdout = child.stdout().take().ok_or_else(|| anyhow::anyhow!("stdout was not piped"))?;
    let mut stderr = child.stderr().take().ok_or_else(|| anyhow::anyhow!("stderr was not piped"))?;
    let stderr_task = tokio::spawn(async move {
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).await?;
        Ok::<_, io::Error>(bytes)
    });

    if let Err(error) = tokio_io::copy(&mut stdout, destination).await {
        drop(stdout);
        let _ = child.disconnect().await;
        let _ = stderr_task.await;
        return Err(error).context("Failed to write the remote download");
    }
    drop(stdout);

    let stderr = stderr_task.await.context("Failed to join remote stderr reader")??;
    let status = child.wait().await.context("Failed to wait for remote command")?;
    if !status.success() {
        bail!("{}", remote_command_failure(cmd, &[], &stderr));
    }

    Ok(())
}

pub async fn run_cmd_with_stdin(session: &Session, cmd: &str, stdin_bytes: &[u8]) -> Result<()> {
    run_cmd_with_stdin_output(session, cmd, stdin_bytes).await.map(|_| ())
}

pub async fn run_cmd_with_stdin_output(session: &Session, cmd: &str, stdin_bytes: &[u8]) -> Result<String> {
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
        bail!("{}", remote_command_failure(cmd, &output.stdout, &output.stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub async fn stream_cmd_with_stdin(session: &Session, cmd: &str, stdin_bytes: &[u8]) -> Result<()> {
    let mut child = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;

    let mut stdin = child.stdin().take().ok_or_else(|| anyhow::anyhow!("stdin was not piped"))?;
    stdin.write_all(stdin_bytes).await.context("Failed to write stdin to remote command")?;
    stdin.shutdown().await.context("Failed to close stdin for remote command")?;
    drop(stdin);

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

    let _ = tokio::join!(stdout_task, stderr_task);

    let status = child.wait().await.context("Failed to wait for remote command")?;

    if !status.success() {
        bail!("Remote command failed: {cmd}");
    }

    Ok(())
}

/// Streams a small protocol prefix and an asynchronous payload without buffering
/// the payload while remote output is drained concurrently.
pub async fn stream_cmd_with_reader<R>(session: &Session, cmd: &str, prefix: &[u8], mut reader: R) -> Result<()>
where
    R: tokio_io::AsyncRead + Unpin,
{
    let mut child = session
        .command("bash")
        .arg("-c")
        .arg(cmd)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .await
        .with_context(|| format!("Failed to execute remote command: {cmd}"))?;
    let mut stdin = child.stdin().take().ok_or_else(|| anyhow::anyhow!("stdin was not piped"))?;
    let stdout = child.stdout().take().ok_or_else(|| anyhow::anyhow!("stdout was not piped"))?;
    let stderr = child.stderr().take().ok_or_else(|| anyhow::anyhow!("stderr was not piped"))?;
    let stdout_task = tokio::spawn(drain_lines(stdout, false));
    let stderr_task = tokio::spawn(drain_lines(stderr, true));

    let upload = async {
        stdin.write_all(prefix).await.context("Failed to write artifact manifest to remote command")?;
        tokio_io::copy(&mut reader, &mut stdin).await.context("Failed to stream artifact to remote command")?;
        stdin.shutdown().await.context("Failed to close artifact upload")?;
        drop(stdin);
        Ok::<(), anyhow::Error>(())
    }
    .await;
    if let Err(error) = upload {
        let _ = child.disconnect().await;
        let _ = tokio::join!(stdout_task, stderr_task);
        return Err(error);
    }

    let (stdout_result, stderr_result) = tokio::join!(stdout_task, stderr_task);
    stdout_result.context("Failed to join remote stdout reader")??;
    stderr_result.context("Failed to join remote stderr reader")??;
    let status = child.wait().await.context("Failed to wait for remote command")?;
    if !status.success() {
        bail!("Remote command failed: {cmd}");
    }
    Ok(())
}

async fn drain_lines<R>(stream: R, stderr: bool) -> io::Result<()>
where
    R: tokio_io::AsyncRead + Unpin,
{
    let mut lines = BufReader::new(stream).lines();
    while let Some(line) = lines.next_line().await? {
        if stderr {
            eprintln!("{line}");
        } else {
            println!("{line}");
        }
    }
    Ok(())
}

pub fn remote_command_failure(cmd: &str, stdout: &[u8], stderr: &[u8]) -> String {
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
