use std::io;
use std::process::ExitStatus;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use bonesdeploy_core::config::{default_deploy_user, parse_port};
use openssh::{Session, SessionBuilder, Stdio};
use tokio::io::{self as tokio_io, AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::time::{Instant, timeout, timeout_at};

use crate::config::Bones;

macro_rules! complete_operation {
    ($child:ident, $result:expr) => {
        match $result {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => {
                disconnect_child($child).await;
                return Err(error).context("SSH transport failed during remote operation");
            }
            Err(_) => {
                disconnect_child($child).await;
                bail!("SSH operation timed out");
            }
        }
    };
}

const DEFAULT_CONNECT_DEADLINE: Duration = Duration::from_secs(30);
const DEFAULT_COMMAND_DEADLINE: Duration = Duration::from_secs(30 * 60);
const DEFAULT_TRANSFER_DEADLINE: Duration = Duration::from_secs(2 * 60 * 60);
const DEFAULT_REMOTE_OUTPUT_TAIL_LIMIT: usize = 64 * 1024;
const DEFAULT_COMMAND_OUTPUT_LIMIT: usize = 64 * 1024;
const CHILD_CLEANUP_DEADLINE: Duration = Duration::from_secs(5);

/// Deadlines and diagnostic limits applied to every SSH transport operation.
#[derive(Clone, Copy, Debug)]
pub struct TransportPolicy {
    pub connect_deadline: Duration,
    pub command_deadline: Duration,
    pub transfer_deadline: Duration,
    pub remote_output_tail_limit: usize,
    pub command_output_limit: usize,
}

impl Default for TransportPolicy {
    fn default() -> Self {
        Self {
            connect_deadline: DEFAULT_CONNECT_DEADLINE,
            command_deadline: DEFAULT_COMMAND_DEADLINE,
            transfer_deadline: DEFAULT_TRANSFER_DEADLINE,
            remote_output_tail_limit: DEFAULT_REMOTE_OUTPUT_TAIL_LIMIT,
            command_output_limit: DEFAULT_COMMAND_OUTPUT_LIMIT,
        }
    }
}

/// The SSH session and the policy governing work performed through it.
pub struct SshTransport {
    session: Session,
    policy: TransportPolicy,
}

impl SshTransport {
    pub async fn connect(config: &Bones) -> Result<Self> {
        Self::connect_as(&default_deploy_user(), &config.host, parse_port(&config.port)?).await
    }

    pub async fn connect_privileged(config: &Bones) -> Result<Self> {
        Self::connect_as(&config.ssh_user, &config.host, parse_port(&config.port)?).await
    }

    pub async fn connect_as(user: &str, host: &str, port: u16) -> Result<Self> {
        Self::connect_as_with_policy(user, host, port, TransportPolicy::default()).await
    }

    pub async fn connect_as_with_policy(user: &str, host: &str, port: u16, policy: TransportPolicy) -> Result<Self> {
        let mut builder = SessionBuilder::default();
        let connect = builder.user(user.into()).port(port).connect(host);
        let session = timeout(policy.connect_deadline, connect)
            .await
            .map_err(|_| anyhow!("SSH connection timed out"))?
            .with_context(|| format!("SSH transport failed while connecting to {user}@{host}:{port}"))?;
        Ok(Self { session, policy })
    }

    /// Wraps an already-established session in the transport policy boundary.
    pub fn from_session(session: Session, policy: TransportPolicy) -> Self {
        Self { session, policy }
    }

    pub async fn close(self) -> Result<()> {
        self.session.close().await.context("SSH transport failed while closing the session")
    }

    pub async fn run_cmd(&self, cmd: &str) -> Result<String> {
        let deadline = Instant::now() + self.policy.command_deadline;
        let mut child = self.spawn(cmd, Stdio::null(), deadline).await?;
        let stdout = take_stdout(&mut child)?;
        let stderr = take_stderr(&mut child)?;
        let result = timeout_at(deadline, async {
            let (stdout, stderr) = tokio::try_join!(
                read_command_output(stdout, self.policy.command_output_limit),
                drain_stream(stderr, true, false, self.policy.remote_output_tail_limit)
            )
            .map_err(|error| anyhow!(error))?;
            Ok::<_, anyhow::Error>((stdout, stderr))
        })
        .await;
        let (stdout, stderr) = complete_operation!(child, result);
        let status = wait_for_child(child, deadline).await?;
        ensure_success(status.success(), &tail_from_bytes(&stdout, self.policy.remote_output_tail_limit), &stderr)?;
        Ok(String::from_utf8_lossy(&stdout).to_string())
    }

    pub async fn stream_cmd(&self, cmd: &str) -> Result<()> {
        self.stream::<tokio_io::Empty>(cmd, None, None).await
    }

    pub async fn download_cmd<W>(&self, cmd: &str, destination: &mut W) -> Result<()>
    where
        W: AsyncWrite + Unpin,
    {
        let deadline = Instant::now() + self.policy.transfer_deadline;
        let mut child = self.spawn(cmd, Stdio::null(), deadline).await?;
        let stdout = take_stdout(&mut child)?;
        let stderr = take_stderr(&mut child)?;
        let result = timeout_at(deadline, async {
            tokio::try_join!(
                copy_with_tail(stdout, destination, self.policy.remote_output_tail_limit),
                drain_stream(stderr, true, false, self.policy.remote_output_tail_limit)
            )
            .map_err(|error| anyhow!(error))
        })
        .await;
        let (stdout, stderr) = complete_operation!(child, result);
        let status = wait_for_child(child, deadline).await?;
        ensure_success(status.success(), &stdout, &stderr)
    }

    pub async fn run_cmd_with_stdin(&self, cmd: &str, stdin_bytes: &[u8]) -> Result<()> {
        self.run_cmd_with_stdin_output(cmd, stdin_bytes).await.map(|_| ())
    }

    pub async fn run_cmd_with_stdin_output(&self, cmd: &str, stdin_bytes: &[u8]) -> Result<String> {
        let deadline = Instant::now() + self.policy.command_deadline;
        let mut child = self.spawn(cmd, Stdio::piped(), deadline).await?;
        let stdin = take_stdin(&mut child)?;
        let stdout = take_stdout(&mut child)?;
        let stderr = take_stderr(&mut child)?;
        let result = timeout_at(deadline, async {
            let ((), stdout, stderr) = tokio::try_join!(
                write_bytes(stdin, stdin_bytes),
                read_command_output(stdout, self.policy.command_output_limit),
                drain_stream(stderr, true, false, self.policy.remote_output_tail_limit)
            )
            .map_err(|error| anyhow!(error))?;
            Ok::<_, anyhow::Error>((stdout, stderr))
        })
        .await;
        let (stdout, stderr) = complete_operation!(child, result);
        let status = wait_for_child(child, deadline).await?;
        ensure_success(status.success(), &tail_from_bytes(&stdout, self.policy.remote_output_tail_limit), &stderr)?;
        Ok(String::from_utf8_lossy(&stdout).to_string())
    }

    pub async fn stream_cmd_with_stdin(&self, cmd: &str, stdin_bytes: &[u8]) -> Result<()> {
        self.stream::<tokio_io::Empty>(cmd, Some(stdin_bytes), None).await
    }

    /// Streams a small protocol prefix and an asynchronous payload without buffering the payload.
    pub async fn stream_cmd_with_reader<R>(&self, cmd: &str, prefix: &[u8], reader: R) -> Result<()>
    where
        R: AsyncRead + Unpin,
    {
        self.stream(cmd, Some(prefix), Some(reader)).await
    }

    async fn stream<R>(&self, cmd: &str, bytes: Option<&[u8]>, reader: Option<R>) -> Result<()>
    where
        R: AsyncRead + Unpin,
    {
        let duration = if reader.is_some() { self.policy.transfer_deadline } else { self.policy.command_deadline };
        let deadline = Instant::now() + duration;
        let mut child = self.spawn(cmd, if bytes.is_some() { Stdio::piped() } else { Stdio::null() }, deadline).await?;
        let stdin = bytes.map(|_| take_stdin(&mut child)).transpose()?;
        let stdout = take_stdout(&mut child)?;
        let stderr = take_stderr(&mut child)?;
        let result = timeout_at(deadline, async {
            let upload = async {
                if let Some(mut stdin) = stdin {
                    if let Some(bytes) = bytes {
                        stdin.write_all(bytes).await?;
                    }
                    if let Some(mut reader) = reader {
                        tokio_io::copy(&mut reader, &mut stdin).await?;
                    }
                    stdin.shutdown().await?;
                }
                Ok::<(), io::Error>(())
            };
            let ((), stdout, stderr) = tokio::try_join!(
                upload,
                drain_stream(stdout, false, true, self.policy.remote_output_tail_limit),
                drain_stream(stderr, true, true, self.policy.remote_output_tail_limit)
            )
            .map_err(|error| anyhow!(error))?;
            Ok::<_, anyhow::Error>((stdout, stderr))
        })
        .await;
        let (stdout, stderr) = complete_operation!(child, result);
        let status = wait_for_child(child, deadline).await?;
        ensure_success(status.success(), &stdout, &stderr)
    }

    async fn spawn(&self, cmd: &str, stdin: Stdio, deadline: Instant) -> Result<openssh::RemoteChild<'_>> {
        timeout_at(
            deadline,
            self.session
                .command("bash")
                .arg("-c")
                .arg(cmd)
                .stdin(stdin)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn(),
        )
        .await
        .map_err(|_| anyhow!("SSH operation timed out"))?
        .context("SSH transport failed while starting remote command")
    }
}

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn take_stdin(child: &mut openssh::RemoteChild<'_>) -> Result<openssh::ChildStdin> {
    child.stdin().take().ok_or_else(|| anyhow!("SSH transport failed: stdin was not piped"))
}

