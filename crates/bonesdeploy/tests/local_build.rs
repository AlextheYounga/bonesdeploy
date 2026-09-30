use std::fs;
use std::os::unix::fs::symlink;

use bonesdeploy::local_build::sanitize_exported_context;
use bonesdeploy_core::paths;

#[test]
fn exported_context_removes_root_environment_files_but_keeps_public_build_configuration() -> anyhow::Result<()> {
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
