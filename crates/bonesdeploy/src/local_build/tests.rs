use std::fs;
use std::os::unix::fs::symlink;

use super::{
    BUILDER_IMAGE, ContainerStart, MountOwnership, TARGET_PLATFORM_NAME, build_scripts, create_command,
    docker_info_command, force_remove_command, normalize_mount_ownership_command, sanitize_exported_context,
    target_probe_command,
};
use anyhow::{Context, anyhow};
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::paths;

#[test]
fn local_docker_command_uses_the_pinned_platform_and_workspace_contract() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let environment_file = tempfile::NamedTempFile::new()?;
    let config = Bones::default();
    let input = ContainerStart {
        source: temp.path(),
        deployment: temp.path(),
        cache: temp.path(),
        config: &config,
        environment: &[],
    };
    let command = create_command(&input, "build-1", environment_file.path());
    assert_eq!(command.get_program().to_string_lossy(), "docker");
    let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();

    assert!(args.windows(2).any(|pair| pair == ["--platform", TARGET_PLATFORM_NAME]));
    assert!(args.windows(2).any(|pair| pair == ["--pull=never", "--platform"]));
    assert!(args.iter().any(|arg| arg == BUILDER_IMAGE));
    assert!(args.iter().any(|arg| arg.ends_with(":/workspace/source")));
    assert!(args.iter().any(|arg| arg.ends_with(":/workspace/cache:rw")));
    assert!(!args.iter().any(|arg| arg.contains("/workspace/deployment")));
    assert!(args.iter().any(|arg| arg == "--security-opt=no-new-privileges"));
    assert!(!args.iter().any(|arg| arg == "--privileged" || arg.contains("docker.sock")));
    assert!(!args.iter().any(|arg| arg == "--user"));
    Ok(())
}

#[test]
fn ownership_cleanup_runs_as_container_root_without_following_symlinks() {
    let command = normalize_mount_ownership_command("build-1", MountOwnership { uid: 1001, gid: 1002 });
    let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();

    assert_eq!(command.get_program().to_string_lossy(), "docker");
    assert_eq!(args[0..5], ["exec", "--user", "0:0", "build-1", "sh"]);
    assert_eq!(args[5], "-c");
    assert_eq!(args[6], "find -P /workspace/source /workspace/cache -exec chown -h 1001:1002 {} +");
}

#[test]
fn cleanup_always_uses_forced_container_removal() {
    let command = force_remove_command("build-1");
    let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();

    assert_eq!(command.get_program().to_string_lossy(), "docker");
    assert_eq!(args, ["rm", "--force", "--time", "0", "--ignore", "build-1"]);
}

#[test]
fn cleanup_failure_is_attached_without_replacing_the_primary_build_error() {
    let result = super::finish_with_cleanup(Err(anyhow!("script failed")), Err(anyhow!("ownership failed")));
    assert!(result.is_err());
    let message = result.map_or_else(|error| format!("{error:#}"), |_| String::new());
    assert!(message.contains("script failed"));
    assert!(message.contains("cleanup also failed"));
    assert!(message.contains("ownership failed"));
}

#[test]
fn docker_info_probe_requires_a_linux_engine() {
    let command = docker_info_command();

    assert_eq!(command.get_program().to_string_lossy(), "docker");
    let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();
    assert_eq!(args.as_slice(), ["info", "--format", "{{.OSType}}"]);
}

#[test]
fn target_probe_uses_the_pinned_image_platform_and_never_pulls() {
    let command = target_probe_command();
    assert_eq!(command.get_program().to_string_lossy(), "docker");
    let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();

    assert_eq!(args, vec!["run", "--rm", "--pull=never", "--platform", TARGET_PLATFORM_NAME, BUILDER_IMAGE, "true"]);
}

#[test]
fn build_scripts_are_discovered_only_under_the_exported_context() -> anyhow::Result<()> {
    let exported = tempfile::tempdir()?;
    let worktree = tempfile::tempdir()?;
    let exported_build = exported.path().join("infra/deployment/build");
    let worktree_build = worktree.path().join("infra/deployment/build");
    fs::create_dir_all(&exported_build)?;
    fs::create_dir_all(&worktree_build)?;
    fs::write(exported_build.join("01_committed.sh"), "true")?;
    fs::write(worktree_build.join("01_dirty.sh"), "false")?;
    fs::write(worktree.path().join("infra/deployment/dirty.txt"), "dirty")?;

    let (deployment, scripts) = build_scripts(exported.path())?.context("committed script expected")?;

    assert_eq!(deployment, exported.path().join("infra/deployment"));
    assert_eq!(scripts, vec![exported_build.join("01_committed.sh")]);
    assert!(!deployment.join("dirty.txt").exists());
    Ok(())
}

#[test]
fn sanitize_exported_context_removes_root_env_but_keeps_env_build() -> anyhow::Result<()> {
    let context = tempfile::tempdir()?;
    fs::write(context.path().join(paths::DOT_ENV), "SECRET=tracked")?;
    fs::write(context.path().join(paths::ENV_BUILD_FILE), "NODE_VERSION=24")?;

    sanitize_exported_context(context.path())?;

    assert!(!context.path().join(paths::DOT_ENV).exists());
    assert_eq!(fs::read_to_string(context.path().join(paths::ENV_BUILD_FILE))?, "NODE_VERSION=24");

    symlink(paths::ENV_BUILD_FILE, context.path().join(paths::DOT_ENV))?;
    sanitize_exported_context(context.path())?;
    assert!(!context.path().join(paths::DOT_ENV).exists());
    assert_eq!(fs::read_to_string(context.path().join(paths::ENV_BUILD_FILE))?, "NODE_VERSION=24");
    Ok(())
}
