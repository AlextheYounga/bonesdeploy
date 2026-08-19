use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};
use bonesdeploy_core::config::validate_project_name;
use bonesdeploy_core::paths;

pub const APP_PATH: &str = "/app";
pub const SHARED_PATH: &str = "/bonesdeploy/shared";
pub const SOCKET_PATH: &str = "/run/bonesdeploy/php";

pub fn image_name(project: &str) -> Result<String> {
    validate_project_name(project)?;
    Ok(format!("bonesdeploy/laravel-{project}:runtime"))
}

pub fn container_name(project: &str) -> Result<String> {
    validate_project_name(project)?;
    Ok(format!("bonesdeploy-{project}"))
}

pub fn runtime_identity(user: &str) -> Result<String> {
    let passwd = fs::read_to_string(paths::ETC_PASSWD)?;
    let entry = passwd.lines().find(|line| line.split(':').next() == Some(user));
    let Some(entry) = entry else {
        bail!("Runtime user does not exist: {user}");
    };
    let fields = entry.split(':').collect::<Vec<_>>();
    if fields.len() < 4 {
        bail!("Runtime user entry is malformed: {user}");
    }
    Ok(format!("{}:{}", fields[2], fields[3]))
}

pub fn application_command(project: &str, project_root: &Path, runtime_user: &str, image: &str) -> Result<Command> {
    let container = container_name(project)?;
    let source = project_root.join("current");
    let shared = project_root.join("shared");
    if !source.is_dir() {
        bail!("Active release is missing: {}", source.display());
    }
    if !shared.is_dir() {
        bail!("Shared directory is missing: {}", shared.display());
    }

    let identity = runtime_identity(runtime_user)?;
    let mut command = Command::new("docker");
    command
        .args([
            "run",
            "--name",
            &container,
            "--rm",
            "--read-only",
            "--cap-drop=ALL",
            "--security-opt=no-new-privileges",
            "--tmpfs",
            "/tmp:rw,noexec,nosuid,nodev",
            "--mount",
            &format!("type=bind,src={},dst={},readonly", source.display(), APP_PATH),
            "--mount",
            &format!("type=bind,src={},dst={},rw", shared.display(), SHARED_PATH),
            "--mount",
            &format!("type=bind,src=/run/{project},dst={},rw", SOCKET_PATH),
            "--user",
            &identity,
            image,
            "php-fpm",
            "-F",
        ])
        .current_dir(project_root);
    Ok(command)
}
