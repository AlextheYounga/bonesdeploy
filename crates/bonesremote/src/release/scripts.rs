use std::fs;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{Context, Result, bail};
use shared::paths::{self, DeploymentPaths};

pub(super) fn deployment_log_path(paths: &DeploymentPaths, release_name: &str, script_name: &str) -> PathBuf {
    Path::new(&paths.build_logs).join(format!("{release_name}-{script_name}.log"))
}

pub(super) struct ScriptEnv<'a> {
    pub(super) project_name: &'a str,
    pub(super) project_root: &'a str,
    pub(super) repo_path: &'a str,
    pub(super) web_root: &'a str,
}

pub(super) struct DeploymentRun<'a, Out, Err> {
    script: &'a Path,
    build_root: &'a Path,
    log_path: &'a Path,
    env: ScriptEnv<'a>,
    consoles: ConsoleTargets<Out, Err>,
}

pub(super) fn run_deployment_script(
    script: &Path,
    build_root: &Path,
    log_path: &Path,
    env: &ScriptEnv<'_>,
) -> Result<ExitStatus> {
    run_deployment_script_with_consoles(DeploymentRun {
        script,
        build_root,
        log_path,
        env: ScriptEnv {
            project_name: env.project_name,
            project_root: env.project_root,
            repo_path: env.repo_path,
            web_root: env.web_root,
        },
        consoles: ConsoleTargets::system(),
    })
}

pub(super) fn run_deployment_script_with_consoles<Out, Err>(run: DeploymentRun<'_, Out, Err>) -> Result<ExitStatus>
where
    Out: Write + Send + 'static,
    Err: Write + Send + 'static,
{
    if let Some(parent) = run.log_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("Failed to create log directory {}", parent.display()))?;
    }

    let log_file = SharedWriter::new(
        fs::File::create(run.log_path)
            .with_context(|| format!("Failed to open deployment log {}", run.log_path.display()))?,
    );

    let mut child = Command::new("bash")
        .arg(run.script)
        .current_dir(run.build_root)
        .env("PROJECT_NAME", run.env.project_name)
        .env("PROJECT_ROOT", run.env.project_root)
        .env("REPO_PATH", run.env.repo_path)
        .env("WEB_ROOT", run.env.web_root)
        .env("SERVICE_USER", run.env.project_name)
        .env("DEPLOY_USER", paths::DEPLOY_USER)
        .env("GROUP", paths::DEFAULT_GROUP)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to execute deployment script {}", run.script.display()))?;

    let stdout = child.stdout.take().context("Failed to capture deployment script stdout")?;
    let stderr = child.stderr.take().context("Failed to capture deployment script stderr")?;

    let stdout_handle = spawn_stream(stdout, TeeWriter::new(run.consoles.stdout, log_file.clone()));
    let stderr_handle = spawn_stream(stderr, TeeWriter::new(run.consoles.stderr, log_file));

    let status =
        child.wait().with_context(|| format!("Failed to wait for deployment script {}", run.script.display()))?;

    join_stream(stdout_handle, "stdout")?;
    join_stream(stderr_handle, "stderr")?;

    Ok(status)
}

#[derive(Clone)]
pub(super) struct ConsoleTargets<Out, Err> {
    stdout: SharedWriter<Out>,
    stderr: SharedWriter<Err>,
}

impl<Out, Err> ConsoleTargets<Out, Err> {
    pub(super) fn new(stdout: Out, stderr: Err) -> Self {
        Self { stdout: SharedWriter::new(stdout), stderr: SharedWriter::new(stderr) }
    }
}

impl ConsoleTargets<io::Stdout, io::Stderr> {
    fn system() -> Self {
        Self::new(io::stdout(), io::stderr())
    }
}

#[cfg(test)]
impl<Out, Err> ConsoleTargets<Out, Err> {
    fn stdout_snapshot(&self) -> Arc<Mutex<Out>> {
        self.stdout.inner()
    }

    fn stderr_snapshot(&self) -> Arc<Mutex<Err>> {
        self.stderr.inner()
    }
}

struct SharedWriter<W> {
    inner: Arc<Mutex<W>>,
}

impl<W> Clone for SharedWriter<W> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner) }
    }
}

impl<W> SharedWriter<W> {
    fn new(writer: W) -> Self {
        Self { inner: Arc::new(Mutex::new(writer)) }
    }

    #[cfg(test)]
    fn inner(&self) -> Arc<Mutex<W>> {
        Arc::clone(&self.inner)
    }
}

impl<W: Write> Write for SharedWriter<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut writer = self.inner.lock().map_err(|_| io::Error::other("shared writer lock poisoned"))?;
        writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        let mut writer = self.inner.lock().map_err(|_| io::Error::other("shared writer lock poisoned"))?;
        writer.flush()
    }
}

struct TeeWriter<A, B> {
    primary: A,
    secondary: B,
}

impl<A, B> TeeWriter<A, B> {
    fn new(primary: A, secondary: B) -> Self {
        Self { primary, secondary }
    }
}

impl<A: Write, B: Write> Write for TeeWriter<A, B> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.primary.write_all(buf)?;
        self.secondary.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.primary.flush()?;
        self.secondary.flush()
    }
}

fn spawn_stream<R, W>(reader: R, writer: W) -> thread::JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
    W: Write + Send + 'static,
{
    thread::spawn(move || {
        let mut reader = reader;
        let mut writer = writer;
        let mut buffer = [0_u8; 8192];

        loop {
            let read = reader.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            writer.write_all(&buffer[..read])?;
        }

        writer.flush()?;
        Ok(())
    })
}

