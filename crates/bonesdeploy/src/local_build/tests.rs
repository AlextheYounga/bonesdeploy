use std::fs;
use std::os::unix::fs::symlink;

use super::{
    BUILDER_IMAGE, ContainerStart, TARGET_PLATFORM_NAME, build_scripts, create_command, sanitize_exported_context,
    target_probe_command,
};
use anyhow::Context;
use bonesdeploy_core::config::Bones;
use bonesdeploy_core::paths;

#[test]
fn local_podman_command_uses_the_pinned_platform_and_workspace_contract() -> anyhow::Result<()> {
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
    let args: Vec<_> = command.get_args().map(|arg| arg.to_string_lossy().into_owned()).collect();

    assert!(args.windows(2).any(|pair| pair == ["--platform", TARGET_PLATFORM_NAME]));
    assert!(args.windows(2).any(|pair| pair == ["--pull=never", "--platform"]));
    assert!(args.iter().any(|arg| arg == BUILDER_IMAGE));
    assert!(args.iter().any(|arg| arg.ends_with(":/workspace/source")));
    assert!(args.iter().any(|arg| arg.ends_with(":/workspace/cache:rw")));
    assert!(!args.iter().any(|arg| arg.contains("/workspace/deployment")));
    Ok(())
}

#[test]
fn target_probe_uses_the_pinned_image_platform_and_never_pulls() {
    let command = target_probe_command();
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
