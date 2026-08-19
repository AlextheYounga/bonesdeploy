use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use bonesdeploy_core::paths;

use bonesremote::release::state::atomic::atomic_write;

fn temp_dir(prefix: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos));
    fs::create_dir_all(&path)?;
    Ok(path)
}

#[test]
fn atomic_write_creates_parent_and_persists_content() -> Result<()> {
    let root = temp_dir("bonesremote_atomic_new")?;
    let target = root.join("nested").join(paths::ACTIVE_DEPLOYMENT_FILE);

    atomic_write(&target, b"{\"phase\":\"building\"}")?;

    assert_eq!(fs::read_to_string(&target)?, "{\"phase\":\"building\"}");

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn atomic_write_replaces_existing_content_and_keeps_mode() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let root = temp_dir("bonesremote_atomic_replace")?;
    let target = root.join(paths::ACTIVE_DEPLOYMENT_FILE);
    fs::write(&target, "old")?;
    fs::set_permissions(&target, fs::Permissions::from_mode(0o600))?;

    atomic_write(&target, b"new")?;

    assert_eq!(fs::read_to_string(&target)?, "new");
    assert_eq!(fs::metadata(&target)?.permissions().mode() & 0o777, 0o600);

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn atomic_write_leaves_no_temporary_file_behind() -> Result<()> {
    let root = temp_dir("bonesremote_atomic_no_tmp")?;
    let target = root.join(paths::STAGED_RELEASE_FILE);
    fs::write(&target, "stale")?;

    atomic_write(&target, b"20260804_190321-46a0b75c-a7f2\n")?;

    let leftovers: Vec<_> = fs::read_dir(&root)?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_name().to_string_lossy().contains(".tmp-"))
        .collect();
    assert!(leftovers.is_empty(), "temporary state files must be renamed away");

    fs::remove_dir_all(root).ok();
    Ok(())
}
