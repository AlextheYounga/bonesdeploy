use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Error, ErrorKind};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::pin::Pin;
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;
use std::task::{Context as TaskContext, Poll};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context, Result, bail};
use bonesdeploy::infra::ssh::{SshCommand, SshTarget, SshTransport, TransportPolicy};
use tempfile::TempDir;
use tokio::io::{AsyncRead, ReadBuf, repeat};
use tokio::sync::{Mutex, MutexGuard};

use crate::{expected_error, short_policy};

struct SshdFixture {
    _directory: TempDir,
    port: u16,
    private_key: PathBuf,
    process: Child,
}

impl SshdFixture {
    fn start() -> Result<Option<Self>> {
        if !tool_available("sshd") || !tool_available("ssh-keygen") || !tool_available("ssh") {
            eprintln!("skipping local-sshd SSH transport smoke test: OpenSSH tools unavailable");
            return Ok(None);
        }
        let directory = tempfile::tempdir().context("failed to create sshd fixture directory")?;
        let private_key = directory.path().join("id_ed25519");
        run("ssh-keygen", ["-q", "-t", "ed25519", "-N", "", "-f"], Some(&private_key))?;
        let public_key =
            fs::read_to_string(private_key.with_extension("pub")).context("failed to read fixture public key")?;
        let authorized_keys = directory.path().join("authorized_keys");
        fs::write(&authorized_keys, public_key).context("failed to write fixture authorized keys")?;
        let host_key = directory.path().join("host_ed25519");
        run("ssh-keygen", ["-q", "-t", "ed25519", "-N", "", "-f"], Some(&host_key))?;
        let port = free_port()?;
        let config = directory.path().join("sshd_config");
        fs::write(
            &config,
            format!(
                "Port {port}\nListenAddress 127.0.0.1\nHostKey {}\nAuthorizedKeysFile {}\nPidFile {}\nPasswordAuthentication no\nChallengeResponseAuthentication no\nUsePAM no\nStrictModes no\nLogLevel ERROR\n",
                host_key.display(),
                authorized_keys.display(),
                directory.path().join("sshd.pid").display(),
            ),
        )?;
        let sshd = ["/usr/sbin/sshd", "/usr/bin/sshd"]
            .into_iter()
            .map(PathBuf::from)
            .find(|path| path.is_file())
            .ok_or_else(|| anyhow::anyhow!("sshd executable disappeared"))?;
        let mut process = Command::new(sshd)
            .args(["-D", "-e", "-f"])
            .arg(&config)
            .stderr(Stdio::inherit())
            .spawn()
            .context("failed to start fixture sshd")?;
        if wait_for_port(port).is_err() {
            let _ = process.kill();
            let _ = process.wait();
            eprintln!("skipping local-sshd SSH transport smoke test: sshd did not start");
            return Ok(None);
        }
        Ok(Some(Self { _directory: directory, port, private_key, process }))
    }

    async fn transport(&self) -> Result<SshTransport> {
        let mut policy = TransportPolicy::default();
        policy.connect_deadline = Duration::from_secs(2);
        self.transport_with_policy(policy).await
    }

    async fn transport_with_policy(&self, policy: TransportPolicy) -> Result<SshTransport> {
        let user = username()?;
        SshTransport::connect_as_with_command(
            SshTarget { user: &user, host: "127.0.0.1", port: self.port },
            policy,
            SshCommand {
                executable: PathBuf::from("/usr/bin/ssh"),
                connection_args: vec![
                    "-i".into(),
                    self.private_key.clone().into_os_string(),
                    "-o".into(),
                    "BatchMode=yes".into(),
                    "-o".into(),
                    "StrictHostKeyChecking=no".into(),
                    "-o".into(),
                    "UserKnownHostsFile=/dev/null".into(),
                    "-o".into(),
                    "LogLevel=ERROR".into(),
                ],
            },
        )
        .await
        .context("failed to connect to fixture sshd")
    }
}

