use std::io;

use anyhow::{Result, bail};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::process::ChildStdout;

#[derive(Debug)]
pub(super) struct OutputTail {
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

pub(super) async fn read_command_output(mut stream: ChildStdout, limit: usize) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    let mut buffer = [0; 8192];
    loop {
        let count = stream.read(&mut buffer).await?;
        if count == 0 {
            return Ok(bytes);
        }
        if bytes.len().saturating_add(count) > limit {
            return Err(io::Error::other("remote command output exceeded the configured limit"));
        }
        bytes.extend_from_slice(&buffer[..count]);
    }
}

pub(super) async fn copy_with_tail<W>(
    mut stream: ChildStdout,
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

pub(super) async fn drain_stream<R>(stream: R, stderr: bool, live: bool, limit: usize) -> io::Result<OutputTail>
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

pub(super) fn tail_from_bytes(bytes: &[u8], limit: usize) -> OutputTail {
    let mut tail = OutputTail::new(limit);
    tail.push(bytes);
    tail
}

pub(super) fn ensure_success(success: bool, stdout: &OutputTail, stderr: &OutputTail) -> Result<()> {
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
