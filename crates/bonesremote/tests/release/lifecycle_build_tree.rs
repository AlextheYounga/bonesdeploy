use std::env;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use bonesremote::release::lifecycle::build::tree::{
    ensure_release_dir_empty, is_dir_empty, normalize_relative_path, prepare_release_tree, set_release_tree_identity,
    validate_symlink_target,
};

#[test]
fn promote_refuses_nonempty_release_directory() -> Result<()> {
    let root = env::temp_dir().join(format!("bonesremote-promote-nonempty-{}", process::id()));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    let source = root.join("source");
    let destination = root.join("releases").join("20260804_190321-46a0b75c-0000");
    fs::create_dir_all(source.join("index.html"))?;
    fs::write(source.join("index.html").join("x"), "x")?;
    fs::create_dir_all(&destination)?;
    fs::write(destination.join("existing.txt"), "do-not-touch")?;

    assert!(prepare_release_tree(&source, &destination, "root", "root").is_err());
    assert_eq!(fs::read_to_string(destination.join("existing.txt"))?, "do-not-touch");

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn empty_release_directory_passes_the_nonempty_guard() -> Result<()> {
    let root = temp_root("bonesremote-promote-empty");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    let destination = root.join("releases").join("candidate");
    fs::create_dir_all(&destination)?;

    assert!(is_dir_empty(&destination)?);
    assert!(ensure_release_dir_empty(&destination).is_ok());

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn nonempty_release_directory_fails_the_nonempty_guard() -> Result<()> {
    let root = temp_root("bonesremote-promote-filled");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    let destination = root.join("releases").join("candidate");
    fs::create_dir_all(&destination)?;
    fs::write(destination.join("file"), "x")?;

    assert!(ensure_release_dir_empty(&destination).is_err());

    fs::remove_dir_all(root).ok();
    Ok(())
}

fn temp_root(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0_u128, |duration| duration.as_nanos());
    env::temp_dir().join(format!("{prefix}_{}_{}", process::id(), nanos))
}

#[test]
fn candidate_tree_is_writable_by_its_temporary_owner() -> Result<()> {
    let root = temp_root("bonesremote-promote-writable");
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    let public = root.join("public");
    fs::create_dir_all(&public)?;
    fs::write(public.join("index.php"), "<?php")?;

    let metadata = fs::metadata(&root)?;
    set_release_tree_identity(&root, metadata.uid(), metadata.gid())?;

    assert_eq!(fs::metadata(&public)?.permissions().mode() & 0o777, 0o750);
    assert_eq!(fs::metadata(public.join("index.php"))?.permissions().mode() & 0o777, 0o640);

    fs::remove_dir_all(root).ok();
    Ok(())
}

#[test]
fn normalize_relative_path_rejects_escape() {
    let root = Path::new("/tmp/release-root");
    let escaped = normalize_relative_path(Path::new("/tmp/release-root/app/../../etc/passwd"), root);
    assert!(escaped.is_err());
}

#[test]
fn validate_symlink_target_rejects_absolute_and_escaping_targets() {
    let root = Path::new("/tmp/release-root");
    assert!(validate_symlink_target(Path::new("/tmp/release-root/x"), Path::new("/etc/passwd"), root).is_err());
    assert!(validate_symlink_target(Path::new("/tmp/release-root/public/x"), Path::new("../../evil"), root).is_err());
}
