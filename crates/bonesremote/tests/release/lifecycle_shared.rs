use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process;

use anyhow::Result;

use bonesdeploy_core::paths;

use bonesremote::release::lifecycle::wire_shared::{link_relative, remove_if_present};

fn temp_dir(label: &str) -> Result<PathBuf> {
    let dir = env::temp_dir().join(format!("bonesremote-wire-{label}-{}", process::id()));
    if dir.exists() {
        fs::remove_dir_all(&dir)?;
    }
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[test]
fn link_relative_creates_symlink_to_shared_target() -> Result<()> {
    let root = temp_dir("link_relative")?;
    let shared = root.join("shared/.env");
    let parent = shared.parent().ok_or_else(|| anyhow::anyhow!("shared test path should have a parent"))?;
    fs::create_dir_all(parent)?;
    fs::write(&shared, "FOO=bar\n")?;
    fs::set_permissions(&shared, PermissionsExt::from_mode(0o600))?;

    let release = root.join("releases/now");
    fs::create_dir_all(&release)?;
    link_relative(&release, paths::DOT_ENV, &shared)?;

    let link = release.join(".env");
    assert!(link.is_symlink());
    let linked_target = fs::read_link(&link)?;
    assert_eq!(linked_target, shared);
    assert_eq!(fs::read_to_string(&link)?, "FOO=bar\n");

    fs::remove_dir_all(&root).ok();
    Ok(())
}

#[test]
fn remove_if_present_handles_files_dirs_and_missing() -> Result<()> {
    let root = temp_dir("remove_if_present")?;
    let missing = root.join("missing");
    remove_if_present(&missing)?;

    let file = root.join("file.txt");
    fs::write(&file, "x")?;
    remove_if_present(&file)?;
    assert!(!file.exists());

    let dir = root.join("dir");
    fs::create_dir_all(&dir)?;
    remove_if_present(&dir)?;
    assert!(!dir.exists());

    fs::remove_dir_all(&root).ok();
    Ok(())
}
