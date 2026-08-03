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
/// thread. Reads in 8 KiB chunks until EOF.
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
    use std::os::unix::prelude::PermissionsExt;
    use std::path::{Path, PathBuf};
    use std::process::{Command, ExitStatus, Stdio};
    use std::time::{SystemTime, UNIX_EPOCH};

    use anyhow::{Context, Result};

    use super::stream_child_output;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{nanos}"));
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

    fn run_script(script: &Path, build_root: &Path, log_path: &Path) -> Result<ExitStatus> {
        let mut child = Command::new("bash")
            .arg("-c")
            .arg("umask 0002\nexec bash \"$@\"")
            .arg("bonesdeploy-umask")
            .arg(script)
            .current_dir(build_root)
            .env("PROJECT_NAME", "demo")
            .env("PROJECT_ROOT", "/srv/deployments/demo")
            .env("REPO_PATH", "/home/git/demo.git")
            .env("WEB_ROOT", "public")
            .env("SERVICE_USER", "demo")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .with_context(|| format!("Failed to execute script {}", script.display()))?;
        stream_child_output(&mut child, log_path, &format!("script {}", script.display()))
    }

    #[test]
    fn streams_output_to_console_and_log() -> Result<()> {
        let root = temp_dir("bonesremote_output_stream")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let script = root.join("00_hello.sh");
        write_file(&script, "#!/usr/bin/env bash\necho 'hello-stdout'\necho 'hello-stderr' >&2\n")?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let status = run_script(&script, &build_root, &logs.join("00_hello.sh.log"))?;

        assert!(status.success(), "passing script should exit zero");
        let log = fs::read_to_string(logs.join("00_hello.sh.log"))?;
        assert!(log.contains("hello-stdout"), "log should contain stdout\n{log}");
        assert!(log.contains("hello-stderr"), "log should contain stderr\n{log}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn preserves_failing_exit_status() -> Result<()> {
        let root = temp_dir("bonesremote_output_failing")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let script = root.join("01_install.sh");
        write_file(&script, "#!/usr/bin/env bash\necho 'about to fail' >&2\nexit 7\n")?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let status = run_script(&script, &build_root, &logs.join("01_install.sh.log"))?;

        assert!(!status.success(), "failing script should exit non-zero");
        assert_eq!(status.code(), Some(7), "failing script should preserve exit code 7");
        let log = fs::read_to_string(logs.join("01_install.sh.log"))?;
        assert!(log.contains("about to fail"), "log should still be written for failing script\n{log}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }

    #[test]
    fn applies_group_writable_umask() -> Result<()> {
        let root = temp_dir("bonesremote_output_umask")?;
        let build_root = root.join("workspace");
        let logs = root.join("logs");
        fs::create_dir_all(&build_root)?;
        fs::create_dir_all(&logs)?;

        let out_file = build_root.join("umask_probe.txt");
        let script = root.join("00_probe.sh");
        write_file(&script, &format!("#!/usr/bin/env bash\necho hi > \"{}\"\n", out_file.display()))?;
        fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

        let status = run_script(&script, &build_root, &logs.join("00_probe.sh.log"))?;

        assert!(status.success());
        let mode = fs::metadata(&out_file)?.permissions().mode() & 0o777;
        assert_eq!(mode, 0o664, "umask 0002 should make created files group-writable (0664), got {mode:o}");

        fs::remove_dir_all(root).ok();
        Ok(())
    }
}
