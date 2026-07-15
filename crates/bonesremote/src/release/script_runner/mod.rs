#[cfg(test)]
use std::env;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::os::unix::prelude::PermissionsExt;
#[cfg(test)]
use std::path::{Path, PathBuf};
#[cfg(test)]
use std::process::{Command, ExitStatus, Stdio};
#[cfg(test)]
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(test)]
use anyhow::{Context, Result};

mod build;
mod output;
mod prepare;

pub(crate) use build::{BuildContainer, BuildScriptEnv, build_user_command};
pub(crate) use prepare::{PrepareScriptEnv, run_prepare_script};

// ── test-only deployment runner ──────────────────────────────────────

#[cfg(test)]
pub(crate) struct HostScriptEnv<'a> {
    pub(crate) project_name: &'a str,
    pub(crate) project_root: &'a str,
    pub(crate) repo_path: &'a str,
    pub(crate) web_root: &'a str,
}

#[cfg(test)]
pub(crate) fn run_deployment_script(
    script: &Path,
    build_root: &Path,
    log_path: &Path,
    env: &HostScriptEnv<'_>,
) -> Result<ExitStatus> {
    let mut child = Command::new("bash")
        .arg("-c")
        .arg("umask 0002\nexec bash \"$@\"")
        .arg("bonesdeploy-umask")
        .arg(script)
        .current_dir(build_root)
        .env("PROJECT_NAME", env.project_name)
        .env("PROJECT_ROOT", env.project_root)
        .env("REPO_PATH", env.repo_path)
        .env("WEB_ROOT", env.web_root)
        .env("SERVICE_USER", env.project_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("Failed to execute deployment script {}", script.display()))?;

    output::stream_child_output(&mut child, log_path, &format!("deployment script {}", script.display()))
}

// ── tests ────────────────────────────────────────────────────────────

#[cfg(test)]
fn temp_dir(prefix: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("{prefix}_{nanos}"));
    fs::create_dir_all(&path)?;
    Ok(path)
}

#[cfg(test)]
fn write_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)?;
    Ok(())
}

#[cfg(test)]
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
    let status = run_deployment_script(
        &script,
        &build_root,
        &log_path,
        &HostScriptEnv {
            project_name: "demo",
            project_root: "/srv/deployments/demo",
            repo_path: "/home/git/demo.git",
            web_root: "public",
        },
    )?;

    assert!(status.success(), "passing script should exit zero");

    let log = fs::read_to_string(&log_path)?;
    assert!(log.contains("hello-stdout"), "log should contain stdout\n{log}");
    assert!(log.contains("hello-stderr"), "log should contain stderr\n{log}");

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[cfg(test)]
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
    let status = run_deployment_script(
        &script,
        &build_root,
        &log_path,
        &HostScriptEnv {
            project_name: "demo",
            project_root: "/srv/deployments/demo",
            repo_path: "/home/git/demo.git",
            web_root: "public",
        },
    )?;

    assert!(!status.success(), "failing script should exit non-zero");
    assert_eq!(status.code(), Some(7), "failing script should preserve exit code 7");
    let log = fs::read_to_string(&log_path)?;
    assert!(log.contains("about to fail"), "log should still be written for failing script\n{log}");

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[cfg(test)]
#[test]
fn run_deployment_script_applies_group_writable_umask() -> Result<()> {
    let root = temp_dir("bonesremote_deploy_runner_umask")?;
    let build_root = root.join("workspace");
    let logs = root.join("logs");
    fs::create_dir_all(&build_root)?;
    fs::create_dir_all(&logs)?;

    let out_file = build_root.join("umask_probe.txt");
    let script = root.join("00_probe.sh");
    write_file(&script, &format!("#!/usr/bin/env bash\necho hi > \"{}\"\n", out_file.display()))?;
    fs::set_permissions(&script, PermissionsExt::from_mode(0o755))?;

    let log_path = logs.join("20260612_211412-00_probe.sh.log");
    let status = run_deployment_script(
        &script,
        &build_root,
        &log_path,
        &HostScriptEnv {
            project_name: "demo",
            project_root: "/srv/deployments/demo",
            repo_path: "/home/git/demo.git",
            web_root: "public",
        },
    )?;

    assert!(status.success());
    let mode = fs::metadata(&out_file)?.permissions().mode() & 0o777;
    assert_eq!(mode, 0o664, "umask 0002 should make created files group-writable (0664), got {mode:o}");

    fs::remove_dir_all(root).ok();
    Ok(())
}
