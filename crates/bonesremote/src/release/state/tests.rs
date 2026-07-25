use std::env;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use shared::paths;

use super::{
    ScopedRoot, clear_staged_release, current_release_name, list_releases_sorted, point_symlink_atomically,
    read_staged_release, release_dir, set_sites_root_for_tests, staged_release_path, write_staged_release,
};

fn temp_dir_path(test_name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_nanos());
    env::temp_dir().join(format!("bonesremote_release_state_test_{}_{}_{}", process::id(), nanos, test_name))
}

fn temp_root(test_name: &str) -> Result<(ScopedRoot, PathBuf)> {
    let path = temp_dir_path(test_name);
    fs::create_dir_all(&path)?;
    Ok((set_sites_root_for_tests(path.clone()), path))
}

fn project_root_for(root: &Path) -> String {
    root.join("deploy").to_string_lossy().to_string()
}

#[test]
fn write_then_read_staged_release_round_trips() -> Result<()> {
    let (_guard, _root) = temp_root("round_trip")?;

    write_staged_release("unitapp", "20260507_151500")?;
    let release_name = read_staged_release("unitapp")?;
    assert_eq!(release_name, "20260507_151500");

    Ok(())
}

#[test]
fn read_staged_release_rejects_empty_file() -> Result<()> {
    let (_guard, root) = temp_root("empty_state")?;
    let state_path = root.join("emptyapp").join(paths::STAGED_RELEASE_FILE);
    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&state_path, " \n")?;

    let result = read_staged_release("emptyapp");
    assert!(result.is_err());

    Ok(())
}

#[test]
fn clear_staged_release_removes_state_file() -> Result<()> {
    let (_guard, _root) = temp_root("clear_state")?;

    write_staged_release("clearapp", "20260507_151501")?;
    clear_staged_release("clearapp")?;
    assert!(!staged_release_path("clearapp").exists());

    Ok(())
}

#[test]
fn point_symlink_atomically_creates_parent_dirs_and_points_to_target() -> Result<()> {
    let root = temp_dir_path("point_symlink_parent");
    fs::create_dir_all(&root)?;

    let target = root.join("target_dir");
    fs::create_dir_all(&target)?;

    let link_path = root.join("nested/path/current");
    point_symlink_atomically(&link_path, &target)?;

    assert!(link_path.exists());
    let linked = fs::read_link(&link_path)?;
    assert_eq!(linked, target);

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn point_symlink_atomically_repoints_existing_link() -> Result<()> {
    let root = temp_dir_path("point_symlink_repoint");
    fs::create_dir_all(&root)?;

    let target_a = root.join("target_a");
    let target_b = root.join("target_b");
    fs::create_dir_all(&target_a)?;
    fs::create_dir_all(&target_b)?;

    let link_path = root.join(paths::CURRENT_LINK);
    point_symlink_atomically(&link_path, &target_a)?;
    point_symlink_atomically(&link_path, &target_b)?;

    let linked = fs::read_link(&link_path)?;
    assert_eq!(linked, target_b);

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn current_release_name_resolves_symlink_target_name() -> Result<()> {
    let root = temp_dir_path("current_release_name");
    fs::create_dir_all(&root)?;

    let project_root = project_root_for(&root);
    let release_path = release_dir(&project_root, "20260507_151502");
    fs::create_dir_all(&release_path)?;
    let current = Path::new(&project_root).join(paths::CURRENT_LINK);
    if let Some(parent) = current.parent() {
        fs::create_dir_all(parent)?;
    }
    symlink(&release_path, &current)?;

    assert_eq!(current_release_name(&project_root)?, "20260507_151502");

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn list_releases_sorted_skips_placeholder() -> Result<()> {
    let root = temp_dir_path("list_releases");
    fs::create_dir_all(&root)?;

    let project_root = project_root_for(&root);
    fs::create_dir_all(release_dir(&project_root, "20260507_151500"))?;
    fs::create_dir_all(release_dir(&project_root, paths::PLACEHOLDER_RELEASE_NAME))?;
    fs::create_dir_all(release_dir(&project_root, "20260507_151501"))?;

    assert_eq!(list_releases_sorted(&project_root)?, vec!["20260507_151500", "20260507_151501"]);

    fs::remove_dir_all(root).ok();
    Ok(())
}
