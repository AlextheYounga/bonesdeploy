use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};
use bonesdeploy_core::config::validate_project_name;
use bonesdeploy_core::paths;

pub(crate) const APP_PATH: &str = "/app";
pub(crate) const SHARED_PATH: &str = "/bonesdeploy/shared";
pub(crate) const SOCKET_PATH: &str = "/run/bonesdeploy/php";

pub(crate) fn image_name(project: &str) -> Result<String> {
    validate_project_name(project)?;
    Ok(format!("bonesdeploy/laravel-{project}:runtime"))
}

pub(crate) fn container_name(project: &str) -> Result<String> {
    validate_project_name(project)?;
    Ok(format!("bonesdeploy-{project}"))
}

pub(crate) fn runtime_identity(user: &str) -> Result<String> {
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

pub(crate) fn application_command(
    project: &str,
    project_root: &Path,
    runtime_user: &str,
    image: &str,
) -> Result<Command> {
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

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::process;

    use anyhow::Result;

    use super::{APP_PATH, SHARED_PATH, application_command, container_name, image_name};

    #[test]
    fn names_are_site_scoped() -> Result<()> {
        assert_eq!(image_name("atlas")?, "bonesdeploy/laravel-atlas:runtime");
        assert_eq!(container_name("atlas")?, "bonesdeploy-atlas");
        Ok(())
    }

    #[test]
    fn application_command_has_restricted_mounts() -> Result<()> {
        let root = env::temp_dir().join(format!("bonesremote-docker-command-{}", process::id()));
        fs::create_dir_all(root.join("current"))?;
        fs::create_dir_all(root.join("shared"))?;
        let command = application_command("atlas", &root, "root", "image")?;
        let args = command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>().join(" ");
        assert!(args.contains("--read-only"));
        assert!(args.contains("--cap-drop=ALL"));
        assert!(args.contains("no-new-privileges"));
        assert!(args.contains(APP_PATH));
        assert!(args.contains(SHARED_PATH));
        assert!(!args.contains("docker.sock"));
        fs::remove_dir_all(root)?;
        Ok(())
    }
}