impl Drop for SshdFixture {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
    }
}

#[tokio::test]
async fn connect_timeout_is_bounded_when_the_server_never_completes_handshake() -> Result<()> {
    let _lock = fixture_lock().await;
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let port = listener.local_addr()?.port();
    let blocker = thread::spawn(move || {
        let _connection = listener.accept();
        thread::sleep(Duration::from_secs(2));
    });
    let start = Instant::now();
    let error = expected_error(
        SshTransport::connect_as_with_policy(&username()?, "127.0.0.1", port, short_policy()).await,
        "connection should time out",
    )?;
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(error.to_string().contains("timed out"));
    blocker.join().map_err(|_| anyhow::anyhow!("blocker thread should finish"))?;
    Ok(())
}

#[tokio::test]
async fn command_output_has_an_exact_limit_and_non_streaming_stderr_is_not_a_live_output_path() -> Result<()> {
    let _lock = fixture_lock().await;
    let Some(fixture) = SshdFixture::start()? else { return Ok(()) };
    let transport = fixture.transport().await?;
    let output = transport.run_cmd("head -c 65536 /dev/zero | tr '\\0' x").await?;
    assert_eq!(output.len(), 64 * 1024);

    let error = expected_error(
        transport.run_cmd("head -c 65537 /dev/zero | tr '\\0' x").await,
        "oversized command output should fail",
    )?;
    assert!(format!("{error:#}").contains("output exceeded the configured limit"));
    Ok(())
}

#[tokio::test]
async fn abrupt_remote_channel_closure_is_a_transport_failure() -> Result<()> {
    let _lock = fixture_lock().await;
    let Some(fixture) = SshdFixture::start()? else { return Ok(()) };
    let transport = fixture.transport().await?;
    let error = expected_error(transport.stream_cmd("kill -9 $$").await, "abrupt closure should fail")?;
    assert!(format!("{error:#}").contains("SSH remote command exited unsuccessfully"));
    Ok(())
}

#[tokio::test]
async fn command_and_transfer_timeouts_disconnect_hanging_remote_channels() -> Result<()> {
    let _lock = fixture_lock().await;
    let Some(fixture) = SshdFixture::start()? else { return Ok(()) };
    let mut timeout_policy = short_policy();
    timeout_policy.connect_deadline = TransportPolicy::default().connect_deadline;
    let transport = fixture.transport_with_policy(timeout_policy).await?;
    let start = Instant::now();
    let error = expected_error(transport.stream_cmd("sleep 10").await, "command should time out")?;
    assert!(start.elapsed() < Duration::from_secs(1));
    assert!(error.to_string().contains("timed out"));
    let transport = fixture.transport().await?;
    assert_eq!(transport.run_cmd("printf reused").await?, "reused");

    let transport = fixture.transport_with_policy(timeout_policy).await?;
    let error = expected_error(
        transport.stream_cmd("exec >/dev/null 2>&1; sleep 10").await,
        "command with closed output should time out",
    )?;
    assert!(error.to_string().contains("timed out"));

    let transport = fixture.transport_with_policy(timeout_policy).await?;
    let error =
        expected_error(transport.stream_cmd_with_stdin("sleep 10", b"input").await, "stdin command should time out")?;
    assert!(error.to_string().contains("timed out"));

    let transport = fixture.transport_with_policy(timeout_policy).await?;
    let mut destination = Vec::new();
    let error = expected_error(transport.download_cmd("sleep 10", &mut destination).await, "download should time out")?;
    assert!(error.to_string().contains("timed out"));

    let transport = fixture.transport_with_policy(timeout_policy).await?;
    let error = expected_error(
        transport.stream_cmd_with_reader("sleep 10", b"manifest", repeat(0)).await,
        "artifact upload should time out",
    )?;
    assert!(error.to_string().contains("timed out"));
    Ok(())
}