async fn wait_for_child(child: openssh::RemoteChild<'_>, deadline: Instant) -> Result<ExitStatus> {
    // `wait` consumes the child; timing it out drops that handle, which openssh
    // documents as terminating the local SSH process for the remote channel.
    timeout_at(deadline, child.wait())
        .await
        .map_err(|_| anyhow!("SSH operation timed out"))?
        .context("SSH transport failed while waiting for remote command")
}

fn take_stdout(child: &mut openssh::RemoteChild<'_>) -> Result<openssh::ChildStdout> {
    child.stdout().take().ok_or_else(|| anyhow!("SSH transport failed: stdout was not piped"))
}

fn take_stderr(child: &mut openssh::RemoteChild<'_>) -> Result<openssh::ChildStderr> {
    child.stderr().take().ok_or_else(|| anyhow!("SSH transport failed: stderr was not piped"))
}

async fn write_bytes(mut stdin: openssh::ChildStdin, bytes: &[u8]) -> io::Result<()> {
    stdin.write_all(bytes).await?;
    stdin.shutdown().await
}

async fn read_command_output(mut stream: openssh::ChildStdout, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = stream.read(&mut buffer).await?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len().saturating_add(count) > limit {
            return Err(io::Error::new(io::ErrorKind::Other, "remote command output exceeded the configured limit"));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

async fn copy_with_tail<W>(
    mut stream: openssh::ChildStdout,
    destination: &mut W,
    limit: usize,
) -> io::Result<OutputTail>
where
    W: AsyncWrite + Unpin,
{
    let mut tail = OutputTail::new(limit);
    let mut buffer = [0; 8192];
    loop {
        let count = stream.read(&mut buffer).await?;
        if count == 0 {
            return Ok(tail);
        }
        destination.write_all(&buffer[..count]).await?;
        tail.push(&buffer[..count]);
    }
}

async fn drain_stream<R>(stream: R, stderr: bool, live: bool, limit: usize) -> io::Result<OutputTail>
where
    R: AsyncRead + Unpin,
{
    let mut reader = BufReader::new(stream);
    let mut tail = OutputTail::new(limit);
    let mut line = Vec::new();
    loop {
        line.clear();
        let count = reader.read_until(b'\n', &mut line).await?;
        if count == 0 {
            return Ok(tail);
        }
        tail.push(&line);
        let text = String::from_utf8_lossy(&line);
        if live {
            if stderr {
                eprint!("{text}");
            } else {
                print!("{text}");
            }
        }
    }
}

async fn disconnect_child(child: openssh::RemoteChild<'_>) {
    let _ = timeout(CHILD_CLEANUP_DEADLINE, child.disconnect()).await;
}

#[derive(Debug)]
struct OutputTail {
    bytes: Vec<u8>,
    limit: usize,
    truncated: bool,
}

impl OutputTail {
    fn new(limit: usize) -> Self {
        Self { bytes: Vec::new(), limit, truncated: false }
    }

    fn push(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
        if self.bytes.len() > self.limit {
            self.bytes.drain(..self.bytes.len() - self.limit);
            self.truncated = true;
        }
    }
}

fn tail_from_bytes(bytes: &[u8], limit: usize) -> OutputTail {
    let mut tail = OutputTail::new(limit);
    tail.push(bytes);
    tail
}

fn ensure_success(success: bool, stdout: &OutputTail, stderr: &OutputTail) -> Result<()> {
    if !success {
        bail!("{}", remote_command_failure(stdout, stderr));
    }
    Ok(())
}

fn remote_command_failure(stdout: &OutputTail, stderr: &OutputTail) -> String {
    let mut message = String::from("SSH remote command exited unsuccessfully");
    append_tail(&mut message, "stdout", stdout);
    append_tail(&mut message, "stderr", stderr);
    message
}

fn append_tail(message: &mut String, name: &str, tail: &OutputTail) {
    if tail.bytes.is_empty() {
        return;
    }
    message.push_str("\n");
    message.push_str(name);
    message.push_str(" tail");
    if tail.truncated {
        message.push_str(" (truncated)");
    }
    message.push_str(":\n");
    message.push_str(String::from_utf8_lossy(&tail.bytes).trim());
}
