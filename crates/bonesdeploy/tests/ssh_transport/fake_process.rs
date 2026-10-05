use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
#[cfg(unix)]
use std::process::{Command, Stdio};
#[cfg(unix)]
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
#[cfg(unix)]
use bonesdeploy::infra::ssh::shell_quote;
use bonesdeploy::infra::ssh::{SshCommand, SshTarget, SshTransport, TransportPolicy};
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
use crate::expected_error;
use crate::short_policy;

struct FakeSshFixture {
    _directory: TempDir,
    executable: PathBuf,
    connection_args: Vec<OsString>,
    log: PathBuf,
    #[cfg(unix)]
    pid: PathBuf,
}

impl FakeSshFixture {
    fn new() -> Result<Self> {
        let directory = tempfile::tempdir().context("failed to create fake SSH directory")?;
        let log = directory.path().join("argv.log");
        #[cfg(unix)]
        let pid = directory.path().join("child.pid");
        #[cfg(unix)]
        let (executable, connection_args, script) = {
            let executable = directory.path().join("fake-ssh");
            let script = format!(
                "#!/bin/sh\n{{ for arg do printf '%s\\n' \"$arg\"; done; printf '%s\\n' --END--; }} >> {}\nprintf '%s\\n' \"$$\" > {}\nlast=\nfor arg do last=\"$arg\"; done\neval \"$last\"\n",
                shell_quote(&log.to_string_lossy()),
                shell_quote(&pid.to_string_lossy()),
            );
            (executable, Vec::new(), script)
        };
        #[cfg(windows)]
        let (executable, connection_args, script) = {
            let script_path = directory.path().join("fake-ssh.cmd");
            let script = format!("@echo off\n>>\"{}\" echo %*\nset /p \"=fake-windows\" <nul\n", log.display());
            (PathBuf::from("cmd.exe"), vec!["/C".into(), script_path.into_os_string()], script)
        };
        fs::write(if cfg!(windows) { directory.path().join("fake-ssh.cmd") } else { executable.clone() }, script)?;
        #[cfg(unix)]
        fs::set_permissions(&executable, PermissionsExt::from_mode(0o700))?;
        Ok(Self {
            _directory: directory,
            executable,
            connection_args,
            log,
            #[cfg(unix)]
            pid,
        })
    }

    async fn transport(&self, policy: TransportPolicy) -> Result<SshTransport> {
        SshTransport::connect_as_with_command(
            SshTarget { user: "alice", host: "example.test", port: 2222 },
            policy,
            SshCommand { executable: self.executable.clone(), connection_args: self.connection_args.clone() },
        )
        .await
    }
}

#[cfg(unix)]
#[tokio::test]
async fn fake_ssh_asserts_exact_argv_and_remote_command() -> Result<()> {
    let fixture = FakeSshFixture::new()?;
    let transport = fixture.transport(short_policy()).await?;
    assert_eq!(transport.run_cmd("printf '%s' \"a b; c\"").await?, "a b; c");

    let log = fs::read_to_string(&fixture.log)?;
    let invocations = log
        .split("--END--\n")
        .filter(|invocation| !invocation.is_empty())
        .map(|invocation| invocation.lines().collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(invocations.len(), 2);
    assert_eq!(invocations[0], ["-p", "2222", "--", "alice@example.test", "exec bash -c 'true'"]);
    assert_eq!(
        invocations[1],
        ["-p", "2222", "--", "alice@example.test", r#"exec bash -c 'printf '\''%s'\'' "a b; c"'"#]
    );
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn fake_ssh_covers_stdin_upload_download_failures_limits_timeout_cleanup_and_reuse() -> Result<()> {
    let fixture = FakeSshFixture::new()?;
    let transport = fixture.transport(short_policy()).await?;
    assert_eq!(transport.run_cmd_with_stdin_output("cat", b"stdin").await?, "stdin");
    transport.stream_cmd_with_stdin("cat >/dev/null", b"upload").await?;
    let mut downloaded = Vec::new();
    transport.download_cmd("printf download", &mut downloaded).await?;
    assert_eq!(downloaded, b"download");

    let mut limited_policy = short_policy();
    limited_policy.command_output_limit = 4;
    let limited_transport = fixture.transport(limited_policy).await?;
    let error = expected_error(limited_transport.run_cmd("printf 12345").await, "output limit should fail")?;
    assert!(format!("{error:#}").contains("output exceeded the configured limit"));
    let error = expected_error(transport.run_cmd("printf failure >&2; exit 7").await, "failure should propagate")?;
    assert!(format!("{error:#}").contains("failure"));

    let started = Instant::now();
    let error = expected_error(transport.stream_cmd("exec sleep 10").await, "timeout should fail")?;
    assert!(error.to_string().contains("timed out"));
    assert!(started.elapsed() < Duration::from_secs(2));
    let pid: i32 = fs::read_to_string(&fixture.pid)?.trim().parse()?;
    assert!(!Command::new("kill").args(["-0", &pid.to_string()]).stderr(Stdio::null()).status()?.success());
    assert_eq!(transport.run_cmd("printf reused").await?, "reused");
    Ok(())
}

#[cfg(windows)]
#[tokio::test]
async fn fake_ssh_runs_through_native_cmd_on_windows() -> Result<()> {
    let fixture = FakeSshFixture::new()?;
    let transport = fixture.transport(short_policy()).await?;
    assert_eq!(transport.run_cmd("ignored").await?, "fake-windows");
    assert!(!fs::read_to_string(&fixture.log)?.is_empty());
    Ok(())
}
