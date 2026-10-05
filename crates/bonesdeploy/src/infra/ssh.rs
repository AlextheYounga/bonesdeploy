use std::ffi::OsString;
use std::io;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use bonesdeploy_core::config::{default_deploy_user, parse_port};
use tokio::io::{self as tokio_io, AsyncRead, AsyncWrite, AsyncWriteExt};
use tokio::time::{Instant, timeout_at};

use crate::config::Bones;

mod output;
mod process;

use output::{copy_with_tail, drain_stream, ensure_success, read_command_output, tail_from_bytes};
use process::{disconnect_child, ssh_executable, take_stderr, take_stdin, take_stdout, wait_for_child, write_bytes};

macro_rules! complete_operation {
    ($child:ident, $result:expr) => {
        match $result {
            Ok(Ok(value)) => value,
            Ok(Err(error)) => {
                disconnect_child(&mut $child).await;
                return Err(error).context("SSH transport failed during remote operation");
            }
            Err(_) => {
                disconnect_child(&mut $child).await;
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

pub fn shell_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

#[doc(hidden)]
pub struct SshCommand {
    pub executable: PathBuf,
    pub connection_args: Vec<OsString>,
}

#[doc(hidden)]
pub struct SshTarget<'a> {
    pub user: &'a str,
    pub host: &'a str,
    pub port: u16,
}

/// Connection arguments and policy for system OpenSSH operations.
pub struct SshTransport {
    executable: PathBuf,
    connection_args: Vec<OsString>,
    user: String,
    host: String,
    port: u16,
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
        Self::connect_as_with_command(
            SshTarget { user, host, port },
            policy,
            SshCommand { executable: ssh_executable().into(), connection_args: Vec::new() },
        )
        .await
    }

    #[doc(hidden)]
    pub async fn connect_as_with_command(
        target: SshTarget<'_>,
        policy: TransportPolicy,
        command: SshCommand,
    ) -> Result<Self> {
        let transport = Self {
            executable: command.executable,
            connection_args: command.connection_args,
            user: target.user.into(),
            host: target.host.into(),
            port: target.port,
            policy,
        };
        transport.verify_connection().await?;
        Ok(transport)
    }

    pub async fn run_cmd(&self, cmd: &str) -> Result<String> {
        let deadline = Instant::now() + self.policy.command_deadline;
        let mut child = self.spawn(cmd, Stdio::null())?;
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
        let mut child = self.spawn(cmd, Stdio::null())?;
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
        let mut child = self.spawn(cmd, Stdio::piped())?;
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
        let mut child = self.spawn(cmd, if bytes.is_some() { Stdio::piped() } else { Stdio::null() })?;
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

    async fn verify_connection(&self) -> Result<()> {
        let deadline = Instant::now() + self.policy.connect_deadline;
        let mut child = self.spawn("true", Stdio::null())?;
        let stdout = take_stdout(&mut child)?;
        let stderr = take_stderr(&mut child)?;
        let result = timeout_at(deadline, async {
            tokio::try_join!(
                drain_stream(stdout, false, false, self.policy.remote_output_tail_limit),
                drain_stream(stderr, true, false, self.policy.remote_output_tail_limit)
            )
            .map_err(anyhow::Error::from)
        })
        .await;
        let (stdout, stderr) = match result {
            Ok(Ok(output)) => output,
            Ok(Err(error)) => {
                disconnect_child(&mut child).await;
                return Err(error).context("SSH transport failed while connecting");
            }
            Err(_) => {
                disconnect_child(&mut child).await;
                bail!("SSH connection timed out");
            }
        };
        let status = wait_for_child(child, deadline).await.with_context(|| {
            format!("SSH transport failed while connecting to {}@{}:{}", self.user, self.host, self.port)
        })?;
        ensure_success(status.success(), &stdout, &stderr).with_context(|| {
            format!("SSH transport failed while connecting to {}@{}:{}", self.user, self.host, self.port)
        })
    }
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};
    use std::path::PathBuf;

    use super::{SshCommand, SshTarget, SshTransport, TransportPolicy};

    #[tokio::test]
    async fn missing_openssh_names_the_windows_prerequisite_and_executable() -> Result<()> {
        let result = SshTransport::connect_as_with_command(
            SshTarget { user: "user", host: "host", port: 22 },
            TransportPolicy::default(),
            SshCommand { executable: PathBuf::from("missing/ssh.exe"), connection_args: Vec::new() },
        )
        .await;
        let error = result.err().context("missing OpenSSH should fail")?;
        let message = format!("{error:#}");
        assert!(message.contains("OpenSSH client"), "{message}");
        assert!(message.contains("ssh.exe"), "{message}");
        Ok(())
    }
}
