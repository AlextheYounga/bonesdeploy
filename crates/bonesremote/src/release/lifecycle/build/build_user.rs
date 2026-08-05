use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context, Result, bail};
use bonesdeploy_core::paths;

pub(super) struct BuildScriptEnv<'a> {
    pub(super) project_name: &'a str,
    pub(super) build_user: &'a str,
    pub(super) web_root: &'a str,
    pub(super) deployment_dir: &'a Path,
    pub(super) build_cache_dir: &'a Path,
    pub(super) build_env_vars: &'a [(String, String)],
    /// Maximum seconds each build script may run before systemd terminates it.
    /// `None` disables the timeout.
    pub(super) script_timeout_seconds: Option<u64>,
}

pub(super) fn build_user_command(build_user: &str) -> Command {
    let mut command = Command::new("systemd-run");
    command.arg(format!("--machine={build_user}@")).args(["--quiet", "--user", "--collect", "--pipe", "--wait"]);
    command
}

/// A `build_user_command` that systemd terminates after `timeout_seconds`.
/// This bounds runaway build scripts without depending on Podman or the build
/// user's manager to cooperate: systemd enforces the deadline and kills the
/// script's whole process tree.
pub(super) fn build_script_command(build_user: &str, timeout_seconds: u64) -> Command {
    let mut command = build_user_command(build_user);
    command.arg(format!("--property=RuntimeMaxSec={timeout_seconds}s"));
    command
}

pub(super) fn build_user_control_command(build_user: &str) -> Command {
    let mut command = build_user_command(build_user);
    command.arg("--property=RuntimeMaxSec=20s");
    command
}

pub(crate) fn ensure_build_user_ready(build_user: &str, working_dir: &Path) -> Result<()> {
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

    let status = Command::new("systemctl")
        .args(["is-active", "--quiet", &format!("user@{uid}.service")])
        .status()
        .with_context(|| format!("Failed to inspect the systemd user manager for {build_user}"))?;
    if !status.success() {
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

fn identity_id(build_user: &str, flag: &str, label: &str) -> Result<u32> {
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

pub(crate) fn validate_build_cache(path: &Path, uid: u32, gid: u32) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use std::{
        env, fs,
        os::unix::fs::{MetadataExt, PermissionsExt},
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    use anyhow::Result;

    use super::*;

    fn temp_dir(prefix: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
        let path = env::temp_dir().join(format!("{prefix}_{nanos}"));
        fs::create_dir_all(&path)?;
        Ok(path)
    }

    #[test]
    fn build_cache_validation_requires_private_owned_directory() -> Result<()> {
        let root = temp_dir("bonesremote-build-cache")?;
        let cache = root.join("cache");
        fs::create_dir(&cache)?;
        fs::set_permissions(&cache, PermissionsExt::from_mode(0o700))?;
        let metadata = fs::metadata(&cache)?;
        validate_build_cache(&cache, metadata.uid(), metadata.gid())?;

        fs::set_permissions(&cache, PermissionsExt::from_mode(0o755))?;
        assert!(validate_build_cache(&cache, metadata.uid(), metadata.gid()).is_err());
        fs::remove_dir_all(root).ok();
        Ok(())
    }

    fn command_to_string(command: &Command) -> String {
        Command::new("sh")
            .arg("-c")
            .arg("printf '%s\\n' \"$@\"")
            .arg("dummy")
            .args(command.get_args())
            .output()
            .map(|output| String::from_utf8_lossy(&output.stdout).into_owned())
            .unwrap_or_default()
    }

    #[test]
    fn build_script_command_includes_runtime_max_sec() {
        let command = build_script_command("demo-build", 300);
        assert!(command_to_string(&command).contains("--property=RuntimeMaxSec=300s"));
    }

    #[test]
    fn plain_build_user_command_has_no_runtime_max_sec() {
        let command = build_user_command("demo-build");
        assert!(!command_to_string(&command).contains("RuntimeMaxSec"));
    }

    #[test]
    fn build_script_command_runs_as_the_build_user_machine() {
        let command = build_script_command("demo-build", 300);
        assert!(command_to_string(&command).contains("--machine=demo-build@"));
    }
}
