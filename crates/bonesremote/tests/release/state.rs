use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use bonesremote::release::state::{
    ScopedSitesRoot, clear_staged_release, override_sites_root, read_staged_release, staged_release, store,
    write_staged_release,
};

fn temp_root(test_name: &str) -> Result<(ScopedSitesRoot, PathBuf)> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    let path = env::temp_dir().join(format!("bonesremote_state_test_{}_{}_{}", process::id(), nanos, test_name));
    fs::create_dir_all(&path)?;
    Ok((override_sites_root(path.clone()), path))
}

#[test]
fn write_then_read_staged_release_round_trips() -> Result<()> {
    let (_guard, _root) = temp_root("round_trip")?;

    write_staged_release("unitapp", "20260507_151500")?;
    assert_eq!(read_staged_release("unitapp")?, "20260507_151500");

    Ok(())
}

#[test]
fn read_staged_release_rejects_missing_state() -> Result<()> {
    let (_guard, root) = temp_root("empty_state")?;

    assert!(read_staged_release("emptyapp").is_err());
    let state = store::read_state("emptyapp")?;
    assert!(state.staged_release().is_none());
    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn clear_staged_release_removes_the_pointer() -> Result<()> {
    let (_guard, _root) = temp_root("clear_state")?;

    write_staged_release("clearapp", "20260507_151501")?;
    assert_eq!(staged_release("clearapp")?.as_deref(), Some("20260507_151501"));
    clear_staged_release("clearapp")?;
    assert!(staged_release("clearapp")?.is_none());

    Ok(())
}
