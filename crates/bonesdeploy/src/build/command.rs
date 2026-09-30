use std::collections::VecDeque;
use std::error::Error;
use std::fmt::{self, Display, Formatter};
use std::io::{self, Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};

const DIAGNOSTIC_TAIL_BYTES: usize = 64 * 1024;

pub(crate) struct CommandOutput {
    pub stdout: Vec<u8>,
}

#[derive(Debug)]
pub(crate) struct TimedOut {
    action: String,
    seconds: u64,
    diagnostics: Diagnostics,
}

impl Display for TimedOut {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} exceeded its {}-second timeout{}", self.action, self.seconds, self.diagnostics)
    }
}

impl Error for TimedOut {}

#[derive(Debug)]
struct CommandFailed {
    action: String,
    status: ExitStatus,
    diagnostics: Diagnostics,
}

impl Display for CommandFailed {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(formatter, "Failed to {}: {}{}", self.action, self.status, self.diagnostics)
    }
}

impl Error for CommandFailed {}

#[derive(Debug, Default)]
struct Diagnostics {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

impl Display for Diagnostics {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        if self.stdout.is_empty() && self.stderr.is_empty() {
            return Ok(());
        }
        write!(
            formatter,
            "\nstdout tail:\n{}\nstderr tail:\n{}",
            display_bytes(&self.stdout),
            display_bytes(&self.stderr)
        )
    }
}

/// Runs a local Docker-backed build operation, streaming output and retaining
/// bounded diagnostics for a failure.
pub(crate) fn run(command: Command, action: &str, timeout: Option<u64>) -> Result<()> {
    execute(command, action, timeout, true).map(|_| ())
}

/// Runs a local Docker-backed build operation whose stdout is required by its caller.
pub(crate) fn output(command: Command, action: &str, timeout: Option<u64>) -> Result<CommandOutput> {
    execute(command, action, timeout, false)
}

pub(crate) fn timed_out(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| cause.downcast_ref::<TimedOut>().is_some())
}

pub(crate) fn failed(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| cause.downcast_ref::<CommandFailed>().is_some())
}

fn execute(mut command: Command, action: &str, timeout: Option<u64>, echo_output: bool) -> Result<CommandOutput> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().with_context(|| format!("Failed to {action}"))?;
    let stdout = child.stdout.take().context("Build command stdout was not piped")?;
    let stderr = child.stderr.take().context("Build command stderr was not piped")?;
    let stdout_reader = thread::spawn(move || drain(stdout, echo_output, false));
    let stderr_reader = thread::spawn(move || drain(stderr, echo_output, true));
    let started = Instant::now();

    let status = loop {
        if let Some(status) = child.try_wait().with_context(|| format!("Failed to wait for {action}"))? {
            break status;
        }
        if let Some(seconds) = timeout.filter(|seconds| started.elapsed() >= Duration::from_secs(*seconds)) {
            let kill_error = child.kill().err();
            let status = child.wait().with_context(|| format!("Failed to wait for timed out {action}"))?;
            let (_, stdout_tail) = finish_reader(stdout_reader, "stdout")?;
            let (_, stderr_tail) = finish_reader(stderr_reader, "stderr")?;
            if let Some(error) = kill_error.filter(|error| error.kind() != io::ErrorKind::InvalidInput) {
                return Err(error).with_context(|| format!("Failed to stop timed out {action}"));
            }
            return Err(anyhow::Error::new(TimedOut {
                action: action.into(),
                seconds,
                diagnostics: Diagnostics { stdout: stdout_tail, stderr: stderr_tail },
            })
            .context(format!("{action} terminated with status {status}")));
        }
        thread::sleep(Duration::from_millis(25));
    };

    let (stdout, stdout_tail) = finish_reader(stdout_reader, "stdout")?;
    let (_, stderr_tail) = finish_reader(stderr_reader, "stderr")?;
    let diagnostics = Diagnostics { stdout: stdout_tail, stderr: stderr_tail };
    if !status.success() {
        return Err(anyhow::Error::new(CommandFailed { action: action.into(), status, diagnostics }));
    }
    Ok(CommandOutput { stdout })
}

fn drain<R: Read>(mut reader: R, echo_output: bool, stderr: bool) -> Result<(Vec<u8>, Vec<u8>)> {
    let mut output = Vec::new();
    let mut tail = VecDeque::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = reader.read(&mut buffer).context("Failed to read build command output")?;
        if read == 0 {
            break;
        }
        let chunk = &buffer[..read];
        if echo_output {
            if stderr {
                io::stderr().lock().write_all(chunk).context("Failed to display build command stderr")?;
            } else {
                io::stdout().lock().write_all(chunk).context("Failed to display build command stdout")?;
            }
        }
        if !stderr {
            output.extend_from_slice(chunk);
        }
        append_tail(&mut tail, chunk);
    }
    Ok((output, tail.into()))
}

fn append_tail(tail: &mut VecDeque<u8>, bytes: &[u8]) {
    let start = bytes.len().saturating_sub(DIAGNOSTIC_TAIL_BYTES);
    let bytes = &bytes[start..];
    let excess = tail.len().saturating_add(bytes.len()).saturating_sub(DIAGNOSTIC_TAIL_BYTES);
    tail.drain(..excess);
    tail.extend(bytes);
}

fn finish_reader(reader: thread::JoinHandle<Result<(Vec<u8>, Vec<u8>)>>, stream: &str) -> Result<(Vec<u8>, Vec<u8>)> {
    reader.join().map_err(|_| anyhow::anyhow!("Build command {stream} reader panicked"))?
}

fn display_bytes(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}
