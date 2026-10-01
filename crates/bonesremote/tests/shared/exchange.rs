use std::fs;

use anyhow::Result;
use bonesremote::shared::exchange_directories;
use tempfile::tempdir;

#[test]
fn exchanges_two_real_directories_atomically() -> Result<()> {
    let root = tempdir()?;
    let left = root.path().join("left");
    let right = root.path().join("right");
    fs::create_dir(&left)?;
    fs::create_dir(&right)?;
    fs::write(left.join("value"), "old")?;
    fs::write(right.join("value"), "new")?;

    exchange_directories(&left, &right)?;

    assert_eq!(fs::read_to_string(left.join("value"))?, "new");
    assert_eq!(fs::read_to_string(right.join("value"))?, "old");
    Ok(())
}
