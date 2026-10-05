use std::io;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use tokio::io::AsyncWriteExt;
use tokio::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::time::{Instant, timeout, timeout_at};

use super::{SshTransport, shell_quote};

const CHILD_CLEANUP_DEADLINE: Duration = Duration::from_secs(5);

impl SshTransport {
    pub(super) fn spawn(&self, cmd: &str, stdin: Stdio) -> Result<Child> {
        let mut command = Command::new(&self.executable);
        command
            .args(&self.connection_args)
            .arg("-p")
            .arg(self.port.to_string())
            .arg("--")
            .arg(format!("{}@{}", self.user, self.host))
            .arg(remote_command(cmd))
            .stdin(stdin)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        command.spawn().with_context(|| {
            format!(
                "Failed to start {}; install the OpenSSH client and ensure it is on PATH",
                self.executable.display()
            )
        })
    }
}

pub(super) fn ssh_executable() -> &'static str {
    if cfg!(windows) { "ssh.exe" } else { "ssh" }
}

fn remote_command(command: &str) -> String {
    format!("exec bash -c {}", shell_quote(command))
}

pub(super) fn take_stdin(child: &mut Child) -> Result<ChildStdin> {
    child.stdin.take().ok_or_else(|| anyhow!("SSH transport failed: stdin was not piped"))
}

pub(super) async fn wait_for_child(mut child: Child, deadline: Instant) -> Result<ExitStatus> {
    match timeout_at(deadline, child.wait()).await {
        Ok(result) => result.context("SSH transport failed while waiting for remote command"),
        Err(_) => {
            disconnect_child(&mut child).await;
            bail!("SSH operation timed out")
        }
    }
}

pub(super) fn take_stdout(child: &mut Child) -> Result<ChildStdout> {
    child.stdout.take().ok_or_else(|| anyhow!("SSH transport failed: stdout was not piped"))
}

pub(super) fn take_stderr(child: &mut Child) -> Result<ChildStderr> {
    child.stderr.take().ok_or_else(|| anyhow!("SSH transport failed: stderr was not piped"))
}

pub(super) async fn write_bytes(mut stdin: ChildStdin, bytes: &[u8]) -> io::Result<()> {
    stdin.write_all(bytes).await?;
    stdin.shutdown().await
}

pub(super) async fn disconnect_child(child: &mut Child) {
    let _ = child.start_kill();
    let _ = timeout(CHILD_CLEANUP_DEADLINE, child.wait()).await;
}