fn join_stream(handle: thread::JoinHandle<Result<()>>, stream_name: &str) -> Result<()> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => bail!("Deployment output thread for {stream_name} panicked"),
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io::{Cursor, Write};
    use std::path::{Path, PathBuf};
    use std::process;
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::Result;
    use std::os::unix::prelude::PermissionsExt;

    use shared::paths::DeploymentPaths;

    use super::{
        ConsoleTargets, DeploymentRun, ScriptEnv, TeeWriter, deployment_log_path, run_deployment_script_with_consoles,
    };

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    fn write_file(path: &Path, content: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)?;
        Ok(())
    }

    /// Duplicates every byte into both writers.
    #[test]
    fn tee_writer_writes_to_both_targets() -> Result<()> {
        let stdout = Cursor::new(Vec::new());
        let log = Cursor::new(Vec::new());
        let mut writer = TeeWriter::new(stdout, log);

        writer.write_all(b"hello\nworld")?;
        writer.flush()?;

        let TeeWriter { primary: stdout, secondary: log } = writer;
        assert_eq!(stdout.into_inner(), b"hello\nworld");
        assert_eq!(log.into_inner(), b"hello\nworld");
        Ok(())
    }

    /// Streams deployment output into both console targets and the log file.
    #[test]
    fn run_deployment_script_streams_output_to_console_and_log() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_runner_stream")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let script = root.join("00_hello.sh");
        write_file(&script, "#!/usr/bin/env bash\necho 'hello-stdout'\necho 'hello-stderr' >&2\n")?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let log_path = logs.join("20260612_211412-00_hello.sh.log");
        let consoles = ConsoleTargets::new(Cursor::new(Vec::new()), Cursor::new(Vec::new()));
        let status = run_deployment_script_with_consoles(DeploymentRun {
            script: &script,
            build_root: &build_root,
            log_path: &log_path,
            env: ScriptEnv {
                project_name: "demo",
                project_root: "/srv/deployments/demo",
                repo_path: "/home/git/demo.git",
                web_root: "public",
            },
            consoles: consoles.clone(),
        })?;

        assert!(status.success(), "passing script should exit zero");

        let stdout = consoles.stdout_snapshot();
        let stderr = consoles.stderr_snapshot();

        let stdout = stdout.lock().map_err(|_| anyhow::anyhow!("stdout writer lock poisoned"))?;
        let stderr = stderr.lock().map_err(|_| anyhow::anyhow!("stderr writer lock poisoned"))?;
        assert_eq!(stdout.get_ref(), b"hello-stdout\n");
        assert_eq!(stderr.get_ref(), b"hello-stderr\n");

        let log = fs::read_to_string(&log_path)?;
        assert!(log.contains("hello-stdout"), "log should contain stdout\n{log}");
        assert!(log.contains("hello-stderr"), "log should contain stderr\n{log}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    /// Preserves the failing script's exit status after tee-ing output to the log file.
    #[test]
    fn run_deployment_script_preserves_failing_exit_status() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_runner_failing")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let script = root.join("01_install.sh");
        write_file(&script, "#!/usr/bin/env bash\necho 'about to fail' >&2\nexit 7\n")?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let log_path = logs.join("20260612_211412-01_install.sh.log");
        let consoles = ConsoleTargets::new(Cursor::new(Vec::new()), Cursor::new(Vec::new()));
        let status = run_deployment_script_with_consoles(DeploymentRun {
            script: &script,
            build_root: &build_root,
            log_path: &log_path,
            env: ScriptEnv {
                project_name: "demo",
                project_root: "/srv/deployments/demo",
                repo_path: "/home/git/demo.git",
                web_root: "public",
            },
            consoles,
        })?;

        assert!(!status.success(), "failing script should exit non-zero");
        assert_eq!(status.code(), Some(7), "failing script should preserve exit code 7");
        let log = fs::read_to_string(&log_path)?;
        assert!(log.contains("about to fail"), "log should still be written for failing script\n{log}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    /// Creates the log directory on demand so the deploy runner can write into a fresh project root.
    #[test]
    fn run_deployment_script_creates_missing_log_directory() -> Result<()> {
        let root = temp_dir("bonesremote_deploy_runner_mkdir")?;
        let build_root = root.join("workspace");
        fs::create_dir_all(&build_root)?;

        let script = root.join("00_pass.sh");
        write_file(&script, "#!/usr/bin/env bash\necho ok\n")?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let log_path = root.join("build/logs/20260612_211412-00_pass.sh.log");
        let consoles = ConsoleTargets::new(Cursor::new(Vec::new()), Cursor::new(Vec::new()));
        let status = run_deployment_script_with_consoles(DeploymentRun {
            script: &script,
            build_root: &build_root,
            log_path: &log_path,
            env: ScriptEnv {
                project_name: "demo",
                project_root: "/srv/deployments/demo",
                repo_path: "/home/git/demo.git",
                web_root: "public",
            },
            consoles,
        })?;

        assert!(status.success());
        assert!(log_path.exists(), "log file should be created even when its directory is missing");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    /// Builds the log path under the centralized `project_root/build/logs` directory.
    #[test]
    fn deployment_log_path_lives_under_build_logs() {
        let paths = DeploymentPaths::new("demo", "/home/git/demo.git", "/srv/deployments/demo", "public");
        let log = deployment_log_path(&paths, "20260612_211412", "02_run_build.sh");

        assert_eq!(
            log,
            PathBuf::from("/srv/deployments/demo/build/logs/20260612_211412-02_run_build.sh.log"),
            "log path should derive from centralized build_logs directory"
        );
    }
}
