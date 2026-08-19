use std::collections::BTreeSet;
use std::env;
use std::error::Error;
use std::fs;
use std::io;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use bonesremote::commands::doctor::security::collection::filesystem::{collect_path_tree, collect_release};
use bonesremote::commands::doctor::security::types::{Account, Site};

fn temporary_root(name: &str) -> PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_nanos();
    env::temp_dir().join(format!("bonesremote-doctor-{name}-{}-{nonce}", process::id()))
}

fn site(root: &Path) -> Site {
    let account = Account {
        name: "atlas".to_string(),
        uid: 1001,
        gid: 1001,
        shell: "/usr/sbin/nologin".to_string(),
        groups: BTreeSet::from([1001]),
    };
    Site { name: "atlas".to_string(), project_root: root.to_path_buf(), runtime: account.clone(), build: account }
}

#[test]
fn protected_path_scan_does_not_follow_symlink_targets() -> Result<(), Box<dyn Error>> {
    let root = temporary_root("protected-symlink");
    let protected = root.join("protected");
    let outside = root.join("outside");
    fs::create_dir_all(&protected)?;
    fs::create_dir_all(&outside)?;
    fs::write(outside.join("unrelated"), "x")?;
    symlink(&outside, protected.join("outside"))?;

    let tree = collect_path_tree(&protected, false).map_err(io::Error::other)?;
    fs::remove_dir_all(&root)?;

    assert!(!tree.nodes.contains_key(&outside));
    assert!(!tree.nodes.contains_key(&outside.join("unrelated")));
    Ok(())
}

#[test]
fn release_boundary_scan_skips_nested_entries_unless_exhaustive() -> Result<(), Box<dyn Error>> {
    let root = temporary_root("release-boundary");
    let release = root.join("releases/2026-08-03");
    let nested_file = release.join("nested/application-file");
    fs::create_dir_all(nested_file.parent().ok_or("nested file has parent")?)?;
    fs::write(&nested_file, "x")?;
    symlink(&release, root.join("current"))?;
    let site = site(&root);

    let boundary = collect_release(&site, false).map_err(io::Error::other)?;
    let exhaustive = collect_release(&site, true).map_err(io::Error::other)?;
    fs::remove_dir_all(&root)?;

    assert!(boundary.filesystem.nodes.contains_key(&release));
    assert!(!boundary.filesystem.nodes.contains_key(&nested_file));
    assert!(exhaustive.filesystem.nodes.contains_key(&nested_file));
    Ok(())
}
