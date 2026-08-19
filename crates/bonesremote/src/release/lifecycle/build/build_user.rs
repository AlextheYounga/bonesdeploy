use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

use crate::inspection::systemd;

pub struct BuildScriptEnv<'a> {
    pub project_name: &'a str,
    pub build_user: &'a str,
    pub build_group: &'a str,
    pub web_root: &'a str,
    pub deployment_dir: &'a Path,
    pub build_cache_dir: &'a Path,
    pub build_env_vars: &'a [(String, String)],
    /// Maximum seconds each build script may run before systemd terminates it.
    /// `None` disables the timeout.
    pub script_timeout_seconds: Option<u64>,
}

pub fn build_user_command(build_user: &str) -> Command {
    let mut command = Command::new("systemd-run");
    command.arg(format!("--machine={build_user}@")).args(["--quiet", "--user", "--collect", "--pipe", "--wait"]);
    command
}

/// A `build_user_command` that systemd terminates after `timeout_seconds`.
/// This bounds runaway build scripts without depending on Podman or the build
/// user's manager to cooperate: systemd enforces the deadline and kills the
/// script's whole process tree.
pub fn build_script_command(build_user: &str, timeout_seconds: u64) -> Command {
    let mut command = build_user_command(build_user);
    command.arg(format!("--property=RuntimeMaxSec={timeout_seconds}s"));
    command
}

pub fn build_user_control_command(build_user: &str) -> Command {
    let mut command = build_user_command(build_user);
    command.arg("--property=RuntimeMaxSec=20s");
    command
}

pub fn ensure_build_user_ready(build_user: &str, working_dir: &Path) -> Result<()> {
    let uid = identity_id(build_user, "-u", "UID")?;
    let gid = identity_id(build_user, "-g", "GID")?;
    validate_build_cache(&paths::bonesdeploy_user_cache(build_user), uid, gid)?;

    let status = Command::new("systemctl")
        .args(["start", &format!("user@{uid}.service")])
        .status()
        .with_context(|| format!("Failed to start the systemd user manager for {build_user}"))?;
    if !status.success() {
        bail!("Failed to start the systemd user manager for {build_user}: {status}");
    }

    if !systemd::active_status(&format!("user@{uid}.service"))? {
        bail!("The systemd user manager for {build_user} is not active");
    }

    let mut command = build_user_control_command(build_user);
    let status = command
        .current_dir(working_dir)
        .args(["podman", "info", "--format", "{{.Host.Security.Rootless}}"])
        .stdout(Stdio::null())
        .status()
        .with_context(|| format!("Failed to check rootless Podman for {build_user}"))?;
    if !status.success() {
        bail!(
            "Rootless Podman is not ready for {build_user}. Its user session or Podman namespace is unhealthy; repair it before deploying."
        );
    }

    Ok(())
}

pub fn identity_id(build_user: &str, flag: &str, label: &str) -> Result<u32> {
    let output = Command::new("id")
        .args([flag, build_user])
        .output()
        .with_context(|| format!("Failed to resolve build user {build_user}"))?;
    if !output.status.success() {
        bail!("Failed to resolve build {label} for {build_user}");
    }

    let value = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if value.is_empty() {
        bail!("Build user {build_user} has no {label}");
    }
    value.parse().with_context(|| format!("Build user {build_user} has an invalid {label}: {value}"))
}

pub fn validate_build_cache(path: &Path, uid: u32, gid: u32) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Build cache is missing: {}. Reapply BonesInfra.", path.display()))?;
    if !metadata.file_type().is_dir() {
        bail!("Build cache is not a directory: {}. Reapply BonesInfra.", path.display());
    }
    if metadata.uid() != uid || metadata.gid() != gid {
        bail!("Build cache has unsafe ownership: {}. Reapply BonesInfra.", path.display());
    }
    if metadata.permissions().mode() & 0o777 != 0o700 {
        bail!("Build cache must have mode 0700: {}. Reapply BonesInfra.", path.display());
    }
    Ok(())
}