#[tokio::test]
async fn remote_exit_retains_bounded_remote_tails_without_stdin_contents() -> Result<()> {
    let _lock = fixture_lock().await;
    let Some(fixture) = SshdFixture::start()? else { return Ok(()) };
    let transport = fixture.transport().await?;
    let secret = b"stdin-secret-must-not-appear";
    let command = "printf stdout; head -c 70000 /dev/zero | tr '\\0' e >&2; exit 7";
    let error = expected_error(transport.run_cmd_with_stdin(command, secret).await, "remote exit should fail")?;
    let message = format!("{error:#}");
    assert!(message.contains("SSH remote command exited unsuccessfully"));
    assert!(message.contains("stdout tail"));
    assert!(message.contains("stderr tail (truncated)"));
    assert!(!message.contains("stdin-secret-must-not-appear"));

    let transport = fixture.transport().await?;
    let error = expected_error(
        transport.stream_cmd_with_reader("cat >/dev/null; exit 2", b"", &b"artifact-secret-must-not-appear"[..]).await,
        "remote exit should fail",
    )?;
    assert!(!format!("{error:#}").contains("artifact-secret-must-not-appear"));
    Ok(())
}

#[tokio::test]
async fn local_reader_failures_are_returned_as_transport_failures() -> Result<()> {
    let _lock = fixture_lock().await;
    let Some(fixture) = SshdFixture::start()? else { return Ok(()) };
    let transport = fixture.transport().await?;
    let error = expected_error(
        transport.stream_cmd_with_reader("cat >/dev/null", b"manifest", FailingReader).await,
        "reader failure should fail the transport operation",
    )?;
    assert!(format!("{error:#}").contains("SSH transport failed"));
    Ok(())
}

struct FailingReader;

impl AsyncRead for FailingReader {
    fn poll_read(self: Pin<&mut Self>, _: &mut TaskContext<'_>, _: &mut ReadBuf<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Err(Error::new(ErrorKind::BrokenPipe, "fixture reader failed")))
    }
}

#[tokio::test]
async fn streaming_drains_stdout_and_stderr_before_reporting_remote_exit() -> Result<()> {
    let _lock = fixture_lock().await;
    let Some(fixture) = SshdFixture::start()? else { return Ok(()) };
    let transport = fixture.transport().await?;
    let error = expected_error(
        transport.stream_cmd("printf stdout; printf stderr >&2; exit 3").await,
        "remote exit should fail",
    )?;
    let message = format!("{error:#}");
    assert!(message.contains("stdout"));
    assert!(message.contains("stderr"));
    Ok(())
}

#[tokio::test]
async fn remote_command_preserves_shell_metacharacters() -> Result<()> {
    let _lock = fixture_lock().await;
    let Some(fixture) = SshdFixture::start()? else { return Ok(()) };
    let transport = fixture.transport().await?;

    assert_eq!(
        transport.run_cmd(r#"printf '%s' "spaces ' quotes \$dollars; semicolon""#).await?,
        "spaces ' quotes $dollars; semicolon"
    );
    Ok(())
}

fn run<I, S>(program: &str, args: I, path: Option<&Path>) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(program);
    command.args(args);
    if let Some(path) = path {
        command.arg(path);
    }
    let status = command.status().with_context(|| format!("failed to run {program}"))?;
    if !status.success() {
        bail!("{program} failed with status {status}");
    }
    Ok(())
}

fn tool_available(program: &str) -> bool {
    Command::new(program).arg("-V").output().is_ok()
}

fn free_port() -> Result<u16> {
    Ok(TcpListener::bind("127.0.0.1:0")?.local_addr()?.port())
}

fn wait_for_port(port: u16) -> Result<()> {
    for _ in 0..20 {
        if TcpStream::connect(("127.0.0.1", port)).is_ok() {
            return Ok(());
        }
        thread::sleep(Duration::from_millis(20));
    }
    bail!("fixture sshd did not start")
}

fn username() -> Result<String> {
    env::var("USER").context("USER must be set for the sshd fixture")
}

async fn fixture_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().await
}
