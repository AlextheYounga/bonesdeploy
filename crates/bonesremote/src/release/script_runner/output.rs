use std::fs;
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::{Child, ExitStatus};
use std::thread;

use anyhow::{Context, Result, bail};

/// Streams a child process's stdout and stderr to the terminal *and* to a log file
/// concurrently. Blocks until the child exits, then returns its exit status.
pub(crate) fn stream_child_output(child: &mut Child, log_path: &Path, label: &str) -> Result<ExitStatus> {
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create log directory {}", parent.display()))?;
    }

    let log_file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("Failed to open deployment log {}", log_path.display()))?;

    let stdout = child.stdout.take().context("Failed to capture deployment stdout")?;
    let stderr = child.stderr.take().context("Failed to capture deployment stderr")?;

    let stdout_handle =
        spawn_stream(stdout, io::stdout(), log_file.try_clone().context("Failed to clone deployment log")?);
    let stderr_handle = spawn_stream(stderr, io::stderr(), log_file);

    let status = child.wait().with_context(|| format!("Failed to wait for {label}"))?;

    join_stream(stdout_handle, "stdout")?;
    join_stream(stderr_handle, "stderr")?;

    Ok(status)
}

/// Copies `reader` to `primary` and `secondary` simultaneously on a background
/// thread. Reads in 8 KiB chunks until EOF. Returns a join handle so the caller
/// can wait for completion.
fn spawn_stream<R, W1, W2>(reader: R, primary: W1, secondary: W2) -> thread::JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
    W1: Write + Send + 'static,
    W2: Write + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = reader;
        let mut primary = primary;
        let mut secondary = secondary;
        let mut buffer = [0_u8; 8192];

        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            primary.write_all(&buffer[..read])?;
            secondary.write_all(&buffer[..read])?;
        }

        primary.flush()?;
        secondary.flush()?;
        Ok(())
    })
}

/// Joins a `spawn_stream` thread handle, propagating any I/O error or panic.
fn join_stream(handle: thread::JoinHandle<Result<()>>, stream_name: &str) -> Result<()> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => bail!("Deployment output thread for {stream_name} panicked"),
    }
}
