use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use bonesremote::commands::deploy::rollback::switch_and_verify;

fn temp_root(prefix: &str) -> Result<PathBuf> {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nonce));
    fs::create_dir_all(&root)?;
    Ok(root)
}

#[test]
fn rollback_points_current_to_previous_release_when_restart_succeeds() -> Result<()> {
    let root = temp_root("bonesremote_rollback_ok")?;
    let current_link = root.join("current");
    let previous_dir = root.join("releases/20260101_000000");
    let _current_dir = root.join("releases/20260102_000000");
    fs::create_dir_all(&previous_dir)?;
    symlink(&previous_dir, &current_link)?;

    let project_root = root.to_string_lossy().into_owned();
    switch_and_verify(&project_root, "20260102_000000", "20260101_000000", || Ok(()))?;

    assert_eq!(fs::read_link(&current_link)?, previous_dir);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn rollback_restores_original_release_when_restart_fails() -> Result<()> {
    let root = temp_root("bonesremote_rollback_restore")?;
    let current_link = root.join("current");
    let current_dir = root.join("releases/20260102_000000");
    let previous_dir = root.join("releases/20260101_000000");
    fs::create_dir_all(&previous_dir)?;
    fs::create_dir_all(&current_dir)?;
    symlink(&current_dir, &current_link)?;

    let project_root = root.to_string_lossy().into_owned();
    let result = switch_and_verify(&project_root, "20260102_000000", "20260101_000000", || {
        anyhow::bail!("simulated restart failure")
    });

    assert!(result.is_err());
    assert_eq!(fs::read_link(&current_link)?, current_dir, "current must be restored after failed rollback");
    fs::remove_dir_all(root)?;
    Ok(())
}
